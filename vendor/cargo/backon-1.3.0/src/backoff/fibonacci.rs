use core::time::Duration;

use crate::backoff::BackoffBuilder;

/// FibonacciBuilder is used to build a [`FibonacciBackoff`] which offers a delay with Fibonacci-based retries.
///
/// # Default
///
/// - jitter: false
/// - min_delay: 1s
/// - max_delay: 60s
/// - max_times: 3
///
/// # Examples
///
/// ```no_run
/// use anyhow::Result;
/// use backon::FibonacciBuilder;
/// use backon::Retryable;
///
/// async fn fetch() -> Result<String> {
///     Ok(reqwest::get("https://www.rust-lang.org")
///         .await?
///         .text()
///         .await?)
/// }
///
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() -> Result<()> {
///     let content = fetch.retry(FibonacciBuilder::default()).await?;
///     println!("fetch succeeded: {}", content);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct FibonacciBuilder {
    jitter: bool,
    min_delay: Duration,
    max_delay: Option<Duration>,
    max_times: Option<usize>,
}

impl Default for FibonacciBuilder {
    fn default() -> Self {
        Self {
            jitter: false,
            min_delay: Duration::from_secs(1),
            max_delay: Some(Duration::from_secs(60)),
            max_times: Some(3),
        }
    }
}

impl FibonacciBuilder {
    /// Set the jitter for the backoff.
    ///
    /// When jitter is enabled, FibonacciBackoff will add a random jitter between `(0, min_delay)` to the delay.
    pub fn with_jitter(mut self) -> Self {
        self.jitter = true;
        self
    }

    /// Set the minimum delay for the backoff.
    pub fn with_min_delay(mut self, min_delay: Duration) -> Self {
        self.min_delay = min_delay;
        self
    }

    /// Set the maximum delay for the current backoff.
    ///
    /// The delay will not increase if the current delay exceeds the maximum delay.
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = Some(max_delay);
        self
    }

    /// Set no maximum delay for the backoff.
    ///
    /// The delay will keep increasing.
    ///
    /// _The delay will saturate at `Duration::MAX` which is an **unrealistic** delay._
    pub fn without_max_delay(mut self) -> Self {
        self.max_delay = None;
        self
    }

    /// Set the maximum number of attempts for the current backoff.
    ///
    /// The backoff will stop if the maximum number of attempts is reached.
    pub fn with_max_times(mut self, max_times: usize) -> Self {
        self.max_times = Some(max_times);
        self
    }

    /// Set no maximum number of attempts for the current backoff.
    ///
    /// The backoff will not stop by itself.
    ///
    /// _The backoff could stop reaching `usize::MAX` attempts but this is **unrealistic**._
    pub fn without_max_times(mut self) -> Self {
        self.max_times = None;
        self
    }
}

impl BackoffBuilder for FibonacciBuilder {
    type Backoff = FibonacciBackoff;

    fn build(self) -> Self::Backoff {
        FibonacciBackoff {
            jitter: self.jitter,
            min_delay: self.min_delay,
            max_delay: self.max_delay,
            max_times: self.max_times,

            previous_delay: None,
            current_delay: None,
            attempts: 0,
        }
    }
}

impl BackoffBuilder for &FibonacciBuilder {
    type Backoff = FibonacciBackoff;

    fn build(self) -> Self::Backoff {
        (*self).build()
    }
}

/// FibonacciBackoff offers a delay with Fibonacci-based retries.
///
/// This backoff strategy is constructed by [`FibonacciBuilder`].
#[doc(hidden)]
#[derive(Debug)]
pub struct FibonacciBackoff {
    jitter: bool,
    min_delay: Duration,
    max_delay: Option<Duration>,
    max_times: Option<usize>,

    previous_delay: Option<Duration>,
    current_delay: Option<Duration>,
    attempts: usize,
}

