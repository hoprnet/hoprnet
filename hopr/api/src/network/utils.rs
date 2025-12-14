/// An exponential moving average calculator.
///
/// Based on the formula:
///
/// EMA_t = EMA_{t-1} + (Value_t - EMA_{t-1}) / min(t, FACTOR)
///
/// this object maintains a running average that emphasizes recent values more heavily
/// and creates a smooth long-tailed averaging effect over time.
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct ExponentialMovingAverage<const FACTOR: usize> {
    count: usize,
    average: f64,
}

impl<const FACTOR: usize> ExponentialMovingAverage<FACTOR> {
    /// Updates the moving average with a new value.
    pub fn update(&mut self, value: impl Into<f64>) {
        let value: f64 = value.into();
        self.count += 1;
        self.average = self.average + (value - self.average) / (std::cmp::min(self.count, FACTOR) as f64);
    }

    /// Retrieves the current value of the moving average.
    pub fn get(&self) -> f64 {
        self.average
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn running_average_should_compute_the_windowed_average_correctly() {
        let mut avg = super::ExponentialMovingAverage::<5>::default();

        for i in 1..=10 {
            avg.update(i);
        }

        assertables::assert_in_delta!(avg.get(), 6.6, 0.1);
    }

    #[test]
    fn running_average_should_compute_the_average_from_constant_correctly() {
        let mut avg = super::ExponentialMovingAverage::<5>::default();

        for _ in 1..=10 {
            avg.update(3);
        }

        assertables::assert_f64_eq!(avg.get(), 3.0);
    }
}
