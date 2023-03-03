use crate::{analyzer, recorder};
use std::{cell, rc, time};

/// Data for one Frame
#[derive(Debug)]
pub struct Frame<R: Send> {
    /// Timestamp since start
    pub time: f32,

    /// Frame number
    pub frame: usize,

    info: rc::Rc<cell::RefCell<triple_buffer::Output<R>>>,
}

impl<R: Send> Frame<R> {
    /// Get access to the latest info shared from the analyzer
    ///
    /// # Example
    /// ```
    /// # vis_core::default_config();
    /// # let mut frames = vis_core::Visualizer::new(0.0, |i, _s| i)
    /// #     .frames();
    /// for frame in frames.iter() {
    ///     println!("Time: {}", frame.time);
    ///
    ///     frame.info(|info|
    ///         println!("Info: {:?}", info)
    ///     );
    /// #
    /// #     if frame.time > 0.3 {
    /// #         break;
    /// #     }
    /// }
    /// ```
    pub fn info<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&R) -> O,
    {
        f(self.info.borrow_mut().read())
    }
}

/// Frames Iterator
#[derive(Debug)]
pub struct Frames<R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    info: rc::Rc<cell::RefCell<triple_buffer::Output<R>>>,
    analyzer: Option<(A, triple_buffer::Input<R>)>,
    recorder: Box<dyn recorder::Recorder>,
}

impl<R, A> Frames<R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    pub fn from_vis(vis: crate::Visualizer<R, A>) -> Frames<R, A> {
        let (inp, outp) = triple_buffer::TripleBuffer::new(&vis.initial).split();
        let mut f = Frames {
            info: rc::Rc::new(cell::RefCell::new(outp)),
            analyzer: Some((vis.analyzer, inp)),
            recorder: vis
                .recorder
                .unwrap_or_else(|| recorder::RecorderBuilder::new().build()),
        };

        if let Some(num) = vis.async_analyzer {
            if num != 0 {
                f.detach_analyzer(num);
            }
        } else {
            if let Some(num) = crate::CONFIG.get("audio.conversions") {
                f.detach_analyzer(num);
            }
        }

        f
    }

    /// Move analyzer to a separate thread
    pub fn detach_analyzer(&mut self, num: usize) {
        let (mut analyzer, mut info) = self.analyzer.take().unwrap();
        let buffer = self.recorder.sample_buffer().clone();

        let conv_time = std::time::Duration::new(0, (1000000000 / num) as u32);
        log::debug!("Conversion Time: {:?}", conv_time);

        std::thread::Builder::new()
            .name("analyzer".into())
            .spawn(move || loop {
                let start = std::time::Instant::now();
                analyzer(info.input_buffer(), &buffer);
                info.publish();

                let now = std::time::Instant::now();
                let duration = now - start;
                log::trace!("Conversion Time (real): {:?}", duration);

                if duration < conv_time {
                    let sleep = conv_time - duration;
                    log::trace!("Sleeping for {:?}", sleep);
                    std::thread::sleep(sleep);
                }
            })
            .unwrap();
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

/// Borrowed Frames Iterator
#[derive(Debug)]
pub struct FramesIter<'a, R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    visualizer: &'a mut Frames<R, A>,
    buffer: analyzer::SampleBuffer,
    start_time: time::Instant,
    frame: usize,
}

impl<'a, R, A> Iterator for FramesIter<'a, R, A>
where
    R: Clone + Send + 'static,
    for<'r> A: FnMut(&'r mut R, &analyzer::SampleBuffer) -> &'r mut R + Send + 'static,
{
    type Item = Frame<R>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((ref mut analyzer, ref mut info)) = self.visualizer.analyzer {
            analyzer(info.input_buffer(), &self.buffer);
            info.publish();
        }

        let frame = self.frame;
        self.frame += 1;

        Some(Frame {
            time: crate::helpers::time(self.start_time),
            frame,
            info: self.visualizer.info.clone(),
        })
    }
}
