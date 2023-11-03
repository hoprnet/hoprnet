use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div};

/// Basic implementation of Simple Moving Average (SMA).
/// The maximum window size is bound by 2^32 - 1.
/// The second-most useful filter type, bested only by coffee filters.
#[derive(Clone, Debug, PartialEq)]
pub struct SMA<T>
where
    T: for<'a> Add<&'a T, Output = T> + Div<T, Output = T> + Clone + From<u32> + Display + Default,
{
    window: AllocRingBuffer<T>,
    samples_added: usize,
}

impl<T> SMA<T>
where
    T: for<'a> Add<&'a T, Output = T> + Div<T, Output = T> + Clone + From<u32> + Display + Default,
{
    /// Creates an empty SMA instance with the given window size.
    /// The maximum window size is u32::MAX and must be greater than 1.
    pub fn new(window_size: u32) -> Self {
        assert!(window_size > 1, "window size must be greater than 1");
        Self {
            window: AllocRingBuffer::new(window_size as usize),
            samples_added: 0,
        }
    }

    /// Creates SMA instance given window size and some initial samples.
    pub fn new_with_samples(window_size: u32, initial_samples: &[T]) -> Self {
        let mut ret = Self::new(window_size);
        initial_samples.iter().for_each(|s| ret.add_sample(s.clone()));
        ret
    }

    /// Adds a sample.
    pub fn add_sample(&mut self, sample: T) {
        self.window.push(sample);
        self.samples_added += 1;
    }

    /// Returns the total number of samples added, regardless the window size.
    pub fn num_samples_added(&self) -> usize {
        self.samples_added
    }

    /// Returns the window size.
    pub fn window_size(&self) -> usize {
        self.window.capacity()
    }

    /// Calculates the moving average value.
    pub fn get_average(&self) -> T {
        if !self.window.is_empty() {
            let mut ret = T::default();
            for v in self.window.iter() {
                ret = ret + v;
            }
            ret / (self.window.len() as u32).into()
        } else {
            Default::default()
        }
    }
}

impl<T> Display for SMA<T>
where
    T: for<'a> Add<&'a T, Output = T> + Div<T, Output = T> + Clone + From<u32> + Display + Default,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_average())
    }
}

#[cfg(test)]
mod tests {
    use crate::sma::SMA;

    #[test]
    fn test_sma_empty() {
        let sma = SMA::<u32>::new(3);
        assert_eq!(3, sma.window_size(), "invalid windows size");
        assert_eq!(0, sma.num_samples_added(), "invalid number of samples");
        assert_eq!(0, sma.get_average(), "invalid empty average");
    }

    #[test]
    fn test_sma_should_calculate_avg_correctly() {
        let mut sma = SMA::<u32>::new(3);
        sma.add_sample(0);
        sma.add_sample(1);
        sma.add_sample(2);
        sma.add_sample(3);

        assert_eq!(4, sma.num_samples_added(), "number of samples added must be correct");
        assert_eq!(2, sma.get_average(), "invalid average");
        assert_eq!(3, sma.window_size(), "window size is invalid");
    }
}
