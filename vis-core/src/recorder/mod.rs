#[cfg(feature = "pulseaudio")]
pub mod pulse;

#[cfg(feature = "cpalrecord")]
pub mod cpal;

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
    fn sync(&mut self, _time: f32) -> bool {
        true
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecorderBuilder {
    pub rate: Option<usize>,
    pub buffer_size: Option<usize>,
    pub read_size: Option<usize>,
    pub recorder: Option<String>,
}

impl RecorderBuilder {
    pub fn new() -> RecorderBuilder {
        Default::default()
    }

    pub fn rate(&mut self, rate: usize) -> &mut RecorderBuilder {
        self.rate = Some(rate);
        self
    }

    pub fn buffer_size(&mut self, buffer_size: usize) -> &mut RecorderBuilder {
        self.buffer_size = Some(buffer_size);
        self
    }

    pub fn read_size(&mut self, read_size: usize) -> &mut RecorderBuilder {
        self.read_size = Some(read_size);
        self
    }

    pub fn recorder<S: Into<String>>(&mut self, rec: S) -> &mut RecorderBuilder {
        self.recorder = Some(rec.into());
        self
    }

    pub fn build(&mut self) -> Box<dyn Recorder> {
        let rate = self
            .rate
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.rate", 8000));
        let buffer_size = self
            .buffer_size
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.buffer", 16000));
        let read_size = self
            .read_size
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.read_size", 32));
        let recorder = self
            .recorder
            .as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.recorder", "cpal".to_string()));

        match &*recorder {
            #[cfg(feature = "cpalrecord")]
            "cpal" => self::cpal::CPalBuilder {
                rate: Some(rate),
                buffer_size: Some(buffer_size),
                read_size: Some(read_size),
                ..Default::default()
            }
            .build(),

            #[cfg(feature = "pulseaudio")]
            "pulse" => self::cpal::PulseBuilder {
                rate: Some(rate),
                buffer_size: Some(buffer_size),
                read_size: Some(read_size),
                ..Default::default()
            }
            .build(),

            _ => {
                panic!("Recorder type does not exist!");
            }
        }
    }
}
