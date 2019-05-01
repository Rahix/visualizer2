//! A framework for audio-visualization in Rust.
//!
//! # Example
//! ```rust
//! // The data-type for storing analyzer results
//! #[derive(Debug, Clone)]
//! pub struct AnalyzerResult {
//!     spectrum: vis_core::analyzer::Spectrum<Vec<f32>>,
//!     volume: f32,
//!     beat: f32,
//! }
//!
//! fn main() {
//!     // Initialize the logger.  Take a look at the sources if you want to customize
//!     // the logger.
//!     vis_core::default_log();
//!
//!     // Load the default config source.  More about config later on.  You can also
//!     // do this manually if you have special requirements.
//!     vis_core::default_config();
//!
//!     // Initialize some analyzer-tools.  These will be moved into the analyzer closure
//!     // later on.
//!     let mut analyzer = vis_core::analyzer::FourierBuilder::new()
//!         .length(512)
//!         .window(vis_core::analyzer::window::nuttall)
//!         .plan();
//!
//!     let spectrum = vis_core::analyzer::Spectrum::new(vec![0.0; analyzer.buckets()], 0.0, 1.0);
//!
//!     let mut frames = vis_core::Visualizer::new(
//!         AnalyzerResult {
//!             spectrum,
//!             volume: 0.0,
//!             beat: 0.0,
//!         },
//!         // This closure is the "analyzer".  It will be executed in a loop to always
//!         // have the latest data available.
//!         move |info, samples| {
//!             analyzer.analyze(samples);
//!
//!             info.spectrum.fill_from(&analyzer.average());
//!             info.volume = samples.volume(0.3) * 400.0;
//!             info.beat = info.spectrum.slice(50.0, 100.0).max() * 0.01;
//!             info
//!         },
//!     )
//!     // Build the frame iterator which is the base of your loop later on
//!     .frames();
//!
//!     for frame in frames.iter() {
//!         // This is just a primitive example, your vis code belongs here
//!
//!         // Inside this closure you have access to the latest data from
//!         // the analyzer
//!         frame.info(|info| {
//!             for _ in 0..info.volume as usize {
//!                 print!("#");
//!             }
//!             println!("");
//!         });
//!         std::thread::sleep(std::time::Duration::from_millis(30));
//! #
//! #       if frame.frame > 20 {
//! #           break;
//! #       }
//!     }
//! }
//! ```
pub mod analyzer;
pub mod frames;
pub mod helpers;
pub mod recorder;
pub mod visualizer;

#[doc(inline)]
pub use crate::frames::Frames;
#[doc(inline)]
pub use crate::visualizer::Visualizer;

/// `ezconf` configuration
///
/// Usually you will call [`default_config`](fn.default_config.html) in the beginning
/// which will populate this object, but you can also specify your own custom config
/// sources.
///
/// # Example
/// To make use of this config, use code similar to this:
///
/// ```rust
/// # vis_core::default_config();
/// let some_configurable_value = vis_core::CONFIG.get_or(
///     // Toml path to value
///     "foo.bar",
///     // Default value.  Type gets inferred from this
///     123,
/// );
/// ```
pub static CONFIG: ezconf::Config = ezconf::INIT;

/// Initialize config from default sources
///
/// The default sources are:
/// * `./visualizer.toml`
/// * `./config/visualizer.toml`
/// * Defaults from code
pub fn default_config() {
    CONFIG
        .init(
            [
                ezconf::Source::File("visualizer.toml"),
                ezconf::Source::File("config/visualizer.toml"),
            ]
            .iter(),
        )
        .expect("Can't load config");
}

/// Initialize logger
///
/// By default, enable debug output in debug-builds.
pub fn default_log() {
    #[cfg(not(debug_assertions))]
    env_logger::init();

    #[cfg(debug_assertions)]
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    color_backtrace::install();
}
