use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetryToken {
    pub num_retry: usize,
    pub started_at: Instant,
    backoff_base: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RetryResult {
    Wait(Duration),
    RetryNow(RetryToken),
    Expired,
}

impl RetryToken {
    pub fn new(now: Instant, backoff_base: f64) -> Self {
        Self {
            num_retry: 0,
            started_at: now,
            backoff_base,
        }
    }

    pub fn retry_in(&self, base: Duration, max_duration: Duration) -> Option<Duration> {
        let duration = base.mul_f64(self.backoff_base.powi(self.num_retry as i32));
        (duration < max_duration).then_some(duration)
    }

    pub fn retry_at(&self, base: Duration, max_duration: Duration) -> Option<Instant> {
        self.retry_in(base, max_duration).map(|d| self.started_at + d)
    }

    pub fn check(&self, now: Instant, base: Duration, max: Duration) -> RetryResult {
        match self.retry_at(base, max) {
            None => RetryResult::Expired,
            Some(retry_at) if retry_at >= now => RetryResult::Wait(retry_at - now),
            _ => RetryResult::RetryNow(Self {
                num_retry: self.num_retry + 1,
                started_at: self.started_at,
                backoff_base: self.backoff_base,
            }),
        }
    }
}