impl Iterator for FibonacciBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.attempts >= self.max_times.unwrap_or(usize::MAX) {
            return None;
        }
        self.attempts += 1;

        match self.current_delay {
            None => {
                // If current_delay is None, it's must be the first time to retry.
                let mut next = self.min_delay;
                self.current_delay = Some(next);

                // If jitter is enabled, add random jitter based on min delay.
                if self.jitter {
                    next += self.min_delay.mul_f32(fastrand::f32());
                }

                Some(next)
            }
            Some(cur) => {
                let mut next = cur;

                // If current delay larger than max delay, we should stop increment anymore.
                if next < self.max_delay.unwrap_or(Duration::MAX) {
                    if let Some(prev) = self.previous_delay {
                        next = next.saturating_add(prev);
                        self.current_delay = Some(next);
                    }
                    self.previous_delay = Some(cur);
                }

                // If jitter is enabled, add random jitter based on min delay.
                if self.jitter {
                    next += self.min_delay.mul_f32(fastrand::f32());
                }

                Some(next)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::BackoffBuilder;
    use crate::FibonacciBuilder;

    #[test]
    fn test_fibonacci_default() {
        let mut fib = FibonacciBuilder::default().build();

        assert_eq!(Some(Duration::from_secs(1)), fib.next());
        assert_eq!(Some(Duration::from_secs(1)), fib.next());
        assert_eq!(Some(Duration::from_secs(2)), fib.next());
        assert_eq!(None, fib.next());
    }

    #[test]
    fn test_fibonacci_jitter() {
        let mut fib = FibonacciBuilder::default().with_jitter().build();

        let v = fib.next().expect("value must valid");
        assert!(v >= Duration::from_secs(1), "current: {v:?}");
        assert!(v < Duration::from_secs(2), "current: {v:?}");

        let v = fib.next().expect("value must valid");
        assert!(v >= Duration::from_secs(1), "current: {v:?}");
        assert!(v < Duration::from_secs(2), "current: {v:?}");

        let v = fib.next().expect("value must valid");
        assert!(v >= Duration::from_secs(2), "current: {v:?}");
        assert!(v < Duration::from_secs(3), "current: {v:?}");

        assert_eq!(None, fib.next());
    }

    #[test]
    fn test_fibonacci_min_delay() {
        let mut fib = FibonacciBuilder::default()
            .with_min_delay(Duration::from_millis(500))
            .build();

        assert_eq!(Some(Duration::from_millis(500)), fib.next());
        assert_eq!(Some(Duration::from_millis(500)), fib.next());
        assert_eq!(Some(Duration::from_secs(1)), fib.next());
        assert_eq!(None, fib.next());
    }

    #[test]
    fn test_fibonacci_max_delay() {
        let mut fib = FibonacciBuilder::default()
            .with_max_times(4)
            .with_max_delay(Duration::from_secs(2))
            .build();

        assert_eq!(Some(Duration::from_secs(1)), fib.next());
        assert_eq!(Some(Duration::from_secs(1)), fib.next());
        assert_eq!(Some(Duration::from_secs(2)), fib.next());
        assert_eq!(Some(Duration::from_secs(2)), fib.next());
        assert_eq!(None, fib.next());
    }

    #[test]
    fn test_fibonacci_no_max_delay() {
        let mut fib = FibonacciBuilder::default()
            .with_max_times(4)
            .with_min_delay(Duration::from_secs(10_000_000_000_000_000_000))
            .without_max_delay()
            .build();

        assert_eq!(
            Some(Duration::from_secs(10_000_000_000_000_000_000)),
            fib.next()
        );
        assert_eq!(
            Some(Duration::from_secs(10_000_000_000_000_000_000)),
            fib.next()
        );
        assert_eq!(Some(Duration::MAX), fib.next());
        assert_eq!(Some(Duration::MAX), fib.next());
        assert_eq!(None, fib.next());
    }

    #[test]
    fn test_fibonacci_max_times() {
        let mut fib = FibonacciBuilder::default().with_max_times(6).build();

        assert_eq!(Some(Duration::from_secs(1)), fib.next());
        assert_eq!(Some(Duration::from_secs(1)), fib.next());
        assert_eq!(Some(Duration::from_secs(2)), fib.next());
        assert_eq!(Some(Duration::from_secs(3)), fib.next());
        assert_eq!(Some(Duration::from_secs(5)), fib.next());
        assert_eq!(Some(Duration::from_secs(8)), fib.next());
        assert_eq!(None, fib.next());
    }

    #[test]
    fn test_fibonacci_no_max_times() {
        let mut fib = FibonacciBuilder::default()
            .with_min_delay(Duration::from_secs(0))
            .without_max_times()
            .build();

        // to fully test we would need to call this `usize::MAX`
        // which seems unreasonable for a test as it would take too long...
        for _ in 0..10_000 {
            assert_eq!(Some(Duration::from_secs(0)), fib.next());
        }
    }
}
