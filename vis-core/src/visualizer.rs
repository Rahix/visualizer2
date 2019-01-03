use crate::analyzer;
use crate::recorder;

#[derive(Debug)]
pub struct Visualizer<R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    pub initial: R,
    pub analyzer: A,
    pub recorder: Option<Box<dyn recorder::Recorder>>,
    pub async_analyzer: Option<usize>,
}

impl<R, A> Visualizer<R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    pub fn new(initial: R, analyzer: A) -> Visualizer<R, A> {
        Visualizer {
            initial,
            analyzer,
            recorder: None,
            async_analyzer: None,
        }
    }

    pub fn recorder(mut self, r: Box<dyn recorder::Recorder>) -> Visualizer<R, A> {
        self.recorder = Some(r);
        self
    }

    pub fn async_analyzer(mut self, conversions_per_second: usize) -> Visualizer<R, A> {
        self.async_analyzer = Some(conversions_per_second);
        self
    }

    pub fn frames(self) -> crate::Frames<R, A> {
        crate::Frames::from_vis(self)
    }
}
