use crate::analyzer;
use crate::recorder;

#[derive(Debug)]
pub struct Visualizer<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    pub analyzer: Option<A>,
    pub recorder: Option<Box<dyn recorder::Recorder>>,
}

impl<R, A> Default for Visualizer<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    fn default() -> Visualizer<R, A> {
        Visualizer {
            analyzer: None,
            recorder: None,
        }
    }
}

impl<R, A> Visualizer<R, A>
where
    A: FnMut(&analyzer::SampleBuffer) -> R,
{
    pub fn new() -> Visualizer<R, A> {
        Default::default()
    }

    pub fn analyzer(mut self, a: A) -> Visualizer<R, A> {
        self.analyzer = Some(a);
        self
    }

    pub fn recorder(mut self, r: Box<dyn recorder::Recorder>) -> Visualizer<R, A> {
        self.recorder = Some(r);
        self
    }

    pub fn frames(self) -> crate::Frames<R, A> {
        crate::Frames {
            analyzer: self.analyzer.expect("No analyzer given"),
            recorder: self.recorder.unwrap_or_else(|| recorder::default()),
        }
    }
}
