//! Spectrum Storage Type

/// Type Alias for Frequencies
pub type Frequency = f32;

/// Type Alias for Signal Strengths
pub type SignalStrength = f32;

/// Trait for types that can be used as storage for a spectrum
pub trait Storage: std::ops::Deref<Target = [SignalStrength]> {}

/// Trait for types that can be used as mutable storage for a spectrum
pub trait StorageMut: std::ops::Deref<Target = [SignalStrength]> + std::ops::DerefMut {}

impl<T> Storage for T where T: std::ops::Deref<Target = [SignalStrength]> {}

impl<T> StorageMut for T where T: Storage + std::ops::DerefMut {}

#[derive(Debug, Clone)]
pub struct Spectrum<S: Storage> {
    buckets: S,
    width: Frequency,
    lowest: Frequency,
    highest: Frequency,
}

impl<S: Storage> std::ops::Index<usize> for Spectrum<S> {
    type Output = SignalStrength;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buckets[index]
    }
}

impl<S: Storage> std::ops::Index<Frequency> for Spectrum<S> {
    type Output = SignalStrength;

    fn index(&self, index: Frequency) -> &Self::Output {
        &self.buckets[self.freq_to_id(index)]
    }
}

impl<S: StorageMut> std::ops::IndexMut<usize> for Spectrum<S> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buckets[index]
    }
}

impl<S: StorageMut> std::ops::IndexMut<Frequency> for Spectrum<S> {
    fn index_mut(&mut self, index: Frequency) -> &mut Self::Output {
        let idx = self.freq_to_id(index);
        &mut self.buckets[idx]
    }
}

impl Default for Spectrum<Vec<SignalStrength>> {
    fn default() -> Self {
        Spectrum {
            buckets: vec![0.0],
            width: 1.0,
            lowest: 0.0,
            highest: 0.0,
        }
    }
}

impl<S: Storage> Spectrum<S> {
    /// Create a new spectrum
    ///
    /// Takes a storage buffer which is potentially prefilled with spectral data,
    /// the frequency associated with the lowest bucket and the frequency associated
    /// with the highest bucket.
    ///
    /// # Example
    /// ```
    /// # use vis_core::analyzer;
    /// const N: usize = 128;
    /// let spectrum = analyzer::Spectrum::new(vec![0.0; N], 440.0, 660.0);
    /// ```
    pub fn new(data: S, low: Frequency, high: Frequency) -> Spectrum<S> {
        Spectrum {
            width: (high - low) / (data.len() as Frequency - 1.0),
            lowest: low,
            highest: high,

            buckets: data,
        }
    }

    /// Return the frequency of the lowest bucket
    #[inline]
    pub fn lowest(&self) -> Frequency {
        self.lowest
    }

    /// Return the frequency of the highest bucket
    #[inline]
    pub fn highest(&self) -> Frequency {
        self.highest
    }

    /// Respan this spectrum.  Use with care!
    fn respan(&mut self, low: Frequency, high: Frequency) {
        self.width = (high - low) / (self.buckets.len() as Frequency - 1.0);
        self.lowest = low;
        self.highest = high;
    }

    /// Return the index of the bucket associated with a frequency
    pub fn freq_to_id(&self, f: Frequency) -> usize {
        let x = (f - self.lowest) / self.width;

        assert!(x >= 0.0);
        let i = x.round() as usize;
        assert!(i < self.buckets.len());
        i
    }

    /// Return the frequency associated with a bucket
    pub fn id_to_freq(&self, i: usize) -> Frequency {
        assert!(i < self.buckets.len());

        i as Frequency * self.width + self.lowest
    }

    /// Iterate over the buckets of this spectrum
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, SignalStrength> {
        self.buckets.iter()
    }

    /// Return the number of buckets in this spectrum
    pub fn len(&self) -> usize {
        self.buckets.len()
    }

