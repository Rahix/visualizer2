#[macro_use]
extern crate log;
extern crate nalgebra as na;
use midir::{MidiOutput, MidiOutputPort};


use vis_core::analyzer;

#[derive(Debug, Clone)]
pub struct VisInfo {
    beat: u64,
    beat_volume: f32,
    volume: f32,
    analyzer: analyzer::FourierAnalyzer,
    spectrum: analyzer::Spectrum<Vec<f32>>,
}

fn main() {
    vis_core::default_config();
    vis_core::default_log();

    let mut frames = {
        // Analyzer {{{
        let mut beat = analyzer::BeatBuilder::new().build();
        let mut beat_num = 0;

        let analyzer = analyzer::FourierBuilder::new().plan();

        vis_core::Visualizer::new(
            VisInfo {
                beat: 0,
                beat_volume: 0.0,
                volume: 0.0,
                spectrum: analyzer::Spectrum::new(vec![0.0; analyzer.buckets()], 0.0, 1000.0),
                analyzer,
            },
            move |info, samples| {
                if beat.detect(&samples) {
                    beat_num += 1;
                }
                info.beat = beat_num;
                info.beat_volume = beat.last_volume();
                info.volume = samples.volume(0.3);

                info.analyzer.analyze(&samples);
                info.spectrum.fill_from(&info.analyzer.average());

                info
            },
        )
        .async_analyzer(300)
        .frames()
        // }}}
    };

    // Config {{{

    // Columns
    let notes_num = 10;
    let slowdown = vis_core::CONFIG.get_or("noa.cols.slowdown", 0.95);

    let frame_time =
        std::time::Duration::from_micros(1000000 / vis_core::CONFIG.get_or("noa.fps", 30));

    let note_roll_size = vis_core::CONFIG.get_or("noa.cols.note_roll", 20) as f32;

    // }}}

    let midi_out = MidiOutput::new("no-midi Music Visualizer").unwrap();

    // Get an output port (read from console if multiple are available)
    let out_ports = midi_out.ports();
    let out_port: &MidiOutputPort = match out_ports.len() {
        0 => panic!("no MIDI output port found"),
        _ => {
            log::debug!("Available output ports:");
            for p in out_ports.iter() {
                log::debug!(" - {}", midi_out.port_name(p).unwrap());
            }

            if let Some(want_port) = vis_core::CONFIG.get::<String>("midi.output_port") {
                let mut out_port = None;
                for p in out_ports.iter() {
                    if want_port == midi_out.port_name(p).unwrap() {
                        log::debug!("Chose wanted MIDI output port {:?}", want_port);
                        out_port = Some(p);
                    }
                }
                out_port.unwrap_or_else(|| {
                    panic!("Wanted MIDI output port {:?} not found!", want_port)
                })
            } else {
                log::debug!("Choosing MIDI port {:?}", midi_out.port_name(&out_ports[0]));
                &out_ports[0]
            }
        }
    };
    let mut conn_out = midi_out.connect(out_port, "midir-test").unwrap();

    let mut previous_time = 0.0;
    let mut rolling_volume = 0.0;
    let mut last_beat = -100.0;

    let mut notes_spectrum = analyzer::Spectrum::new(vec![0.0; notes_num], 220.0, 660.0);
    let mut notes_rolling_buf = vec![0.0; notes_num];

    let mut last_beat_num = 0;

    let mut maxima_buf = [(0.0, 0.0); 8];

    let mut previous_columns = vec![false; notes_num];
    let mut beat_ended = true;

    for frame in frames.iter() {

        let start = std::time::Instant::now();
        let delta = frame.time - previous_time;
        trace!("Delta: {}s", delta);

        // Audio Info Retrieval {{{
        let (_volume, maxima, notes_rolling_spectrum, _base_volume) = frame.info(|info| {
            rolling_volume = info.volume.max(rolling_volume * slowdown);

            if info.beat != last_beat_num {
                last_beat = frame.time;
                last_beat_num = info.beat;
                beat_ended = false;
            }

            let notes_spectrum = info.spectrum.fill_spectrum(&mut notes_spectrum);

            for (n, s) in notes_rolling_buf.iter_mut().zip(notes_spectrum.iter()) {
                *n = (*n * (note_roll_size - 1.0) + s) / note_roll_size;
            }
            let notes_rolling_spectrum = vis_core::analyzer::Spectrum::new(
                &mut *notes_rolling_buf,
                notes_spectrum.lowest(),
                notes_spectrum.highest(),
            );

            let maxima = notes_rolling_spectrum.find_maxima(&mut maxima_buf);

            (
                info.volume,
                maxima,
                notes_rolling_spectrum,
                info.beat_volume,
            )
        });
        // }}}

        const NOTE_ON_MSG: u8 = 0x90;
        const NOTE_OFF_MSG: u8 = 0x80;
        const VELOCITY: u8 = 0x7f;

        // let vol_float = (rolling_volume.powf(0.5) / 0.50).min(1.0).powi(2).max(0.15);
        let vol_float = (((rolling_volume / 0.18).powf(0.6) - 0.2) / 0.8).min(1.0).max(0.15);
        let vol = (vol_float * 127.0) as u8;
        conn_out.send(&[NOTE_ON_MSG, 70 as u8, vol]).unwrap();

        let beat_dur = 0.1;
        if frame.time == last_beat && vol_float != 0.15 {
            conn_out.send(&[NOTE_ON_MSG, 66 as u8, VELOCITY]).unwrap();
        } else if frame.time - last_beat > beat_dur && !beat_ended {
            conn_out.send(&[NOTE_OFF_MSG, 66 as u8, VELOCITY]).unwrap();
            beat_ended = true;
        }

        let chars = if frame.time - last_beat <= beat_dur && vol_float != 0.15 {
            "XX"
        } else {
            "  "
        };

        let mut columns = vec![false; notes_num];
        for (f, _) in maxima.iter().take(3) {
            let note = notes_rolling_spectrum.freq_to_id(*f);
            columns[note] = true;
        }

        for (i, (prev, now)) in previous_columns.iter().copied().zip(columns.iter().copied()).enumerate() {
            if !prev && now {
                conn_out.send(&[NOTE_ON_MSG, 50 + i as u8, VELOCITY]).unwrap();
            } else if prev && !now {
                conn_out.send(&[NOTE_OFF_MSG, 50 + i as u8, VELOCITY]).unwrap();
            }
        }

        if columns[0] {
            print!("\x1B[48;2;92;38;134m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[1] {
            print!("\x1B[48;2;255;22;144m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[2] {
            print!("\x1B[48;2;244;214;118m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[3] {
            print!("\x1B[48;2;54;205;196m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[4] {
            print!("\x1B[48;2;92;38;134m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[5] {
            print!("\x1B[48;2;255;22;144m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[6] {
            print!("\x1B[48;2;244;214;118m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[7] {
            print!("\x1B[48;2;54;205;196m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[8] {
            print!("\x1B[48;2;92;38;134m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        if columns[9] {
            print!("\x1B[48;2;255;22;144m{}", chars);
        } else {
            print!("\x1B[0m{}", chars);
        }
        print!("\x1B[0m| ");

        for i in 0..64 {
            if i < vol /2 {
                print!("=");
            } else {
                print!(" ");
            }
        }

        print!(" {vol_float:5.3}");

        println!("");



        previous_time = frame.time;
        previous_columns = columns;

        let end = std::time::Instant::now();
        let dur = end - start;
        if dur < frame_time {
            let sleep = frame_time - dur;
            std::thread::sleep(sleep);
        }
    }
}
