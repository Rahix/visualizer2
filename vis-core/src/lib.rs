pub mod analyzer;
pub mod frames;
pub mod helpers;
pub mod recorder;
pub mod visualizer;

pub use crate::frames::Frames;
pub use crate::visualizer::Visualizer;

pub static CONFIG: ezconf::Config = ezconf::INIT;

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

pub fn default_log() {
    #[cfg(not(debug_assertions))]
    env_logger::init();

    #[cfg(debug_assertions)]
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
}
