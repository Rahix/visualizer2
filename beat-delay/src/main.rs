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

            let volume = avg.max();
            info.volume = volume;
            let delta = volume - last_volume;

            if delta > 0.0 && last_delta < 0.0 && last_volume > (info.beat_volume * 0.6) {
                info.beat = true;
                info.beat_volume = last_volume;
            }

            last_volume = volume;
            if delta != 0.0 {
                last_delta = delta;
            }

            info
        },
    )
    .frames();

    // frames.detach_analyzer();

    'main: for frame in frames.iter() {
        log::trace!("Frame: {:7}@{:.3}", frame.frame, frame.time);

        frame.lock_info(|info| {
            if info.beat {
                println!("Beat@{:.3}: {:7.3}", frame.time, info.beat_volume);
            }
            info.beat = false;
        });

        std::thread::sleep_ms(1);
    }
}
