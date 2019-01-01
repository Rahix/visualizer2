use super::Sample;
use crate::analyzer;

pub mod window {
    pub fn blackman(size: usize) -> Vec<f32> {
        apodize::blackman_iter(size).map(|f| f as f32).collect()
    }

    pub fn hamming(size: usize) -> Vec<f32> {
        apodize::hamming_iter(size).map(|f| f as f32).collect()
    }

    pub fn hanning(size: usize) -> Vec<f32> {
        apodize::hanning_iter(size).map(|f| f as f32).collect()
    }

    pub fn none(size: usize) -> Vec<f32> {
        vec![1.0; size]
    }

    pub fn nuttall(size: usize) -> Vec<f32> {
        apodize::nuttall_iter(size).map(|f| f as f32).collect()
    }

    pub fn sine(size: usize) -> Vec<f32> {
        (0..size)
            .map(|i| (i as f32 / (size - 1) as f32 * std::f32::consts::PI).sin())
            .collect()
    }

    pub fn triangular(size: usize) -> Vec<f32> {
        apodize::triangular_iter(size).map(|f| f as f32).collect()
    }

    pub fn from_str(name: &str) -> Option<fn(usize) -> Vec<f32>> {
        match name {
            "blackman" => Some(blackman),
            "hamming" => Some(hamming),
            "hanning" => Some(hanning),
            "none" => Some(none),
            "nuttall" => Some(nuttall),
            "sine" => Some(sine),
            "triangular" => Some(triangular),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct FourierBuilder {
    pub length: Option<usize>,
    pub window: Option<fn(usize) -> Vec<f32>>,
    pub downsample: Option<usize>,
    pub rate: Option<usize>,
}

impl FourierBuilder {
    pub fn new() -> FourierBuilder {
        Default::default()
    }

    pub fn length(&mut self, length: usize) -> &mut FourierBuilder {
        self.length = Some(length);
        self
    }

    pub fn window(&mut self, f: fn(usize) -> Vec<f32>) -> &mut FourierBuilder {
        self.window = Some(f);
        self
    }

    pub fn downsample(&mut self, factor: usize) -> &mut FourierBuilder {
        self.downsample = Some(factor);
        self
    }

    pub fn rate(&mut self, rate: usize) -> &mut FourierBuilder {
        self.rate = Some(rate);
        self
    }

    pub fn plan(&mut self) -> FourierAnalyzer {
        let length = self
            .length
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.fourier_length", 512));
        let window = (self.window.unwrap_or_else(|| {
            window::from_str(&crate::CONFIG.get_or("audio.window", "none".to_string()))
                .expect("Selected window type not found!")
        }))(length);
        let downsample = self
            .downsample
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.downsample", 5));
        let rate = self
            .rate
            .unwrap_or_else(|| crate::CONFIG.get_or("audio.rate", 8000));

        FourierAnalyzer::new(length, window, downsample, rate)
    }
}

#[derive(Clone)]
pub struct FourierAnalyzer {
    length: usize,
    pub buckets: usize,
    window: Vec<Sample>,
    pub downsample: usize,

    rate: usize,
    pub lowest: analyzer::Frequency,
    pub hightest: analyzer::Frequency,

    fft: std::sync::Arc<rustfft::FFT<Sample>>,

    input: [Vec<rustfft::num_complex::Complex<Sample>>; 2],
    output: Vec<rustfft::num_complex::Complex<Sample>>,

    spectra: [analyzer::Spectrum<Vec<analyzer::SignalStrength>>; 2],
    average: analyzer::Spectrum<Vec<analyzer::SignalStrength>>,
}

impl std::fmt::Debug for FourierAnalyzer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "FourierAnalyzer {{ length: {:?}, downsample: {:?}, lowest: {:?}, highest: {:?} }}",
            self.length,
            self.downsample,
            self.lowest,
            self.hightest,
        )
    }
}

