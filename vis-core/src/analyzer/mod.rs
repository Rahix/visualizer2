pub mod beat;
pub mod fourier;
pub mod samples;
pub mod spectrum;

#[doc(inline)]
pub use self::beat::{BeatBuilder, BeatDetector};
#[doc(inline)]
pub use self::fourier::{window, FourierAnalyzer, FourierBuilder};
#[doc(inline)]
pub use self::samples::{Sample, SampleBuffer};
#[doc(inline)]
pub use self::spectrum::{average_spectrum, Frequency, SignalStrength, Spectrum};
