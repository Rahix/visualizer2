use vis_core::analyzer;
use sfml::{system, graphics};

struct AnalyzerResult {
    volume: f32,
    now: f32,
    beat: f32,
}

fn main() {
    use sfml::graphics::RenderTarget;
    use sfml::graphics::Transformable;
    use sfml::graphics::Shape;

    vis_core::default_config();
    vis_core::default_log();

    let context_settings = sfml::window::ContextSettings {
        antialiasing_level: 4,
        .. Default::default()
    };

    let mut window = sfml::graphics::RenderWindow::new(
        (400, 400),
        "Visualizer2 Beat Delay",
        sfml::window::Style::CLOSE,
        &context_settings
    );
    window.set_vertical_sync_enabled(true);
    window.clear(&graphics::Color::BLACK);
    window.display();

    let view = graphics::View::from_rect(&graphics::Rect::new(0.0, 0.0, 2.0, 2.0));
    window.set_view(&view);

    let mut circle = graphics::CircleShape::new(1.0, 64);
    circle.set_position(system::Vector2f::new(0.0, 0.0));

    // Analyzer {{{
    let mut spectralizer = analyzer::FourierBuilder::new()
        .plan();

    let mut avg = analyzer::Spectrum::new(vec![0.0; spectralizer.buckets], 0.0, 1.0);
    let start = std::time::Instant::now();
    let mut last_vol = 0.0;
    let mut last_delta = 0.0;

    let mut frames = vis_core::Visualizer::new(
        AnalyzerResult {
            volume: 0.0,
            now: 0.0,
            beat: -10.0,
        },
        move |info, samples| {
            analyzer::average_spectrum(
                &mut avg,
                &spectralizer.analyze(samples),
            );

            info.now = vis_core::helpers::time(start);

            let volume = avg.slice(50.0, 100.0).max();
            let delta = volume - last_vol;

            if delta < 0.0 && last_delta > 0.0 {
                info.beat = info.now;
            }

            info.volume = volume;
            last_vol = volume;

            if delta != 0.0 {
                last_delta = delta;
            }

            info
        },
    )
    .frames();
    // }}}

    let mut actual_time = 10000000.0;
    let mut delays = vec![];

    'main: for frame in frames.iter() {
        log::trace!("Frame: {:7}@{:.3}", frame.frame, frame.time);

        let mut actual = false;
        while let Some(event) = window.poll_event() {
            use sfml::window::Event;

            match event {
                Event::Closed => break 'main,
                Event::KeyPressed {
                    code: sfml::window::Key::Escape, ..
                } => break 'main,
                Event::KeyPressed {
                    code: sfml::window::Key::Space, ..
                } => actual = true,
                _ => (),
            }
        }

        let beat = frame.lock_info(|info| {
            let t = 1.0 - (info.now - info.beat) * 5.0;
            let int = ((t * 255.0).max(0.0) as u8).min(255);

            if actual {
                actual_time = info.now;
            }

            circle.set_fill_color(&graphics::Color::rgb(int, int, int));

            info.beat
        });

        if beat > actual_time {
            let delay = beat - actual_time;
            delays.push(delay);
            println!("Delay: {:8.3}s", delay);
            actual_time = 1000000.0;
        }

        window.clear(&graphics::Color::BLACK);

        window.draw(&circle);

        window.display();
        std::thread::sleep_ms(10);
    }

    println!("Mean delay: {:8.3}s", delays.iter().sum::<f32>() / delays.len() as f32);
}
