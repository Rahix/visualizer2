pub type Frequency = f32;
pub type SignalStrength = f32;

pub trait Storage: std::ops::Deref<Target = [SignalStrength]> {}
pub trait StorageMut: std::ops::Deref<Target = [SignalStrength]> + std::ops::DerefMut {}

impl<T> Storage for T where T: std::ops::Deref<Target = [SignalStrength]> {}

impl<T> StorageMut for T where T: Storage + std::ops::DerefMut {}

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

impl<S: Storage> Spectrum<S> {
    pub fn new(data: S, low: Frequency, high: Frequency) -> Spectrum<S> {
        Spectrum {
            width: (high - low) / (data.len() as Frequency - 1.0),
            lowest: low,
            highest: high,

            buckets: data,
        }
    }

    pub fn freq_to_id(&self, f: Frequency) -> usize {
        let x = (f - self.lowest) / self.width;

        assert!(x >= 0.0);
        let i = x.round() as usize;
        assert!(i < self.buckets.len());
        i
    }

    pub fn id_to_freq(&self, i: usize) -> Frequency {
        assert!(i < self.buckets.len());

        i as Frequency * self.width + self.lowest
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, SignalStrength> {
        self.buckets.iter()
    }

    pub fn len(&self) -> usize {
        self.buckets.len()
    }

    pub fn max(&self) -> SignalStrength {
        *self
            .buckets
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }

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

    pub fn fill_buckets_alloc(&self, n: usize) -> Spectrum<Vec<f32>> {
        self.fill_buckets(vec![0.0; n])
    }

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
        for (mut b, m) in buffer.iter_mut().zip(maxima_iter) {
            *b = m;
            num += 1;
        }

        buffer[0..num].sort_by(|(_, a1), (_, a2)| a2.partial_cmp(a1).unwrap());

        num
    }
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
