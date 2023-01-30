#[macro_use]
extern crate log;
extern crate nalgebra as na;

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
    let slowdown = vis_core::CONFIG.get_or("noa.cols.slowdown", 0.995);

    let frame_time =
        std::time::Duration::from_micros(1000000 / vis_core::CONFIG.get_or("noa.fps", 30));

    // Colors
    let colors: Vec<[f32; 4]> = vis_core::CONFIG.get_or(
        "noa.cols.colors",
        vec![
            [1.0, 0.007443, 0.318893, 1.0],
            [0.915586, 0.704283, 0.214133, 1.0],
            [0.044844, 0.64629, 0.590788, 1.0],
            [0.130165, 0.022207, 0.27614, 1.0],
            [1.0, 0.007443, 0.318893, 1.0],
            [0.915586, 0.704283, 0.214133, 1.0],
            [0.044844, 0.64629, 0.590788, 1.0],
            [0.130165, 0.022207, 0.27614, 1.0],
            [1.0, 0.007443, 0.318893, 1.0],
            [0.915586, 0.704283, 0.214133, 1.0],
            [0.044844, 0.64629, 0.590788, 1.0],
            [0.130165, 0.022207, 0.27614, 1.0],
        ],
    );
    let note_roll_size = vis_core::CONFIG.get_or("noa.cols.note_roll", 20) as f32;

    // }}}

    let mut previous_time = 0.0;
    let mut rolling_volume = 0.0;
    let mut last_beat = -100.0;

    let mut notes_spectrum = analyzer::Spectrum::new(vec![0.0; notes_num], 220.0, 660.0);
    dbg!(&notes_spectrum);
    let mut notes_rolling_buf = vec![0.0; notes_num];

    let mut last_beat_num = 0;

    let mut maxima_buf = [(0.0, 0.0); 8];

    for frame in frames.iter() {

        let start = std::time::Instant::now();
        let delta = frame.time - previous_time;
        trace!("Delta: {}s", delta);

        // Audio Info Retrieval {{{
        let (volume, maxima, notes_rolling_spectrum, base_volume) = frame.info(|info| {
            rolling_volume = info.volume.max(rolling_volume * slowdown);

            if info.beat != last_beat_num {
                last_beat = frame.time;
                last_beat_num = info.beat;
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

        let mut columns = vec![false; notes_num];
        for (f, _) in maxima.iter().take(4) {
            let note = notes_rolling_spectrum.freq_to_id(*f);
            columns[note] = true;
        }

        if columns[0] {
            print!("\x1B[48;2;92;38;134m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[1] {
            print!("\x1B[48;2;255;22;144m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[2] {
            print!("\x1B[48;2;244;214;118m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[3] {
            print!("\x1B[48;2;54;205;196m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[4] {
            print!("\x1B[48;2;92;38;134m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[5] {
            print!("\x1B[48;2;255;22;144m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[6] {
            print!("\x1B[48;2;244;214;118m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[7] {
            print!("\x1B[48;2;54;205;196m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[8] {
            print!("\x1B[48;2;92;38;134m  ");
        } else {
            print!("\x1B[0m  ");
        }
        if columns[9] {
            print!("\x1B[48;2;255;22;144m  ");
        } else {
            print!("\x1B[0m  ");
        }
        println!("\x1B[0m|");

        previous_time = frame.time;

        let end = std::time::Instant::now();
        let dur = end - start;
        if dur < frame_time {
            let sleep = frame_time - dur;
            std::thread::sleep(sleep);
        }
    }
}
