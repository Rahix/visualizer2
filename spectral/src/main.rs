use vis_core::analyzer;
use sfml::{system, graphics};

const BUCKETS: usize = 200;
const LINES: usize = 200;

#[derive(Debug, Clone)]
struct AnalyzerResult {
    analyzer: analyzer::FourierAnalyzer,
    average: analyzer::Spectrum<Vec<f32>>,
    beat: usize,
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
    let mut frames = {
        let mut analyzer = analyzer::FourierBuilder::new().plan();
        let mut average = analyzer::Spectrum::new(vec![0.0; analyzer.buckets()], 0.0, 1.0);

        // Beat
        let mut beat = analyzer::BeatBuilder::new().build();
        let mut beat_num = 0;

        vis_core::Visualizer::new(
            AnalyzerResult {
                analyzer,
                average,
                beat: 0,
            },
            move |info, samples| {
                info.analyzer.analyze(&samples);

                info.average.fill_from(&info.analyzer.average());

                if beat.detect(&samples) {
                    beat_num += 1;
                }
                info.beat = beat_num;

                info
            },
        )
        .frames()
    };
    // }}}

    let mut last_beat = 0;
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

        frame.info(|info| {
            use sfml::graphics::Shape;

            let max = info.average.max();
            let n50 = info.average.freq_to_id(50.0);
            let n100 = info.average.freq_to_id(100.0);

            let beat = if info.beat > last_beat {
                last_beat = info.beat;
                rectangle.set_fill_color(&graphics::Color::rgb(255, 255, 255));
                true
            } else {
                false
            };

            for (i, b) in info.average.iter().enumerate() {
                use sfml::graphics::Transformable;

                let int = ((b / max).sqrt() * 255.0) as u8;
                if !beat {
                    rectangle.set_fill_color(&graphics::Color::rgb(int, int, int));
                    if i == n50 || i == n100 {
                        rectangle.set_fill_color(&graphics::Color::rgb(255, 0, 0));
                    }
                }
                rectangle.set_position(system::Vector2f::new(i as f32 / BUCKETS as f32, LINES as f32 - 1.0));
                window.draw(&rectangle);
            }
        });

        window.display();
        std::thread::sleep_ms(10);
    }
}
