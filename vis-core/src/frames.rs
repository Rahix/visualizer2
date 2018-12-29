use crate::{analyzer, recorder};
use std::sync;
use std::time;

#[derive(Debug)]
pub struct Frame<R> {
    pub time: f32,
    pub frame: usize,
    info: sync::Arc<sync::Mutex<R>>,
}

impl<R> Frame<R> {
    pub fn lock_info<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&mut R) -> O,
    {
        f(&mut self.info.lock().expect("Can't lock audio info"))
    }
}

#[derive(Debug)]
pub struct Frames<R, A>
where
    R: Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    info: sync::Arc<sync::Mutex<R>>,
    analyzer: Option<A>,
    recorder: Box<dyn recorder::Recorder>,
}

impl<R, A> Frames<R, A>
where
    R: Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    pub fn from_vis(vis: crate::Visualizer<R, A>) -> Frames<R, A> {
        let mut f = Frames {
            info: sync::Arc::new(sync::Mutex::new(vis.initial)),
            analyzer: Some(vis.analyzer),
            recorder: vis.recorder.unwrap_or_else(|| recorder::default()),
        };

        if vis.async_analyzer.unwrap_or(false) {
            f.detach_analyzer();
        }

        f
    }

    pub fn detach_analyzer(&mut self) {
        let mut analyzer = self.analyzer.take().unwrap();
        let info = self.info.clone();
        let buffer = self.recorder.sample_buffer().clone();

        std::thread::Builder::new()
            .name("analyzer".into())
            .spawn(move || {
                loop {
                    analyzer(&mut info.lock().unwrap(), &buffer);
                    // Todo, properly implement this detacher
                    std::thread::sleep_ms(1);
                }
            }).unwrap();
    }

    pub fn iter<'a>(&'a mut self) -> FramesIter<'a, R, A> {
        FramesIter {
            buffer: self.recorder.sample_buffer().clone(),
            visualizer: self,
            start_time: time::Instant::now(),
            frame: 0,
        }
    }
}

#[derive(Debug)]
pub struct FramesIter<'a, R, A>
where
    R: Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    visualizer: &'a mut Frames<R, A>,
    buffer: analyzer::SampleBuffer,
    start_time: time::Instant,
    frame: usize,
}

impl<'a, R, A> Iterator for FramesIter<'a, R, A>
where
    R: Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    type Item = Frame<R>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ref mut analyzer) = self.visualizer.analyzer {
            analyzer(&mut self.visualizer.info.lock().unwrap(), &self.buffer);
        }

        let f = Frame {
            time: crate::helpers::time(self.start_time),
            frame: self.frame,
            info: self.visualizer.info.clone(),
        };

        self.frame += 1;

        Some(f)
    }
}
