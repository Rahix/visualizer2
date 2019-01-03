pub mod analyzer;
pub mod frames;
pub mod helpers;
pub mod recorder;
pub mod visualizer;

pub use crate::frames::Frames;
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
}
