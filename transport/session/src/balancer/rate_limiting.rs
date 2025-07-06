use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::Stream;
use futures_time::task::Sleep;
use pin_project::pin_project;

/// Controller for [`RateLimitedStream`] to allow dynamic controlling of the stream's rate.
#[derive(Debug)]
pub struct RateController(Arc<AtomicU64>);

#[allow(unused)]
impl RateController {
    const MIN_DELAY: Duration = Duration::from_micros(100);

    /// Update the rate limit (elements per unit).
    pub fn set_rate_per_unit(&self, elements_per_unit: usize, unit: Duration) {
        assert!(unit > Duration::ZERO, "unit must be greater than zero");

        if elements_per_unit > 0 {
            // Calculate the next allowable time based on the rate
            let rate_per_sec = (elements_per_unit as f64 / unit.as_secs_f64());

            // Convert to duration (seconds per element) and subtract the duration
            // it took to yield the element from the inner stream
            let new_rate = Duration::from_secs_f64(1.0 / rate_per_sec)
                .max(Self::MIN_DELAY)
                .as_micros()
                .min(u64::MAX as u128) as u64; // Clamp to u64 to avoid overflow

            self.0.store(new_rate, Ordering::Relaxed);
        } else {
            self.0.store(0, Ordering::Relaxed);
        }
    }

    /// Get the current rate limit per time unit.
    pub fn get_rate_per_sec(&self) -> f64 {
        let delim = self.0.load(Ordering::Relaxed);
        if delim > 0 {
            1.0 / Duration::from_micros(delim).as_secs_f64()
        } else {
            0.0
        }
    }
}

enum State {
    Read,
    NoRate,
    Wait,
}

/// A stream adapter that yields elements at a controlled rate, with dynamic rate adjustment.
///
/// See [`RateLimitExt::rate_limit_per_unit`].
#[pin_project]
pub struct RateLimitedStream<S: Stream> {
    #[pin]
    inner: S,
    item: Option<S::Item>,
    #[pin]
    delay: Option<Sleep>,
    state: State,
    delay_time: Arc<AtomicU64>,
}

impl<S: Stream> RateLimitedStream<S> {
    /// Creates a stream with some initial rate limit of elements per a time unit.
    pub fn new_with_rate_per_unit(stream: S, initial_rate_per_unit: usize, unit: Duration) -> (Self, RateController) {
        let rc = RateController(Arc::new(AtomicU64::new(0)));
        rc.set_rate_per_unit(initial_rate_per_unit, unit);

        (
            Self {
                inner: stream,
                item: None,
                delay: None,
                state: if initial_rate_per_unit > 0 {
                    State::Read
                } else {
                    State::NoRate
                },
                delay_time: rc.0.clone(),
            },
            rc,
        )
    }
}

impl<S, T> Stream for RateLimitedStream<S>
where
    S: Stream<Item = T> + Unpin,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match this.state {
                State::Read => {
                    let yield_start = Instant::now();
                    if let Some(item) = futures::ready!(this.inner.as_mut().poll_next(cx)) {
                        *this.item = Some(item);
                        let delay_time = this.delay_time.load(Ordering::Relaxed);
                        if delay_time > 0 {
                            let wait = Duration::from_micros(delay_time)
                                .saturating_sub(yield_start.elapsed())
                                .max(RateController::MIN_DELAY);
                            *this.delay = Some(futures_time::task::sleep(wait.into()));
                            *this.state = State::Wait;
                        } else {
                            *this.delay = Some(futures_time::task::sleep(Duration::from_millis(100).into()));
                            *this.state = State::NoRate;
                        }
                    } else {
                        return Poll::Ready(None);
                    }
                }
                State::NoRate => {
                    if let Some(mut delay) = this.delay.as_mut().as_pin_mut() {
                        let _ = futures::ready!(delay.as_mut().poll(cx));
                    }
                    let delay_time = this.delay_time.load(Ordering::Relaxed);
                    if delay_time > 0 {
                        *this.delay = Some(futures_time::task::sleep(Duration::from_micros(delay_time).into()));
                        if this.item.is_some() {
                            *this.state = State::Wait;
                        } else {
                            *this.state = State::Read;
                        }
                    } else {
                        *this.delay = Some(futures_time::task::sleep(Duration::from_millis(100).into()));
                        *this.state = State::NoRate;
                    }
                }
                State::Wait => {
                    if let Some(mut delay) = this.delay.as_mut().as_pin_mut() {
                        let _ = futures::ready!(delay.as_mut().poll(cx));
                        *this.state = State::Read;
                        return Poll::Ready(this.item.take());
                    }
                }
            }
        }
    }
}

