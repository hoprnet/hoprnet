use std::fmt::Display;
use std::iter::Sum;
use std::ops::{Add, Div};
use ringbuffer::{AllocRingBuffer, RingBuffer};

#[derive(Clone, Debug, PartialEq)]
pub struct SMA<T>
where T: Add<T, Output = T> + Div<usize, Output = T> + Clone + for<'a> Sum<&'a T> {
    window: AllocRingBuffer<T>,
    samples_added: usize,
}

impl<T> SMA<T>
where T: Add<T, Output = T> + Div<usize, Output = T> + Clone + for<'a> Sum<&'a T> {
    pub fn new(window_size: usize) -> Self {
        assert!(window_size > 1, "window size must be greater than 1");
        Self {
            window: AllocRingBuffer::new(window_size),
            samples_added: 0
        }
    }

    pub fn add_sample(&mut self, sample: T) {
        self.window.push(sample);
        self.samples_added += 1;
    }

    pub fn num_samples_added(&self) -> usize {
        self.samples_added
    }

    pub fn window_size(&self) -> usize {
        self.window.capacity()
    }

    pub fn get_average(&self) -> T {
        self.window.iter().sum::<T>() / self.window.len()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_sma_should_calculate_avg_correctly() {

    }
}