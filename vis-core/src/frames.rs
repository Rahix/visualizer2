use crate::{analyzer, recorder};
use std::time;

#[derive(Debug)]
pub struct Frame<R> {
    pub time: f32,
    pub frame: usize,
    pub info: R,
}

#[derive(Debug)]
pub struct Frames<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    pub(crate) analyzer: A,
    pub(crate) recorder: Box<dyn recorder::Recorder>,
}

impl<R, A> Frames<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
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
        let res = (self.visualizer.analyzer)(&self.buffer);

        let f = Frame {
            time: crate::helpers::time(self.start_time),
            frame: self.frame,
            info: res,
        };

        self.frame += 1;

        Some(f)
    }
}
