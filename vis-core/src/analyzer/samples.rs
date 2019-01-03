//! Sample Buffer
use std::collections;
use std::sync;

pub type Sample = f32;

type _SampleBuf = sync::Arc<parking_lot::Mutex<collections::VecDeque<[Sample; 2]>>>;

#[derive(Debug, Clone)]
pub struct SampleBuffer {
    buf: _SampleBuf,
    pub rate: usize,
}

impl SampleBuffer {
    pub fn new(size: usize, rate: usize) -> SampleBuffer {
        let buf = collections::VecDeque::from(vec![[0.0; 2]; size]);

        SampleBuffer {
            buf: sync::Arc::new(parking_lot::Mutex::new(buf)),
            rate,
        }
    }

    pub fn push(&self, new: &[[Sample; 2]]) {
        let mut lock = self.buf.lock();

        #[cfg(debug_assertions)]
        let debug_size = lock.len();

        for sample in new.iter() {
            lock.pop_front().expect("Failed to pop sample!");
            lock.push_back(*sample);
        }

        #[cfg(debug_assertions)]
        assert_eq!(debug_size, lock.len(), "Sample buffer size differs!");
    }

    pub fn iter<'a>(&'a self, size: usize, downsample: usize) -> SampleIterator<'a> {
        let lock = self.buf.lock();

        SampleIterator {
            index: lock.len() - (size * downsample),
            buf: lock,
            downsample,
        }
    }

    pub fn volume(&self, length: f32) -> super::SignalStrength {
        use super::SignalStrength;

        let lock = self.buf.lock();
        let len = lock.len();

        let div = (1.0 / length) as usize;

        (lock
            .iter()
            // Only look at the last tenth of a second
            .skip(len - self.rate / div)
            // RMS
            .map(|s| ((s[0] + s[1]) / 2.0).powi(2) as SignalStrength)
            .sum::<SignalStrength>()
            / len as SignalStrength)
            .sqrt()
    }
}

pub struct SampleIterator<'a> {
    buf: parking_lot::MutexGuard<'a, collections::VecDeque<[Sample; 2]>>,
    index: usize,
    downsample: usize,
}

impl Iterator for SampleIterator<'_> {
    type Item = [f32; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let res = self.buf.get(self.index).cloned();
        self.index += self.downsample;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let buf = SampleBuffer::new(16, 8000);

        buf.push(&[[1.0; 2]; 8]);

        for s in buf.iter(16, 1) {
            println!("{:?}", s);
        }
    }

    #[test]
    fn test_overflow() {
        let buf = SampleBuffer::new(16, 8000);

        buf.push(
            &(100..120)
                .map(|i| [i as Sample, i as Sample])
                .collect::<Vec<_>>(),
        );

        buf.push(
            &(0..32)
                .map(|i| [i as Sample, i as Sample])
                .collect::<Vec<_>>(),
        );

        assert_eq!(
            buf.iter(16, 1).collect::<Vec<_>>(),
            (16..32)
                .map(|i| [i as Sample, i as Sample])
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn test_downsample() {
        let buf = SampleBuffer::new(32, 8000);

        buf.push(
            &(0..32)
                .map(|i| [i as Sample, i as Sample])
                .collect::<Vec<_>>(),
        );

        assert_eq!(
            &buf.iter(7, 4).collect::<Vec<_>>(),
            &[[4.0; 2], [8.0; 2], [12.0; 2], [16.0; 2], [20.0; 2], [24.0; 2], [28.0; 2],]
        );
    }
}
