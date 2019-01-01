pub mod beat;
pub mod fourier;
pub mod samples;
pub mod spectrum;

pub use self::beat::{BeatBuilder, BeatDetector};
pub use self::fourier::{window, FourierAnalyzer, FourierBuilder};
pub use self::samples::{Sample, SampleBuffer};
pub use self::spectrum::{average_spectrum, Frequency, SignalStrength, Spectrum};
