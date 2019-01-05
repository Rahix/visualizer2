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

    pub fn create(&mut self) -> CPalRecorder {
        CPalRecorder::new(8000, 16000)
    }

    pub fn build(&mut self) -> Box<dyn super::Recorder> {
        Box::new(self.create())
    }
}

#[derive(Debug)]
pub struct CPalRecorder {
    rate: usize,
    buffer: analyzer::SampleBuffer,
}

impl CPalRecorder {
    fn new(rate: usize, buffer_size: usize) -> CPalRecorder {
        let buf = analyzer::SampleBuffer::new(buffer_size, rate);

        {
            let buf = buf.clone();
            let mut chunk_buffer = vec![[0.0; 2]; 256];

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
                                        _ => panic!("Slice has wrong length!"),
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
