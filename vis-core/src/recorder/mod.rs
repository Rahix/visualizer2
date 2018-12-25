#[cfg(feature = "pulseaudio")]
pub mod pulse;

use crate::analyzer;

pub trait Recorder: std::fmt::Debug {
    /// Return the sample buffer where this recorder pushes data into
    fn sample_buffer<'a>(&'a self) -> &'a analyzer::SampleBuffer;

    /// Synchronize sample buffer for this time stamp
    ///
    /// Returns true as long as new samples are available
    ///
    /// Async recorders (eg. pulse) will always return true
    /// and ignore this call otherwise
    fn sync(&mut self, time: f32) -> bool;
}

pub fn from_str(name: &str) -> Option<Box<dyn Recorder>> {
    match name {
        "pulse" => Some(Box::new(pulse::PulseBuilder::new().build())),
        _ => None,
    }
}

pub fn default() -> Box<dyn Recorder> {
    from_str(crate::CONFIG.get_or("audio.recorder", "pulse")).expect("Recorder not found")
}
