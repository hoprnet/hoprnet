use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryLog(usize, Instant);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryResult {
    Wait,
    Next(RetryLog),
    Expired,
}

impl RetryLog {
    pub fn new(now: Instant, base: Duration) -> Self {
        Self(0, now + base)
    }

    pub fn check(&self, now: Instant, base: Duration, max: Duration) -> RetryResult {
        if now >= self.1 {
            let base_ms = base.as_millis() as u64;
            let retry_no = self.0 + 1;
            let next = Duration::from_millis(base_ms.pow(retry_no as u32));
            if next < max {
                RetryResult::Next(Self(retry_no, now + next))
            } else {
                RetryResult::Expired
            }
        } else {
            RetryResult::Wait
        }
    }
}
