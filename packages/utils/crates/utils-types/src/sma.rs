use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Add, Div};

/// Simple Moving Average trait.
/// The second-most useful filter type, bested only by coffee filters.
pub trait SMA<T> {
    /// Adds a sample.
    fn add_sample(&mut self, sample: T);

    /// Calculates the moving average value.
    /// Returns `None` if no samples were added.
    fn get_average(&self) -> Option<T>;

    /// Returns the window size.
    fn window_size(&self) -> usize;

    /// Returns the number of elements in the window.
    /// This value is always between 0 and `window_size()`.
    fn len(&self) -> usize;

    /// Indicates whether the window is fully occupied
    /// with samples.
    fn is_window_full(&self) -> bool {
        self.len() == self.window_size()
    }

    /// Indicates whether there are no samples.
    fn is_empty(&self) -> bool;
}

/// Basic implementation of Simple Moving Average (SMA).
/// The maximum window size is bound by 2^32 - 1.
/// Useful mainly for floating-point types, as it does not accumulate floating point error with each sample,
/// but requires `O(N)` of memory and `O(N)` for average computation, `N` being window size.
/// The divisor argument `D` is used only for such types `T` that do not implement `From<u32>` (such as `Duration`,...).
#[derive(Clone, Debug, PartialEq)]
pub struct NoSumSMA<T, D = T>
where
    T: Clone,
{
    window: AllocRingBuffer<T>,
    _div: PhantomData<D>,
}

impl<T, D> SMA<T> for NoSumSMA<T, D>
where
    T: Add<T, Output = T> + Div<D, Output = T> + Clone,
    D: From<u32>,
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
            Some(ret / D::from(self.window.len() as u32))
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

    fn is_empty(&self) -> bool {
        self.window.is_empty()
    }
}

impl<T, D> NoSumSMA<T, D>
where
    T: Add<T, Output = T> + Div<D, Output = T> + Clone,
    D: From<u32>,
{
    /// Creates an empty SMA instance with the given window size.
    /// The maximum window size is u32::MAX and must be greater than 1.
    pub fn new(window_size: u32) -> Self {
        assert!(window_size > 1, "window size must be greater than 1");
        Self {
            window: AllocRingBuffer::new(window_size as usize),
            _div: PhantomData,
        }
    }

    /// Creates SMA instance given window size and some initial samples.
    pub fn new_with_samples(window_size: u32, initial_samples: &[T]) -> Self {
        let mut ret = Self::new(window_size);
        initial_samples.iter().cloned().for_each(|s| ret.add_sample(s));
        ret
    }
}

impl<T, D> Display for NoSumSMA<T, D>
where
    T: Add<T, Output = T> + Div<D, Output = T> + Clone + Default + Display,
    D: From<u32>,
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
        let sma = NoSumSMA::<u32, u32>::new(3);
        assert_eq!(3, sma.window_size(), "invalid windows size");
        assert!(sma.get_average().is_none(), "invalid empty average");
    }

    #[test]
    fn test_nosum_sma_should_calculate_avg_correctly() {
        let mut sma = NoSumSMA::<u32, u32>::new(3);
        sma.add_sample(0);
        sma.add_sample(1);
        sma.add_sample(2);
        sma.add_sample(3);

        assert_eq!(2, sma.get_average().unwrap(), "invalid average");
        assert_eq!(3, sma.window_size(), "window size is invalid");
    }
}
