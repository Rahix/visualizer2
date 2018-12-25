extern crate vis_core;

pub struct AnalyzerResult {
    volume: f32,
}

fn main() {
    vis_core::default_log();
    vis_core::default_config();

    let mut analyzer = vis_core::analyzer::FourierBuilder::new()
        .length(512)
        .window(vis_core::analyzer::window::nuttall)
        .plan();

    let mut frames = vis_core::Visualizer::new()
        .analyzer(|samples| {
            analyzer.analyze(samples);

            AnalyzerResult {
                volume: samples.volume(0.01),
            }
        })
        .frames();

    for frame in frames.iter() {
        for _ in 0..(200.0 * frame.info.volume) as usize {
            print!("#");
        }
        println!("");
    }
}
