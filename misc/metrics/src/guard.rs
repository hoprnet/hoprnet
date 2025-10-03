use crate::{SimpleGauge, metrics::MultiGauge};

/// Creates a RAII guard for a simple gauge metric.
///
/// The metric is incremented when the guard is created and decremented when it is dropped.
pub struct GaugeGuard<'a> {
    metric: &'a SimpleGauge,
    by: f64,
}

impl<'a> GaugeGuard<'a> {
    pub fn new(metric: &'a SimpleGauge, by: f64) -> Self {
        metric.increment(by);
        Self { metric, by }
    }
}

impl<'a> Drop for GaugeGuard<'a> {
    fn drop(&mut self) {
        self.metric.decrement(self.by)
    }
}

/// Creates a RAII guard for a multi-gauge metric.
///
/// The metric with the given labels is incremented when the guard is created and decremented
/// when the guard is dropped.
pub struct MultiGaugeGuard<'a> {
    metric: &'a MultiGauge,
    labels: &'a [&'a str],
    by: f64,
}

impl<'a> MultiGaugeGuard<'a> {
    pub fn new(metric: &'a MultiGauge, labels: &'a [&'a str], by: f64) -> Self {
        metric.increment(labels, by);
        Self { metric, labels, by }
    }
}

impl<'a> Drop for MultiGaugeGuard<'a> {
    fn drop(&mut self) {
        self.metric.decrement(self.labels, self.by)
    }
}
