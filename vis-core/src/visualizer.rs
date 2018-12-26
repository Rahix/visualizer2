use crate::analyzer;
use crate::recorder;

#[derive(Debug)]
pub struct Visualizer<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    pub initial: R,
    pub analyzer: A,
    pub recorder: Option<Box<dyn recorder::Recorder>>,
}

impl<R, A> Visualizer<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    pub fn new(initial: R, analyzer: A) -> Visualizer<R, A> {
        Visualizer {
            initial,
            analyzer,
            recorder: None,
        }
    }

    pub fn recorder(mut self, r: Box<dyn recorder::Recorder>) -> Visualizer<R, A> {
        self.recorder = Some(r);
        self
    }

    pub fn frames(self) -> crate::Frames<R, A> {
        crate::Frames::from_vis(self)
    }
}
