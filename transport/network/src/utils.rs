#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct ExponentialMovingAverage<const FACTOR: usize> {
    count: usize,
    average: f64,
}

impl<const FACTOR: usize> ExponentialMovingAverage<FACTOR> {
    pub fn update(&mut self, value: impl Into<f64>) {
        let value: f64 = value.into();
        self.count += 1;
        self.average = self.average + (value - self.average) / (std::cmp::min(self.count, FACTOR) as f64);
    }

    pub fn get(&self) -> f64 {
        self.average
    }
}
