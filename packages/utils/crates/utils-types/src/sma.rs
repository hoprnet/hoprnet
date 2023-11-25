use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::marker::PhantomData;
use std::ops::{AddAssign, Div, SubAssign};

/// Simple Moving Average trait.
/// The second-most useful filter type, bested only by coffee filters.
pub trait SMA<T> {
    /// Pushes a sample.
    fn push(&mut self, sample: T);

    /// Calculates the moving average value.
    /// Returns `None` if no samples were added.
    fn average(&self) -> Option<T>;

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
/// Useful mainly for floating-point types, as it does not accumulate floating point error with each sample.
/// Requires `O(N)` of memory and `O(N)` for average computation, `N` being window size.
/// The divisor argument `D` is used only for such types `T` that do not implement `From<u32>` (such as `Duration`,...).
#[derive(Clone, Debug, PartialEq)]
pub struct NoSumSMA<T, D = T> {
    window: AllocRingBuffer<T>,
    _div: PhantomData<D>,
}

impl<T, D> SMA<T> for NoSumSMA<T, D>
where
    T: for<'a> Sum<&'a T> + Div<D, Output = T>,
    D: From<u32>,
{
    fn push(&mut self, sample: T) {
        self.window.push(sample);
    }

    fn average(&self) -> Option<T> {
        if !self.is_empty() {
            Some(self.window.iter().sum::<T>() / D::from(self.window.len() as u32))
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
    T: for <'a> Sum<&'a T> + Div<D, Output = T>,
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
    pub fn new_with_samples(window_size: u32, initial_samples: Vec<T>) -> Self {
        let mut ret = Self::new(window_size);
        initial_samples.into_iter().for_each(|s| ret.push(s));
        ret
    }
}

impl<T, D> Display for NoSumSMA<T, D>
where
    T: for <'a> Sum<&'a T> + Div<D, Output = T> + Default + Display,
    D: From<u32>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.average().unwrap_or_default())
    }
}

/// Basic implementation of Simple Moving Average (SMA).
/// The maximum window size is bound by 2^32 - 1.
/// Useful mainly for integer types, as it does accumulate floating point error with each sample.
/// Requires `O(N)` of memory and `O(1)` for average computation, `N` being window size.
/// The divisor argument `D` is used only for such types `T` that do not implement `From<u32>` (such as `Duration`,...).
#[derive(Clone, Debug, PartialEq)]
pub struct SingleSumSMA<T, D = T> {
    window: AllocRingBuffer<T>,
    sum: T,
    _div: PhantomData<D>,
}

impl<T, D> SMA<T> for SingleSumSMA<T, D>
where
    T: AddAssign + SubAssign + Div<D, Output = T> + Copy,
    D: From<u32>,
{
    fn push(&mut self, sample: T) {
        self.sum += sample;

        if self.is_window_full() {
            if let Some(shifted_sample) = self.window.dequeue() {
                self.sum -= shifted_sample;
            }
        }

        self.window.enqueue(sample);
    }

    fn average(&self) -> Option<T> {
        if !self.is_empty() {
            Some(self.sum / D::from(self.window.len() as u32))
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

impl<T, D> SingleSumSMA<T, D>
where
    T: AddAssign + SubAssign + Div<D, Output = T> + Copy + Default,
    D: From<u32>,
{
    /// Creates an empty SMA instance with the given window size.
    /// The maximum window size is u32::MAX and must be greater than 1.
    pub fn new(window_size: u32) -> Self {
        assert!(window_size > 1, "window size must be greater than 1");
        Self {
            window: AllocRingBuffer::new(window_size as usize),
            sum: T::default(),
            _div: PhantomData,
        }
    }

    /// Creates SMA instance given window size and some initial samples.
    pub fn new_with_samples(window_size: u32, initial_samples: Vec<T>) -> Self {
        let mut ret = Self::new(window_size);
        initial_samples.into_iter().for_each(|s| ret.push(s));
        ret
    }
}

impl<T, D> Display for SingleSumSMA<T, D>
where
    T: AddAssign + SubAssign + Div<D, Output = T> + Copy + Display + Default,
    D: From<u32>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.average().unwrap_or_default())
    }
}


#[cfg(test)]
mod tests {
    use crate::sma::{NoSumSMA, SingleSumSMA, SMA};

    #[test]
    fn test_nosum_sma_empty() {
        let sma = NoSumSMA::<u32, u32>::new(3);
        assert_eq!(3, sma.window_size(), "invalid windows size");
        assert!(sma.average().is_none(), "invalid empty average");

        assert!(sma.is_empty(), "should be empty");
        assert_eq!(0, sma.len(), "len is invalid");
    }

    #[test]
    fn test_nosum_sma_should_calculate_avg_correctly() {
        let mut sma = NoSumSMA::<u32, u32>::new(3);
        sma.push(0);
        sma.push(1);
        sma.push(2);
        sma.push(3);

        assert_eq!(Some(2), sma.average(), "invalid average");
        assert_eq!(3, sma.window_size(), "window size is invalid");

        assert!(!sma.is_empty(), "should not be empty");
        assert_eq!(3, sma.len(), "len is invalid");
    }

    #[test]
    fn test_single_sum_sma_empty() {
        let sma = SingleSumSMA::<u32, u32>::new(3);
        assert_eq!(3, sma.window_size(), "invalid windows size");
        assert!(sma.average().is_none(), "invalid empty average");

        assert!(sma.is_empty(), "should be empty");
        assert_eq!(0, sma.len(), "len is invalid");
    }

    #[test]
    fn test_single_sum_sma_should_calculate_avg_correctly() {
        let mut sma = SingleSumSMA::<u32, u32>::new(3);
        sma.push(0);
        sma.push(1);
        sma.push(2);
        sma.push(3);

        assert_eq!(Some(2), sma.average(), "invalid average");
        assert_eq!(3, sma.window_size(), "window size is invalid");

        assert!(!sma.is_empty(), "should not be empty");
        assert_eq!(3, sma.len(), "len is invalid");
    }
}
