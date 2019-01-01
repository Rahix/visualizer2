# vis-core

**vis-core** is the core of *visualizer2*.  It contains the thread orchestration
and a few analyzation tools.  These include:

* [Fourier Spectralizer](src/analyzer/fourier.rs)
* [Beat Detector](src/analyzer/beat.rs)

## Audio Input
In *vis-core* audio input happens using the [recorder](src/recorder/mod.rs).  You
can implement recorders yourself if the one you need does not exist yet.  The interface
is dead simple:

```rust
pub trait Recorder: std::fmt::Debug {
    /// Return the sample buffer where this recorder pushes data into
    fn sample_buffer<'a>(&'a self) -> &'a analyzer::SampleBuffer;

    /// Synchronize sample buffer for this time stamp
    ///
    /// Returns true as long as new samples are available
    ///
    /// Async recorders (eg. pulse) will always return true
    /// and ignore this call otherwise
    fn sync(&mut self, time: f32) -> bool;
}
```

As an example, take a look at the [pulseaudio recorder](src/recorder/pulse.rs).