    pub fn as_ref<'a>(&'a self) -> Spectrum<&'a [SignalStrength]> {
        Spectrum {
            buckets: &self.buckets,
            width: self.width,
            lowest: self.lowest,
            highest: self.highest,
        }
    }

    /// Return the highest signal strengh in this spectrum
    pub fn max(&self) -> SignalStrength {
        *self
            .buckets
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

    /// Return the average signal strengh in this spectrum
    pub fn mean(&self) -> SignalStrength {
        self.buckets.iter().sum::<SignalStrength>() / self.len() as f32
    }

    /// Return a spectrum with the buckets between the specified frequencies
    ///
    /// Requires **no** allocation!  Please note that the returned spectrum might be slightly
    /// off if the specified frequencies are not exactly in the middle of two buckets.
    ///
    /// # Example
    /// ```
    /// # use vis_core::analyzer;
    /// let spectrum = analyzer::Spectrum::new(vec![0.0; 400], 220.0, 660.0);
    /// let sliced = spectrum.slice(220.0, 440.0);
    /// # assert_eq!(sliced.len(), 201);
    /// ```
    pub fn slice<'a>(&'a self, low: Frequency, high: Frequency) -> Spectrum<&'a [SignalStrength]> {
        let start = self.freq_to_id(low);
        let end = self.freq_to_id(high);

        Spectrum {
            buckets: &self.buckets[start..end + 1],
            width: self.width,
            lowest: self.lowest + start as Frequency * self.width,
            highest: self.lowest + (end) as Frequency * self.width,
        }
    }

    /// Allocate a buffer and fill it with data from this spectrum
    ///
    /// Will merge adjacent buckets to fit data into the new buffer.
    pub fn fill_buckets_alloc(&self, n: usize) -> Spectrum<Vec<f32>> {
        self.fill_buckets(vec![0.0; n])
    }

    /// Fill a given buffer with data from this spectrum
    ///
    /// Will merge adjacent buckets to fit data into the new buffer.
    ///
    /// # Example
    /// ```
    /// # use vis_core::analyzer;
    /// let spectrum = analyzer::Spectrum::new(vec![0.0; 400], 220.0, 660.0);
    /// let downscaled = spectrum.fill_buckets(vec![0.0; 20]);
    /// # assert_eq!(downscaled.len(), 20);
    /// ```
    pub fn fill_buckets<S2: StorageMut>(&self, mut buf: S2) -> Spectrum<S2> {
        for i in 0..buf.len() {
            buf[i] = 0.0;
        }

        for (i, v) in self.buckets.iter().enumerate() {
            let bucket = i * buf.len() / self.buckets.len();
            buf[bucket] += v;
        }

        Spectrum {
            width: (self.highest - self.lowest) / (buf.len() as f32 - 1.0),
            lowest: self.lowest,
            highest: self.highest,

            buckets: buf,
        }
    }

    pub fn fill_spectrum<'a, S2: StorageMut>(
        &self,
        other: &'a mut Spectrum<S2>,
    ) -> &'a mut Spectrum<S2> {
        self.fill_buckets(&mut *other.buckets);

        other.respan(self.lowest, self.highest);

        other
    }

    /// Find all maxima in this spectrum and allocate a buffer containing them
    pub fn find_maxima_alloc(&self) -> Vec<(f32, f32)> {
        let derivative = self
            .buckets
            .windows(2)
            .map(|v| v[1] - v[0])
            .collect::<Vec<_>>();

        let mut maxima = derivative
            .windows(2)
            .enumerate()
            .filter_map(|(i, d)| {
                if d[0] > 0.0 && d[1] < 0.0 {
                    Some((self.id_to_freq(i + 1), self.buckets[i + 1]))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        maxima.sort_by(|(_, a1), (_, a2)| a2.partial_cmp(a1).unwrap());

        maxima
    }

    /// Find maxima in this spectrum and fill `buffer` with them
    ///
    /// Please note that this method will behave incorrectly if more than `buffer.len()` maxima
    /// exist.  Maxima are sorted, starting with the biggest.  Returns the number of maxima found.
    ///
    /// # Example
    /// ```
    /// # use vis_core::analyzer;
    /// let mut spectrum = analyzer::Spectrum::new(vec![0.0; 400], 220.0, 660.0);
    ///
    /// // Manually create maxima
    /// spectrum[100] = 10.0;
    /// spectrum[200] = 20.0;
    /// spectrum[300] = 15.0;
    ///
    /// let mut buf = [(0.0, 0.0); 5];
    /// let num = spectrum.find_maxima(&mut buf);
    ///
    /// assert_eq!(num, 3);
    /// assert_eq!(
    ///     &buf[..num],
    ///     &[
    ///         (spectrum.id_to_freq(200), 20.0),
    ///         (spectrum.id_to_freq(300), 15.0),
    ///         (spectrum.id_to_freq(100), 10.0),
    ///     ],
    /// );
    /// ```
    pub fn find_maxima(&self, buffer: &mut [(f32, f32)]) -> usize {
        let derivative = self
            .buckets
            .windows(2)
            .map(|v| v[1] - v[0])
            .collect::<Vec<_>>();

        let derive2 = derivative.clone();
        let maxima_iter = derive2
            .iter()
            .zip(derivative.iter().skip(1))
            .enumerate()
            .filter_map(|(i, (d0, d1))| {
                if d0 > &0.0 && d1 < &0.0 {
                    Some((self.id_to_freq(i + 1), self.buckets[i + 1]))
                } else {
                    None
                }
            });

        let mut num = 0;
        for (b, m) in buffer.iter_mut().zip(maxima_iter) {
            *b = m;
            num += 1;
        }

        buffer[0..num].sort_by(|(_, a1), (_, a2)| a2.partial_cmp(a1).unwrap());

        num
    }
}

impl<S: StorageMut> Spectrum<S> {
    /// Iterate over this spectrums buckets mutably
    pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, SignalStrength> {
        self.buckets.iter_mut()
    }

    /// Fill this spectrum with values from another one
    pub fn fill_from<S2: Storage>(&mut self, other: &Spectrum<S2>) {
        assert_eq!(self.len(), other.len(), "Spectrums have different sizes!");

        self.width = other.width;
        self.lowest = other.lowest;
        self.highest = other.highest;

        for (s, o) in self.iter_mut().zip(other.iter()) {
            *s = *o;
        }
    }
}

/// Compute the average of multiple spectra
pub fn average_spectrum<'a, S: Storage, SMut: StorageMut>(
    out: &'a mut Spectrum<SMut>,
    spectra: &[Spectrum<S>],
) -> &'a Spectrum<SMut> {
    let buffer = &mut out.buckets;

    let num = spectra.len() as SignalStrength;
    debug_assert!(num > 0.0);

    let buckets = buffer.len();
    let lowest = spectra[0].lowest;
    let highest = spectra[0].highest;

    // Clear output
    for b in buffer.iter_mut() {
        *b = 0.0;
    }

    for s in spectra.iter() {
        debug_assert_eq!(s.len(), buckets);
        debug_assert_eq!(s.lowest, lowest);
        debug_assert_eq!(s.highest, highest);

        for (b, x) in buffer.iter_mut().zip(s.buckets.iter()) {
            *b += x;
        }
    }

    for b in buffer.iter_mut() {
        *b /= num;
    }

    out.respan(lowest, highest);

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_integrity<S: Storage>(s: &Spectrum<S>) {
        assert_eq!(
            ((s.highest - s.lowest) / s.width).round() as usize,
            s.buckets.len() - 1
        );
    }

    #[test]
    fn test_default() {
        let def: Spectrum<_> = Default::default();

        check_integrity(&def);
    }

    fn do_tests<F: FnMut(usize, f32, f32, f32, f32, Spectrum<Vec<f32>>)>(mut f: F) {
        for n in [100, 1000, 512, 1337].iter().cloned() {
            for (l, h, low, high) in [
                (100.0, 200.0, 125.0, 175.0),
                (50.0, 8000.0, 100.0, 200.0),
                (600.0, 750.0, 700.0, 750.0),
                (1.0, 20000.0, 1000.0, 2000.0),
                (1.0, 20000.0, 50.0, 100.0),
                (1.0, 20000.0, 2.0, 19999.0),
                (0.0, 10.0, 2.0, 5.0),
            ]
            .iter()
            .cloned()
            {
                println!("Parameters: N: {:5}, Range: {:7.2}-{:7.2}", n, l, h);
                let spectrum = Spectrum::new((0..n).map(|x| x as f32).collect::<Vec<_>>(), l, h);
                check_integrity(&spectrum);

                f(n, l, h, low, high, spectrum)
            }
        }
    }

    #[test]
    fn test_iter() {
        do_tests(|_, _, _, _, _, spectrum| {
            let bucket_list = spectrum.iter().cloned().collect::<Vec<f32>>();

            assert_eq!(bucket_list, &*spectrum.buckets);
        })
    }

    #[test]
    fn test_maxima_alloc() {
        do_tests(|n, _, _, _, _, mut spectrum| {
            let m1 = n / 2 + 25;
            let m2 = n / 5;

            spectrum[m1 - 1] = 500000.0;
            spectrum[m1] = 1000000.0;
            spectrum[m1 + 1] = 500000.0;

            spectrum[m2 - 1] = 350000.0;
            spectrum[m2] = 400000.0;
            spectrum[m2 + 1] = 350000.0;

            let maxima = spectrum.find_maxima_alloc();

            assert_eq!(
                maxima,
                &[
                    (spectrum.id_to_freq(m1), 1000000.0),
                    (spectrum.id_to_freq(m2), 400000.0),
                ]
            );
        })
    }

    #[test]
    fn test_maxima() {
        do_tests(|n, _, _, _, _, mut spectrum| {
            let m1 = n / 2 + 25;
            let m2 = n / 5;

            spectrum[m1 - 1] = 500000.0;
            spectrum[m1] = 1000000.0;
            spectrum[m1 + 1] = 500000.0;

            spectrum[m2 - 1] = 350000.0;
            spectrum[m2] = 400000.0;
            spectrum[m2 + 1] = 350000.0;

            let mut maxima = [(0.0, 0.0); 10];
            let n = spectrum.find_maxima(&mut maxima);

            assert_eq!(
                &maxima[..n],
                &[
                    (spectrum.id_to_freq(m1), 1000000.0),
                    (spectrum.id_to_freq(m2), 400000.0),
                ]
            );
        })
    }

    #[test]
    fn test_conversion() {
        do_tests(|n, _, _, _, _, spectrum| {
            for i in 0..n {
                assert_eq!(i, spectrum.freq_to_id(spectrum.id_to_freq(i)));
            }
        })
    }

    #[test]
    fn test_freq_index() {
        do_tests(|n, _, _, _, _, spectrum| {
            for i in 0..n {
                assert_eq!(
                    spectrum[i as f32 * spectrum.width + spectrum.lowest],
                    i as f32,
                );
            }
        })
    }

    #[test]
    fn test_consistency() {
        do_tests(|n, l, h, _, _, spectrum| {
            println!("- Sanity check");
            assert_eq!(spectrum.lowest, l);
            assert_eq!(spectrum.highest, h);

            println!("- `low` should be 0");
            assert_eq!(spectrum.freq_to_id(l), 0);

            println!("- `high` should be last");
            assert_eq!(spectrum.freq_to_id(h), n - 1);
        })
    }

    #[test]
    fn test_slice() {
        do_tests(|_, _, _, low, high, spectrum| {
            let sliced = spectrum.slice(low, high);
            check_integrity(&sliced);

            println!("- Size should stay the same");
            assert_eq!(sliced.width, spectrum.width);

            println!("- Low frequency right?");
            assert!(
                (sliced.lowest - low).abs() < spectrum.width,
                "{} < {}",
                (sliced.lowest - low).abs(),
                spectrum.width
            );

            println!("- High frequency right?");
            assert!(
                (sliced.highest - high).abs() < spectrum.width,
                "{} < {}",
                (sliced.highest - high).abs(),
                spectrum.width
            );
        })
    }

    #[test]
    fn test_fill() {
        let mut buf = Some(vec![50.0; 20]);
        do_tests(|_, _, _, _, _, spectrum| {
            let buckets = spectrum.fill_buckets(buf.take().unwrap());
            check_integrity(&buckets);

            let spec_sum = spectrum.iter().sum::<f32>();
            let bucket_sum = buckets.iter().sum::<f32>();
            assert_eq!(spec_sum, bucket_sum);

            buf = Some(buckets.buckets);
        })
    }
}
