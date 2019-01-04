//! Beat Detection
use crate::analyzer;

/// Builder for BeatDetector
///
/// Your configuration needs to be a tradeoff between quality of beat-detection
/// and latency.
///
/// The latency is roughly: `(fourier_length * downsample) / rate` seconds
#[derive(Debug, Default)]
pub struct BeatBuilder {
    /// Decay of the beat volume
    ///
    /// The lower this is, the faster a more silent beat will be detected.
    /// Defaults to `2000.0`.  Can also be set from config as `"audio.beat.decay"`.
    pub decay: Option<analyzer::SignalStrength>,

    /// The minimum volume a beat must have, relative to the previous one, to be deteced.
    ///
    /// Defaults to `0.4`.  Can also be set from config as `"audio.beat.trigger"`.
    pub trigger: Option<analyzer::SignalStrength>,

    /// Frequency range to search for beats in.
    ///
    /// Defaults to `50 Hz - 100 Hz`, can also be set from config as `"audio.beat.low"`
    /// and `"audio.beat.high"`
    pub range: Option<(analyzer::Frequency, analyzer::Frequency)>,

    /// Length of the fourier transform for beat detection.
    ///
    /// Keep this as short as possible to minimize delay!  Defaults to 16, can also
    /// be set from config as `"audio.beat.fourier_length"`.
    pub fourier_length: Option<usize>,

    /// Downsampling factor
    ///
    /// Defaults to 10 and can be set from config as `"audio.beat.downsample"`.
    pub downsample: Option<usize>,

    /// Recording rate
    ///
    ///
    /// Defaults to `8000` or `"audio.rate"`.
    pub rate: Option<usize>,
}

impl BeatBuilder {
    /// Create new BeatBuilder
    pub fn new() -> BeatBuilder {
        Default::default()
    }

    /// Set decay
    pub fn decay(&mut self, decay: analyzer::SignalStrength) -> &mut BeatBuilder {
        self.decay = Some(decay);
        self
    }

    /// Set trigger
    pub fn trigger(&mut self, trigger: analyzer::SignalStrength) -> &mut BeatBuilder {
        self.trigger = Some(trigger);
        self
    }

    /// Set frequency range
    pub fn range(
        &mut self,
        low: analyzer::Frequency,
        high: analyzer::Frequency,
    ) -> &mut BeatBuilder {
        self.range = Some((low, high));
        self
    }

    /// Set fourier length
    pub fn fourier_length(&mut self, length: usize) -> &mut BeatBuilder {
        self.fourier_length = Some(length);
        self
    }

    /// Set downsampling factor
    pub fn downsample(&mut self, downsample: usize) -> &mut BeatBuilder {
        self.downsample = Some(downsample);
        self
    }

    /// Set recording rate
    pub fn rate(&mut self, rate: usize) -> &mut BeatBuilder {
        self.rate = Some(rate);
        self
    }

    /// Build the detector
    pub fn build(&mut self) -> BeatDetector {
        BeatDetector::from_builder(self)
    }
}

/// A beat detector
///
/// # Example
/// ```
/// # use vis_core::analyzer;
/// # let samples = analyzer::SampleBuffer::new(32000, 8000);
/// let mut beat = analyzer::BeatBuilder::new()
///     .decay(2000.0)
///     .trigger(0.4)
///     .range(50.0, 100.0)
///     .fourier_length(16)
///     .downsample(10)
///     .rate(8000)
///     .build();
///
/// let isbeat = beat.detect(&samples);
/// ```
pub struct BeatDetector {
    decay: analyzer::SignalStrength,
    trigger: analyzer::SignalStrength,
    range: (analyzer::Frequency, analyzer::Frequency),

    last_volume: analyzer::SignalStrength,
    last_delta: analyzer::SignalStrength,
    last_beat_delta: analyzer::SignalStrength,

    last_peak: analyzer::SignalStrength,
    last_valley: analyzer::SignalStrength,

    analyzer: analyzer::FourierAnalyzer,
}

impl BeatDetector {
    /// Create a BeatDetector from a builder config
    pub fn from_builder(build: &BeatBuilder) -> BeatDetector {
        BeatDetector {
            decay: 1.0
                - 1.0
                    / build
                        .decay
                        .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.decay", 2000.0)),
            trigger: build
                .trigger
                .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.trigger", 0.4)),
            range: build.range.unwrap_or_else(|| {
                (
                    crate::CONFIG.get_or("audio.beat.low", 50.0),
                    crate::CONFIG.get_or("audio.beat.high", 100.0),
                )
            }),

            last_volume: 0.0,
            last_delta: 0.0,
            last_beat_delta: 0.0,

            last_peak: 0.0,
            last_valley: 0.0,

            analyzer: analyzer::FourierBuilder {
                window: Some(analyzer::window::nuttall),
                length: Some(
                    build
                        .fourier_length
                        .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.fourier_length", 16)),
                ),
                downsample: Some(
                    build
                        .downsample
                        .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.downsample", 10)),
                ),
                rate: Some(
                    build
                        .rate
                        .unwrap_or_else(|| crate::CONFIG.get_or("audio.rate", 8000)),
                ),
            }
            .plan(),
        }
    }

    /// Get the volume measured during the last detection cycle
    pub fn last_volume(&self) -> analyzer::SignalStrength {
        self.last_volume
    }

    /// Detect a beat
    ///
    /// Returns true if this cycle is a beat and false otherwise.
    pub fn detect(&mut self, samples: &analyzer::SampleBuffer) -> bool {
        self.analyzer.analyze(samples);
        let volume = self
            .analyzer
            .average()
            .slice(self.range.0, self.range.1)
            .mean();

        // Decay beat_delta to allow quieter beats to be detected
        self.last_beat_delta = self.last_beat_delta * self.decay;
        let delta = volume - self.last_volume;

        let isbeat = if delta < 0.0 && self.last_delta > 0.0 {
            self.last_peak = self.last_volume;
            let beat_delta = self.last_peak - self.last_valley;

            // Check if the peak is big enough
            if beat_delta > (self.last_beat_delta * self.trigger) {
                self.last_beat_delta = self.last_beat_delta.max(beat_delta);
                true
            } else {
                false
            }
        } else if delta > 0.0 && self.last_delta < 0.0 {
            self.last_valley = self.last_volume;
            false
        } else {
            false
        };

        self.last_volume = volume;
        // Only write delta if the last two volumes weren't the same
        if delta != 0.0 {
            self.last_delta = delta;
        }

        isbeat
    }
}
