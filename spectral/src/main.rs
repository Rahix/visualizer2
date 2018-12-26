use vis_core::analyzer;
use sfml::{system, graphics};

const BUCKETS: usize = 200;
const LINES: usize = 200;

struct AnalyzerResult {
    spectrum: analyzer::Spectrum<Vec<f32>>,
}

fn main() {
    use sfml::graphics::RenderTarget;

    vis_core::default_config();
    vis_core::default_log();

    let context_settings = sfml::window::ContextSettings {
        antialiasing_level: 4,
        .. Default::default()
    };

    let mut window = sfml::graphics::RenderWindow::new(
        (1600, 800),
        "Visualizer2 Spectrum Display",
        sfml::window::Style::CLOSE,
        &context_settings
    );
    window.set_vertical_sync_enabled(true);
    window.clear(&graphics::Color::BLACK);
    window.display();

    let view = graphics::View::from_rect(&graphics::Rect::new(0.0, 0.0, 1.0, LINES as f32));
    window.set_view(&view);

    let mut texture = graphics::Texture::new(1600, 800).unwrap();

    let mut rectangle = graphics::RectangleShape::new();
    rectangle.set_size(system::Vector2f::new(1.0 / BUCKETS as f32, 1.0));

    // Analyzer {{{
    let mut spectralizer = analyzer::FourierBuilder::new()
        .plan();

    let mut avg = analyzer::Spectrum::new(vec![0.0; spectralizer.buckets], 0.0, 1.0);

    let mut frames = vis_core::Visualizer::new(
        AnalyzerResult {
            spectrum: analyzer::Spectrum::new(vec![0.0; BUCKETS], spectralizer.lowest, spectralizer.hightest),
        },
        move |info, samples| {
            analyzer::average_spectrum(
                &mut avg,
                &spectralizer.analyze(&samples),
            );

            avg.fill_buckets(&mut info.spectrum.buckets[..]);

            info
        },
    )
    .frames();
    // }}}

    'main: for frame in frames.iter() {
        log::trace!("Frame: {:7}@{:.3}", frame.frame, frame.time);

        while let Some(event) = window.poll_event() {
            use sfml::window::Event;

            match event {
                Event::Closed => break 'main,
                Event::KeyPressed {
                    code: sfml::window::Key::Escape, ..
                } => break 'main,
                _ => (),
            }
        }

        // Move window content up
        {
            use sfml::graphics::Transformable;

            texture.update_from_render_window(&window, 0, 0);
            window.clear(&graphics::Color::BLACK);
            let mut rect_img = graphics::RectangleShape::with_texture(&texture);
            rect_img.set_size(system::Vector2f::new(1.0, LINES as f32));
            rect_img.set_position(system::Vector2f::new(0.0, -1.0));
            window.draw(&rect_img);
        }

        frame.lock_info(|info| {
            let max = info.spectrum.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

            for (i, b) in info.spectrum.iter().enumerate() {
                use sfml::graphics::Transformable;
                use sfml::graphics::Shape;

                let int = ((b / max).sqrt() * 255.0) as u8;
                rectangle.set_fill_color(&graphics::Color::rgb(int, int, int));
                rectangle.set_position(system::Vector2f::new(i as f32 / BUCKETS as f32, LINES as f32 - 1.0));
                window.draw(&rectangle);
            }
        });

        window.display();
        std::thread::sleep_ms(10);
    }
}
