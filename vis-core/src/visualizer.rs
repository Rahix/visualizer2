use crate::analyzer;
use crate::recorder;

/// Builder for a Visualizer
///
/// The "core" of `vis-core`.  Take a look at the crate root for an example on
/// how to use it.
#[derive(Debug)]
pub struct Visualizer<R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    /// Initial value of the data buffer shared between *analyzer* and *recorder*.
    ///
    /// This type **must** be `Clone`.
    pub initial: R,
    /// Analyzer closure
    pub analyzer: A,
    /// Trait-Object for the recorder
    ///
    /// By default, [`recorder::default`](../recorder/fn.default.html) is called, which will consult the config
    /// (`"audio.recorder"`) or use pulse.
    pub recorder: Option<Box<dyn recorder::Recorder>>,
    /// Whether the analyzer should run asynchroneously and if so, how many times per second.
    ///
    /// Can also be set from config as `"audio.conversions"`.
    pub async_analyzer: Option<usize>,
}

impl<R, A> Visualizer<R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    /// Create a new visualizer
    ///
    /// You need to supply an initial value for the shared data and the analyzer closure.
    pub fn new(initial: R, analyzer: A) -> Visualizer<R, A> {
        Visualizer {
            initial,
            analyzer,
            recorder: None,
            async_analyzer: None,
        }
    }

    /// Specify the recorder to be used.
    ///
    /// By default, [`recorder::default`](../recorder/fn.default.html) is called, which will consult the config
    /// (`"audio.recorder"`) or use pulse.
    pub fn recorder(mut self, r: Box<dyn recorder::Recorder>) -> Visualizer<R, A> {
        self.recorder = Some(r);
        self
    }

    /// Make the analyzer run in a separate thread.
    ///
    /// `conversions_per_second` specifies how often the analyzer should be run (at max).
    pub fn async_analyzer(mut self, conversions_per_second: usize) -> Visualizer<R, A> {
        self.async_analyzer = Some(conversions_per_second);
        self
    }

    /// Create a frames iterator from this visualizer config
    ///
    /// The frames iterator should then be iterated over in you main loop:
    ///
    /// ```
    /// # vis_core::default_config();
    /// # let mut frames = vis_core::Visualizer::new(0.0, |i, _s| i)
    /// #     .frames();
    /// 'main: for frame in frames.iter() {
    ///     println!("Time: {}", frame.time);
    ///
    ///     if frame.time > 0.3 {
    ///         break 'main;
    ///     }
    /// }
    /// ```
    pub fn frames(self) -> crate::Frames<R, A> {
        crate::Frames::from_vis(self)
    }
}
