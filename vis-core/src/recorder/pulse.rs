use crate::analyzer;
use std::thread;

#[derive(Debug, Default)]
pub struct PulseBuilder {
    pub rate: Option<usize>,
    pub read_size: Option<usize>,
    pub buffer_size: Option<usize>,
    pub name: Option<(String, String)>,
    pub device: Option<String>,
}

impl PulseBuilder {
    pub fn new() -> PulseBuilder {
        Default::default()
    }

    pub fn rate(&mut self, rate: usize) -> &mut PulseBuilder {
        self.rate = Some(rate);
        self
    }

    pub fn read_size(&mut self, size: usize) -> &mut PulseBuilder {
        self.read_size = Some(size);
        self
    }

    pub fn buffer_size(&mut self, size: usize) -> &mut PulseBuilder {
        self.buffer_size = Some(size);
        self
    }

    pub fn name<S1, S2>(&mut self, name: S1, desc: S2) -> &mut PulseBuilder
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.name = Some((name.into(), desc.into()));
        self
    }

    pub fn device<S: Into<String>>(&mut self, dev: S) -> &mut PulseBuilder {
        self.device = Some(dev.into());
        self
    }

    pub fn create(&self) -> PulseRecorder {
        PulseRecorder::from_builder(self)
    }

    pub fn build(&self) -> Box<dyn super::Recorder> {
        Box::new(self.create())
    }
}

#[derive(Debug)]
pub struct PulseRecorder {
    rate: usize,
    buffer: analyzer::SampleBuffer,
}

impl PulseRecorder {
    fn from_builder(build: &PulseBuilder) -> PulseRecorder {
        let rate = build
            .rate
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.rate", 8000));
        let buffer_size = build
            .buffer_size
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.buffer", 16000));
        let read_size = build
            .buffer_size
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.read_size", 32));
        let (name, desc) = build.name.clone().unwrap_or((
            "visualizer2".to_string(),
            "Pulseaudio recorder for visualizer2".to_string(),
        ));
        let device = build
            .device
            .clone()
            .or_else(|| crate::CONFIG.get("pulse.device"));

        let buf = analyzer::SampleBuffer::new(buffer_size, rate);

        {
            let buf = buf.clone();

            thread::Builder::new()
                .name("pulse-recorder".into())
                .spawn(move || {
                    let rec: pulse_simple::Record<[analyzer::Sample; 2]> =
                        pulse_simple::Record::new(
                            &name,
                            &desc,
                            device.as_ref().map(|s| s.as_str()),
                            rate as u32,
                        );

                    let mut read_buf = vec![[0.0; 2]; read_size];

                    log::debug!("Pulseaudio:");
                    log::debug!("    Sample Rate = {:6}", rate);
                    log::debug!("    Read Size   = {:6}", read_size);
                    log::debug!("    Buffer Size = {:6}", buffer_size);

                    loop {
                        rec.read(&mut read_buf);

                        buf.push(&read_buf);
                        log::trace!("Pushed {} samples", read_size);
                    }
                })
                .unwrap();
        }

        PulseRecorder { rate, buffer: buf }
    }
}

impl super::Recorder for PulseRecorder {
    fn sample_buffer<'a>(&'a self) -> &'a analyzer::SampleBuffer {
        &self.buffer
    }
}
