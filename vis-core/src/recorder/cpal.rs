use crate::analyzer;
use std::thread;

#[derive(Debug, Default)]
pub struct CPalBuilder {
    pub rate: Option<usize>,
    pub buffer_size: Option<usize>,
    pub read_size: Option<usize>,
}

impl CPalBuilder {
    pub fn new() -> CPalBuilder {
        Default::default()
    }

    pub fn rate(&mut self, rate: usize) -> &mut CPalBuilder {
        self.rate = Some(rate);
        self
    }

    pub fn buffer_size(&mut self, buffer_size: usize) -> &mut CPalBuilder {
        self.buffer_size = Some(buffer_size);
        self
    }

    pub fn read_size(&mut self, read_size: usize) -> &mut CPalBuilder {
        self.read_size = Some(read_size);
        self
    }

    pub fn create(&self) -> CPalRecorder {
        CPalRecorder::from_builder(self)
    }

    pub fn build(&self) -> Box<dyn super::Recorder> {
        Box::new(self.create())
    }
}

#[derive(Debug)]
pub struct CPalRecorder {
    rate: usize,
    buffer: analyzer::SampleBuffer,
}

impl CPalRecorder {
    fn from_builder(build: &CPalBuilder) -> CPalRecorder {
        let rate = build
            .rate
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.rate", 8000));
        let buffer_size = build
            .buffer_size
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.buffer", 16000));
        let read_size = build
            .buffer_size
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.read_size", 256));

        let buf = analyzer::SampleBuffer::new(buffer_size, rate);

        {
            let buf = buf.clone();
            let mut chunk_buffer = vec![[0.0; 2]; read_size];

            thread::Builder::new()
                .name("cpal-recorder".into())
                .spawn(move || {
                    let device = cpal::default_input_device().expect("Can't acquire input device");

                    let format = cpal::Format {
                        channels: 2,
                        sample_rate: cpal::SampleRate(rate as u32),
                        data_type: cpal::SampleFormat::F32,
                    };

                    let event_loop = cpal::EventLoop::new();
                    let stream_id = event_loop
                        .build_input_stream(&device, &format)
                        .expect("Failed to build input stream");
                    event_loop.play_stream(stream_id);

                    log::debug!("CPal:");
                    log::debug!("    Sample Rate = {:6}", rate);
                    log::debug!("    Read Size   = {:6}", read_size);
                    log::debug!("    Buffer Size = {:6}", buffer_size);
                    log::debug!("    Device      = \"{}\"", device.name());

                    event_loop.run(|_, data| match data {
                        cpal::StreamData::Input {
                            buffer: cpal::UnknownTypeInputBuffer::F32(buffer),
                        } => {
                            for chunk in buffer.chunks(chunk_buffer.len() * 2) {
                                let len = chunk.len() / 2;

                                for ref mut p in chunk_buffer.iter_mut().zip(chunk.chunks_exact(2))
                                {
                                    match p {
                                        (ref mut b, [l, r]) => **b = [*l, *r],
                                        _ => unreachable!(),
                                    }
                                }

                                buf.push(&chunk_buffer[..len]);
                            }
                        }
                        cpal::StreamData::Input { .. } => {
                            panic!("Buffer type does not match configuration!");
                        }
                        cpal::StreamData::Output { .. } => (),
                    });
                })
                .unwrap();
        }

        CPalRecorder { rate, buffer: buf }
    }
}

impl super::Recorder for CPalRecorder {
    fn sample_buffer<'a>(&'a self) -> &'a analyzer::SampleBuffer {
        &self.buffer
    }
}
