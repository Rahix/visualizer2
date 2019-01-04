//! Beat Detection
use crate::analyzer;

#[derive(Debug, Default)]
pub struct BeatBuilder {
    pub decay: Option<analyzer::SignalStrength>,
    pub trigger: Option<analyzer::SignalStrength>,
    pub range: Option<(analyzer::Frequency, analyzer::Frequency)>,
    pub fourier_length: Option<usize>,
    pub downsample: Option<usize>,
}

impl BeatBuilder {
    pub fn new() -> BeatBuilder {
        Default::default()
    }

    pub fn decay(&mut self, decay: analyzer::SignalStrength) -> &mut BeatBuilder {
        self.decay = Some(decay);
        self
    }

    pub fn trigger(&mut self, trigger: analyzer::SignalStrength) -> &mut BeatBuilder {
        self.trigger = Some(trigger);
        self
    }

    pub fn range(
        &mut self,
        low: analyzer::Frequency,
        high: analyzer::Frequency,
    ) -> &mut BeatBuilder {
        self.range = Some((low, high));
        self
    }

    pub fn fourier_length(&mut self, length: usize) -> &mut BeatBuilder {
        self.fourier_length = Some(length);
        self
    }

    pub fn downsample(&mut self, downsample: usize) -> &mut BeatBuilder {
        self.downsample = Some(downsample);
        self
    }

    pub fn build(&mut self) -> BeatDetector {
        BeatDetector::from_builder(self)
    }
}

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
    pub fn from_builder(build: &BeatBuilder) -> BeatDetector {
        BeatDetector {
            decay: 1.0
                - 1.0
                    / build
                        .decay
                        .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.decay", 1000.0)),
            trigger: build
                .trigger
                .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.trigger", 0.5)),
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

            analyzer: analyzer::FourierBuilder::new()
                .window(analyzer::window::nuttall)
                .length(
                    build
                        .fourier_length
                        .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.fourier_length", 16)),
                )
                .downsample(
                    build
                        .downsample
                        .unwrap_or_else(|| crate::CONFIG.get_or("audio.beat.downsample", 10)),
                )
                .plan(),
        }
    }

    pub fn last_volume(&self) -> analyzer::SignalStrength {
        self.last_volume
    }

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
