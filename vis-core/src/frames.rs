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
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    info: sync::Arc<sync::Mutex<R>>,
    analyzer: A,
    recorder: Box<dyn recorder::Recorder>,
}

impl<R, A> Frames<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    pub fn from_vis(vis: crate::Visualizer<R, A>) -> Frames<R, A> {
        Frames {
            info: sync::Arc::new(sync::Mutex::new(vis.initial)),
            analyzer: vis.analyzer,
            recorder: vis.recorder.unwrap_or_else(|| recorder::default()),
        }
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
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    visualizer: &'a mut Frames<R, A>,
    buffer: analyzer::SampleBuffer,
    start_time: time::Instant,
    frame: usize,
}

impl<'a, R, A> Iterator for FramesIter<'a, R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    type Item = Frame<R>;

    fn next(&mut self) -> Option<Self::Item> {
        {
            let mut lock = self.visualizer.info.lock().unwrap();
            *lock = (self.visualizer.analyzer)(&self.buffer);
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