impl FourierAnalyzer {
    fn new(length: usize, window: Vec<f32>, downsample: usize, rate: usize) -> FourierAnalyzer {
        use rustfft::num_traits::Zero;

        let fft = rustfft::FFTplanner::new(false).plan_fft(length);
        let buckets = length / 2;

        let downsampled_rate = rate as f32 / downsample as f32;
        let lowest = downsampled_rate / length as f32;
        let hightest = downsampled_rate / 2.0;

        let fa = FourierAnalyzer {
            length,
            buckets,
            window,
            downsample,

            rate,
            lowest,
            hightest,

            fft,

            input: [Vec::with_capacity(length), Vec::with_capacity(length)],
            output: vec![rustfft::num_complex::Complex::zero(); length],

            spectra: [
                analyzer::Spectrum::new(vec![0.0; buckets], lowest, hightest),
                analyzer::Spectrum::new(vec![0.0; buckets], lowest, hightest),
            ],
            average: analyzer::Spectrum::new(vec![0.0; buckets], lowest, hightest),
        };

        log::debug!("FourierAnalyzer({:p}):", &fa);
        log::debug!("    Fourier Length      = {:8}", length);
        log::debug!("    Buckets             = {:8}", buckets);
        log::debug!(
            "    Downsampled Rate    = {:8} ({} / {})",
            downsampled_rate,
            rate,
            downsample,
        );
        log::debug!("    Lowest  Frequency   = {:8.3} Hz", lowest);
        log::debug!("    Highest Frequency   = {:8.3} Hz", hightest);

        fa
    }

    pub fn analyze(
        &mut self,
        buf: &analyzer::SampleBuffer,
    ) -> [analyzer::Spectrum<&[analyzer::SignalStrength]>; 2] {
        log::trace!("FourierAnalyzer({:p}): Analyzing ...", &self);

        assert_eq!(buf.rate, self.rate, "Samplerate of buffer does not match!");

        // Copy samples to left and right buffer
        self.input[0].clear();
        self.input[1].clear();
        for ([l, r], window) in buf
            .iter(self.length, self.downsample)
            .zip(self.window.iter())
        {
            self.input[0].push(rustfft::num_complex::Complex::new(l * window, 0.0));
            self.input[1].push(rustfft::num_complex::Complex::new(r * window, 0.0));
        }

        debug_assert_eq!(self.input[0].len(), self.window.len());
        debug_assert_eq!(self.input[1].len(), self.window.len());

        self.fft.process(&mut self.input[0], &mut self.output);
        for (s, o) in self.spectra[0].iter_mut().zip(self.output.iter()) {
            *s = o.norm_sqr();
        }

        self.fft.process(&mut self.input[1], &mut self.output);
        for (s, o) in self.spectra[1].iter_mut().zip(self.output.iter()) {
            *s = o.norm_sqr();
        }

        [self.spectra[0].as_ref(), self.spectra[1].as_ref()]
    }

    pub fn left(&self) -> analyzer::Spectrum<&[analyzer::SignalStrength]> {
        self.spectra[0].as_ref()
    }

    pub fn right(&self) -> analyzer::Spectrum<&[analyzer::SignalStrength]> {
        self.spectra[1].as_ref()
    }

    pub fn average(&mut self) -> analyzer::Spectrum<&[analyzer::SignalStrength]> {
        analyzer::average_spectrum(&mut self.average, &self.spectra);

        self.average.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let analyzer = FourierBuilder::new()
            .length(512)
            .window(window::from_str("nuttall").unwrap())
            .downsample(8)
            .plan();
    }

    #[test]
    fn test_analyze() {
        let mut analyzer = FourierBuilder::new()
            .length(512)
            .window(window::from_str("nuttall").unwrap())
            .downsample(2)
            .plan();

        let buf = crate::analyzer::SampleBuffer::new(1024, 8000);

        buf.push(&[[1.0; 2]; 1024]);

        let mut out_l = crate::analyzer::Spectrum::new(vec![0.0; 256], 0.0, 1.0);
        let mut out_r = crate::analyzer::Spectrum::new(vec![0.0; 256], 0.0, 1.0);

        analyzer.analyze(&buf, &mut [out_l, out_r]);
    }
}
