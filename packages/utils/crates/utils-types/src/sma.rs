use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div};

/// Simple Moving Average trait.
/// The second-most useful filter type, bested only by coffee filters.
pub trait SMA<T>
where
    T: for<'a> Add<&'a T, Output = T> + Div<T, Output = T> + Clone + From<u32> + Display + Default,
{
    /// Adds a sample.
    fn add_sample(&mut self, sample: T);

    /// Calculates the moving average value.
    fn get_average(&self) -> T;

    /// Returns the window size.
    fn window_size(&self) -> usize;

    /// Returns the number of elements in the window.
    /// This value is always between 0 and `window_size()`.
    fn len(&self) -> usize;
}

/// Basic implementation of Simple Moving Average (SMA).
/// The maximum window size is bound by 2^32 - 1.
/// Useful mainly for floating-point types, as it does not accumulate floating point error with each sample,
/// but requires `O(N)` of memory and `O(N)` for average computation, `N` being window size.
#[derive(Clone, Debug, PartialEq)]
pub struct NoSumSMA<T>
where
    T: Clone,
{
    window: AllocRingBuffer<T>,
}

impl<T> SMA<T> for NoSumSMA<T>
where
    T: Add<T, Output = T> + Div<T, Output = T> + Clone + From<u32>,
{
    fn add_sample(&mut self, sample: T) {
        self.window.push(sample);
    }

    fn get_average(&self) -> Option<T> {
        if !self.window.is_empty() {
            let mut ret = self.window[0].clone();
            for v in self.window.iter().skip(1).cloned() {
                ret = ret + v;
            }
            Some(ret / (self.window.len() as u32).into())
        } else {
            None
        }
    }

    fn window_size(&self) -> usize {
        self.window.capacity()
    }

    fn len(&self) -> usize {
        self.window.len()
    }
}

impl<T> SMA<T> for NoSumSMA<T>
    where
        T: Add<T, Output = T> + Div<u32, Output = T> + Clone,
{
    fn add_sample(&mut self, sample: T) {
        self.window.push(sample);
    }

    fn get_average(&self) -> Option<T> {
        if !self.window.is_empty() {
            let mut ret = self.window[0].clone();
            for v in self.window.iter().skip(1).cloned() {
                ret = ret + v;
            }
            Some(ret / (self.window.len() as u32))
        } else {
            None
        }
    }

    fn window_size(&self) -> usize {
        self.window.capacity()
    }

    fn len(&self) -> usize {
        self.window.len()
    }
}

impl<T> NoSumSMA<T>
where
    T: Clone,
{
    /// Creates an empty SMA instance with the given window size.
    /// The maximum window size is u32::MAX and must be greater than 1.
    pub fn new(window_size: u32) -> Self {
        assert!(window_size > 1, "window size must be greater than 1");
        Self {
            window: AllocRingBuffer::new(window_size as usize),
        }
    }

    /// Creates SMA instance given window size and some initial samples.
    pub fn new_with_samples(window_size: u32, initial_samples: &[T]) -> Self {
        let mut ret = Self::new(window_size);
        initial_samples.iter().cloned().for_each(|s| ret.add_sample(s));
        ret
    }
}

impl<T> Display for NoSumSMA<T>
where
    T: Display + Default
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_average().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use crate::sma::{NoSumSMA, SMA};

    #[test]
    fn test_nosum_sma_empty() {
        let sma = NoSumSMA::<u32>::new(3);
        assert_eq!(3, sma.window_size(), "invalid windows size");
        assert_eq!(0, sma.get_average(), "invalid empty average");
    }

    #[test]
    fn test_nosum_sma_should_calculate_avg_correctly() {
        let mut sma = NoSumSMA::<u32>::new(3);
        sma.add_sample(0);
        sma.add_sample(1);
        sma.add_sample(2);
        sma.add_sample(3);

        assert_eq!(2, sma.get_average(), "invalid average");
        assert_eq!(3, sma.window_size(), "window size is invalid");
    }
}