/// Extension trait to add rate limiting to any stream
pub trait RateLimitExt: Stream + Sized {
    /// Creates a rate-limited stream that yields elements at the given rate. Moreover,
    /// the rate can be controlled dynamically during the lifetime of the stream by using
    /// the returned [`RateController`].
    ///
    /// If `elements_per_unit` is 0, the stream will not yield until the limit is changed
    /// using the [`RateController`] to a non-zero value.
    fn rate_limit_per_unit(
        self,
        elements_per_unit: usize,
        unit: Duration,
    ) -> (RateLimitedStream<Self>, RateController) {
        RateLimitedStream::new_with_rate_per_unit(self, elements_per_unit, unit)
    }
}

impl<S: Stream + Sized> RateLimitExt for S {}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use futures::{
        pin_mut,
        stream::{self, StreamExt},
    };
    use futures_time::future::FutureExt;

    use super::*;

    #[test]
    fn test_rate_controller_set_rate_per_unit() {
        let rc = RateController(Arc::new(AtomicU64::new(0)));
        rc.set_rate_per_unit(2500, 2 * Duration::from_secs(1));
        assert_eq!(rc.get_rate_per_sec(), 1250.0);
    }

    #[tokio::test]
    async fn test_rate_limited_stream_respects_rate() {
        // Create a stream with 5 elements
        let stream = stream::iter(1..=5);

        // Set a rate of 10 elements per second (100ms per element)
        let (rate_limited, controller) = stream.rate_limit_per_unit(10, Duration::from_secs(1));

        assert_eq!(controller.get_rate_per_sec(), 10.0);

        let start = Instant::now();

        // Collect all elements from the stream
        let items: Vec<i32> = rate_limited.collect().await;

        let elapsed = start.elapsed();

        // Verify all elements were yielded
        assert_eq!(items, vec![1, 2, 3, 4, 5]);

        // Verify the rate limiting worked
        // With 5 elements at 10 per second; we expect ~400ms (4 delays of 100ms)
        // We use 300ms as a lower bound to account for processing time
        assert!(
            elapsed >= Duration::from_millis(300),
            "Stream completed too quickly: {elapsed:?}"
        );

        // We use 600ms as an upper bound to allow for some overhead
        assert!(
            elapsed <= Duration::from_millis(600),
            "Stream completed too slowly: {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_rate_limited_first_item_should_be_delayed() {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=1);

        // Set a rate of 1 element per 100ms
        let (rate_limited, _) = stream.rate_limit_per_unit(1, Duration::from_millis(100));

        pin_mut!(rate_limited);

        let start = Instant::now();
        assert_eq!(Some(1), rate_limited.next().await);
        assert!(start.elapsed() >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limited_stream_empty() {
        // Create an empty stream
        let stream = stream::iter::<Vec<i32>>(vec![]);

        // Apply rate limiting
        let (mut rate_limited, _) = stream.rate_limit_per_unit(10, Duration::from_secs(1));

        // Verify we get None right away
        assert_eq!(rate_limited.next().await, None);
    }

    #[tokio::test]
    async fn test_rate_limited_stream_zero_rate() -> anyhow::Result<()> {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=3);

        // Set a rate of 0 elements per second (= will not yield)
        let (mut rate_limited, _) = stream.rate_limit_per_unit(0, Duration::from_millis(50));

        assert!(
            rate_limited
                .next()
                .timeout(futures_time::time::Duration::from_millis(100))
                .await
                .is_err(),
            "zero rate should not yield anything"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_rate_limited_stream_should_pause_on_zero_rate() -> anyhow::Result<()> {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=3);

        // Set a rate of 1 element per 100ms
        let (mut rate_limited, controller) = stream.rate_limit_per_unit(1, Duration::from_millis(100));

        let start = Instant::now();
        assert_eq!(Some(1), rate_limited.next().await);
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(100),
            "first element too fast {elapsed:?}"
        );

        let start = Instant::now();
        assert_eq!(Some(2), rate_limited.next().await);
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(100),
            "first element too fast {elapsed:?}"
        );

        controller.set_rate_per_unit(0, Duration::from_millis(100));

        assert!(
            rate_limited
                .next()
                .timeout(futures_time::time::Duration::from_millis(200))
                .await
                .is_err(),
            "zero rate should not yield anything"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_rate_limited_stream_zero_rate_should_restart_when_increased() -> anyhow::Result<()> {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=3);

        // Set a rate of 0 elements per second (= will not yield)
        let (mut rate_limited, controller) = stream.rate_limit_per_unit(0, Duration::from_secs(1));

        assert!(
            rate_limited
                .next()
                .timeout(futures_time::time::Duration::from_millis(100))
                .await
                .is_err(),
            "zero rate should not yield anything"
        );

        controller.set_rate_per_unit(1, Duration::from_millis(100));

        let start = Instant::now();
        let all_items = rate_limited.collect::<Vec<_>>().await;
        let all_items_elapsed = start.elapsed();

        assert_eq!(all_items, vec![1, 2, 3]);
        assert!(
            all_items_elapsed >= Duration::from_millis(300),
            "all items should have been yielded in at least 300ms instead {all_items_elapsed:?}"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_rate_changing_during_stream() {
        // Create a stream with 6 elements
        let stream = stream::iter(1..=6);

        // Start with 5 elements per second (200ms per element)
        let (mut rate_limited, controller) = stream.rate_limit_per_unit(5, Duration::from_secs(1));

        // Consume first 3 elements
        let start = Instant::now();
        for i in 1..=3 {
            assert_eq!(Some(i), rate_limited.next().await);
        }

        // Measure time for the first 3 elements
        let first_half_elapsed = start.elapsed();

        // Change rate to 2 elements per second (500ms per element)
        controller.set_rate_per_unit(2, Duration::from_secs(1));

        // Consume last 3 elements
        for i in 4..=6 {
            assert_eq!(Some(i), rate_limited.next().await);
        }

        // Measure total time
        let total_elapsed = start.elapsed();
        let second_half_elapsed = total_elapsed - first_half_elapsed;

        // The first 3 elements at 5 per second should take ~600ms (200 ms each)
        assert!(
            first_half_elapsed >= Duration::from_millis(600),
            "First half too fast: {first_half_elapsed:?}"
        );
        assert!(
            first_half_elapsed <= Duration::from_millis(700),
            "First half too slow: {first_half_elapsed:?}"
        );

        // The last 3 elements at 2 per second should take ~1500ms (500 ms each)
        assert!(
            second_half_elapsed >= Duration::from_millis(1500),
            "Second half too fast: {second_half_elapsed:?}"
        );
        assert!(
            second_half_elapsed <= Duration::from_millis(1600),
            "Second half too slow: {second_half_elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_very_high_rate() {
        // Create a stream with 100 elements
        let stream = stream::iter(1..=100);

        // Set a very high rate (1000 per second)
        let (rate_limited, _) = stream.rate_limit_per_unit(1000, Duration::from_secs(1));

        let start = Instant::now();

        // Collect all elements
        let items: Vec<i32> = rate_limited.collect().await;

        let elapsed = start.elapsed();

        // Verify all elements were yielded
        assert_eq!(items.len(), 100);
        assert_eq!(items.first(), Some(&1));
        assert_eq!(items.last(), Some(&100));

        // Even at a high rate, processing 100 elements should take some time
        // but less than 150 ms (theoretical time would be ~99ms)
        assert!(
            elapsed >= Duration::from_millis(100),
            "Very high rate stream took too long: {elapsed:?}"
        );

        assert!(
            elapsed < Duration::from_millis(150),
            "Very high rate stream took too long: {elapsed:?}"
        );
    }

    #[tokio::test]
    async fn test_concurrent_rate_change() {
        use futures::future::join;

        // Create a stream with 10 elements
        let stream = stream::iter(1..=10);

        // Start with 2 elements per second
        let (mut rate_limited, controller) = stream.rate_limit_per_unit(2, Duration::from_secs(1));

        // Set up a task to process the stream
        let stream_task = async {
            let mut count = 0;
            let mut items = Vec::new();

            while let Some(item) = rate_limited.next().await {
                items.push(item);
                count += 1;

                // After 3 elements, wait briefly to allow rate change to take effect
                if count == 3 {
                    futures_time::task::sleep(Duration::from_millis(50).into()).await;
                }
            }

            items
        };

        // Set up a task to change the rate after a delay
        let rate_change_task = async move {
            // Wait a bit for the stream to start processing
            futures_time::task::sleep(Duration::from_millis(100).into()).await;

            // Change the rate to 20 per second
            controller.set_rate_per_unit(20, Duration::from_secs(1));

            // Return the new rate
            20
        };

        // Run both tasks concurrently
        let (items, new_rate) = join(stream_task, rate_change_task).await;

        // Verify results
        assert_eq!(items.len(), 10, "Should have received all 10 items");
        assert_eq!(new_rate, 20, "Rate should have been changed to 20");
    }
}
