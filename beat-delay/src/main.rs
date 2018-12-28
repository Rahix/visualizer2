use vis_core::analyzer;

struct VisInfo {
    beat: bool,
    beat_volume: f32,
    volume: f32,
}

fn main() {
    vis_core::default_config();
    vis_core::default_log();

    let mut spectralizer = analyzer::FourierBuilder::new()
        .length(16)
        .downsample(20)
        .plan();

    let mut avg = analyzer::Spectrum::new(vec![0.0; spectralizer.buckets], 0.0, 1.0);
    let mut beat = analyzer::BeatBuilder::new()
        .decay(1000.0)
        .trigger(0.5)
        .build();

    let start = std::time::Instant::now();
    let mut last_volume = 0.0;
    let mut last_delta = 0.0;

    let mut frames = vis_core::Visualizer::new(
        VisInfo {
            beat: false,
            beat_volume: 0.0,
            volume: 0.0,
        },
        move |info, samples| {
            analyzer::average_spectrum(
                &mut avg,
                &spectralizer.analyze(samples),
            );

            let isbeat = beat.detect(&avg);

            if isbeat {
                info.beat = true;
                info.beat_volume = beat.last_volume();
            }
            info.volume = beat.last_volume();

            info
        },
    )
    .frames();

    frames.detach_analyzer();

    'main: for frame in frames.iter() {
        log::trace!("Frame: {:7}@{:.3}", frame.frame, frame.time);

        frame.lock_info(|info| {
            if info.beat {
                println!("Beat@{:.3}: {:7.3}", frame.time, info.beat_volume);
                // for _ in 0..(info.beat_volume * 100.0) as usize {
                //     print!("#");
                // }
                // println!("");
            }
            info.beat = false;

            if false {
                for _ in 0..(info.volume * 100.0) as usize {
                    print!("-");
                }
                println!("");
            }
        });

        std::thread::sleep_ms(30);
    }
}
