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

    let mut spectra = [
        vis_core::analyzer::Spectrum::new(vec![0.0; analyzer.buckets], 0.0, 1.0),
        vis_core::analyzer::Spectrum::new(vec![0.0; analyzer.buckets], 0.0, 1.0),
    ];

    let mut frames = vis_core::Visualizer::new()
        .analyzer(move |samples| {
            analyzer.analyze(samples, &mut spectra);

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
