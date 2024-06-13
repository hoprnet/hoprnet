use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryToken {
    pub num_retry: usize,
    pub started_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryResult {
    Wait(Duration),
    RetryNow(RetryToken),
    Expired,
}

impl RetryToken {
    pub fn new(now: Instant) -> Self {
        Self {
            num_retry: 0,
            started_at: now,
        }
    }

    pub fn retry_at(&self, base: Duration, max_duration: Duration) -> Option<Instant> {
        let duration = base * 2_u32.pow(self.num_retry as u32);
        (duration < max_duration).then_some(self.started_at + duration)
    }

    pub fn check(&self, now: Instant, base: Duration, max: Duration) -> RetryResult {
        match self.retry_at(base, max) {
            None => RetryResult::Expired,
            Some(retry_at) if retry_at >= now => RetryResult::Wait(retry_at - now),
            _ => RetryResult::RetryNow(Self {
                num_retry: self.num_retry + 1,
                started_at: self.started_at,
            }),
        }
    }
}
