use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures::Stream;
use futures_time::task::Sleep;
use pin_project::pin_project;

/// Controller for [`RateLimitedStream`] to allow dynamic controlling of the stream's rate.
#[derive(Debug)]
pub struct RateController(Arc<AtomicUsize>, Arc<AtomicUsize>);

#[allow(unused)]
impl RateController {
    /// Update the rate limit (elements per unit).
    pub fn set_rate_per_unit(&self, elements_per_unit: usize, unit: Duration) {
        self.0.store(elements_per_unit, Ordering::Relaxed);
        self.1.store(unit.as_millis() as usize, Ordering::Relaxed);
    }

    /// Get the current rate limit per time unit.
    pub fn get_rate_per_unit(&self) -> (usize, Duration) {
        (
            self.0.load(Ordering::Relaxed),
            Duration::from_millis(self.1.load(Ordering::Relaxed) as u64),
        )
    }
}

/// A stream adapter that yields elements at a controlled rate, with dynamic rate adjustment.
///
/// See [`RateLimitExt::rate_limit_per_unit`].
#[pin_project]
pub struct RateLimitedStream<S> {
    #[pin]
    inner: S,
    elements_per_unit: Arc<AtomicUsize>,
    unit_in_ms: Arc<AtomicUsize>,
    #[pin]
    delay: Option<Sleep>,
}

impl<S> RateLimitedStream<S> {
    const MAX_RATE_PER_SEC: f64 = 10_000.0;

    /// Creates a stream with some initial rate limit of elements per a time unit.
    pub fn new_with_rate_per_unit(stream: S, initial_rate_per_unit: usize, unit: Duration) -> (Self, RateController) {
        assert!(unit > Duration::ZERO, "unit must be greater than zero");

        let rate = Arc::new(AtomicUsize::new(initial_rate_per_unit));
        let unit = Arc::new(AtomicUsize::new(unit.as_millis() as usize));
        (
            Self {
                inner: stream,
                elements_per_unit: rate.clone(),
                unit_in_ms: unit.clone(),
                delay: None,
            },
            RateController(rate, unit),
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

        // If the rate is zero, schedule the next fixed wake-up to check again
        if this.elements_per_unit.load(Ordering::Relaxed) == 0 {
            this.delay.set(Some(futures_time::task::sleep(Duration::from_millis(100).into())));
        }

        // Check if we are waiting for a delay to complete
        while let Some(delay) = this.delay.as_mut().as_pin_mut() {
            let _ = futures::ready!(delay.poll(cx));
            if this.elements_per_unit.load(Ordering::Relaxed) == 0 {
                // If the rate is still zero, schedule the next fixed wake-up to check again
                this.delay.set(Some(futures_time::task::sleep(Duration::from_millis(100).into())));
            } else {
                this.delay.set(None);
            }
        }

        // Get the next item from the inner stream
        let yield_start = Instant::now();
        if let Some(item) = futures::ready!(this.inner.as_mut().poll_next(cx)) {
            let time_to_yield = yield_start.elapsed();

            // Calculate the next allowable time based on the rate
            let units = this.elements_per_unit.load(Ordering::Relaxed);
            let rate_per_sec = (units as f64
                / Duration::from_millis(this.unit_in_ms.load(Ordering::Relaxed) as u64).as_secs_f64())
                .min(Self::MAX_RATE_PER_SEC);

            // Convert to duration (seconds per element) and subtract the duration
            // it took to yield the element from the inner stream
            let delay_duration = Duration::from_secs_f64(1.0 / rate_per_sec)
                .saturating_sub(time_to_yield)
                .max(Duration::from_secs_f64(1.0 / Self::MAX_RATE_PER_SEC));

            // Create a delay for the calculated duration
            *this.delay = Some(futures_time::task::sleep(delay_duration.into()));

            Poll::Ready(Some(item))
        } else {
            Poll::Ready(None)
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
    use super::*;
    use futures::stream::{self, StreamExt};
    use std::time::{Duration, Instant};
    use futures::pin_mut;
    use futures_time::future::FutureExt;

    #[async_std::test]
    async fn test_rate_limited_stream_respects_rate() {
        // Create a stream with 5 elements
        let stream = stream::iter(1..=5);

        // Set a rate of 10 elements per second (100ms per element)
        let (rate_limited, controller) = stream.rate_limit_per_unit(10, Duration::from_secs(1));

        assert_eq!(controller.get_rate_per_unit().0, 10);

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
            "Stream completed too quickly: {:?}",
            elapsed
        );

        // We use 600ms as an upper bound to allow for some overhead
        assert!(
            elapsed <= Duration::from_millis(600),
            "Stream completed too slowly: {:?}",
            elapsed
        );
    }

    #[async_std::test]
    async fn test_rate_limited_first_item_should_yield_immediately() {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=1);

        // Set a rate of 1 element per 100ms
        let (rate_limited, _) = stream.rate_limit_per_unit(1, Duration::from_millis(100));

        pin_mut!(rate_limited);

        let start = Instant::now();
        assert_eq!(Some(1), rate_limited.next().await);
        assert!(start.elapsed() < Duration::from_millis(100));
    }

    #[async_std::test]
    async fn test_rate_limited_stream_empty() {
        // Create an empty stream
        let stream = stream::iter::<Vec<i32>>(vec![]);

        // Apply rate limiting
        let (mut rate_limited, _) = stream.rate_limit_per_unit(10, Duration::from_secs(1));

        // Verify we get None right away
        assert_eq!(rate_limited.next().await, None);
    }

    #[async_std::test]
    async fn test_rate_limited_stream_zero_rate() -> anyhow::Result<()> {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=3);

        // Set a rate of 0 elements per second (= will not yield)
        let (rate_limited, _) = stream.rate_limit_per_unit(0, Duration::from_secs(1));

        pin_mut!(rate_limited);

        assert!(rate_limited.next().timeout(futures_time::time::Duration::from_millis(100)).await.is_err(), "zero rate should not yield anything");

        Ok(())
    }

    #[async_std::test]
    async fn test_rate_limited_stream_should_pause_on_zero_rate() -> anyhow::Result<()> {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=3);

        // Set a rate of 1 element per 100ms
        let (rate_limited, controller) = stream.rate_limit_per_unit(1, Duration::from_millis(100));

        pin_mut!(rate_limited);

        assert_eq!(Some(1), rate_limited.next().await);

        let start = Instant::now();
        assert_eq!(Some(2), rate_limited.next().await);
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(100), "first element too fast {elapsed:?}");

        controller.set_rate_per_unit(0, Duration::from_millis(100));

        assert!(rate_limited.next().timeout(futures_time::time::Duration::from_millis(200)).await.is_err(), "zero rate should not yield anything");

        Ok(())
    }

    #[async_std::test]
    async fn test_rate_limited_stream_zero_rate_should_restart_when_increased() -> anyhow::Result<()> {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=3);

        // Set a rate of 0 elements per second (= will not yield)
        let (rate_limited, controller) = stream.rate_limit_per_unit(0, Duration::from_secs(1));
        pin_mut!(rate_limited);

        assert!(rate_limited.next().timeout(futures_time::time::Duration::from_millis(100)).await.is_err(), "zero rate should not yield anything");

        controller.set_rate_per_unit(1, Duration::from_millis(100));

        let start = Instant::now();
        let all_items = rate_limited.collect::<Vec<_>>().await;
        let all_items_elapsed = start.elapsed();

        assert_eq!(all_items, vec![1, 2, 3]);
        assert!(all_items_elapsed >= Duration::from_millis(300), "all items should have been yielded in at least 300ms");

        Ok(())
    }

    #[async_std::test]
    async fn test_rate_changing_during_stream() {
        // Create a stream with 6 elements
        let stream = stream::iter(1..=6);

        // Start with 5 elements per second (200ms per element)
        let (mut rate_limited, controller) = stream.rate_limit_per_unit(5, Duration::from_secs(1));

        // Get a timer
        let start = Instant::now();

        // Consume first 3 elements
        for i in 1..=3 {
            let item = rate_limited.next().await.unwrap();
            assert_eq!(item, i);
        }

        // Measure time for the first 3 elements
        let first_half_elapsed = start.elapsed();

        // Change rate to 2 elements per second (500ms per element)
        controller.set_rate_per_unit(2, Duration::from_secs(1));

        // Consume last 3 elements
        for i in 4..=6 {
            let item = rate_limited.next().await.unwrap();
            assert_eq!(item, i);
        }

        // Measure total time
        let total_elapsed = start.elapsed();
        let second_half_elapsed = total_elapsed - first_half_elapsed;

        // The first 3 elements at 5 per second should take ~400ms (2 delays)
        assert!(
            first_half_elapsed >= Duration::from_millis(300),
            "First half too fast: {:?}",
            first_half_elapsed
        );
        assert!(
            first_half_elapsed <= Duration::from_millis(700),
            "First half too slow: {:?}",
            first_half_elapsed
        );

        // The last 3 elements at 2 per second should take ~1000ms (2 delays)
        assert!(
            second_half_elapsed >= Duration::from_millis(800),
            "Second half too fast: {:?}",
            second_half_elapsed
        );
        assert!(
            second_half_elapsed <= Duration::from_millis(1500),
            "Second half too slow: {:?}",
            second_half_elapsed
        );
    }

    #[async_std::test]
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
        // but less than 200ms (theoretical time would be ~99ms)
        assert!(
            elapsed < Duration::from_millis(200),
            "Very high rate stream took too long: {:?}",
            elapsed
        );
    }

    #[async_std::test]
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
                    async_std::task::sleep(Duration::from_millis(50)).await;
                }
            }

            items
        };

        // Set up a task to change the rate after a delay
        let rate_change_task = async move {
            // Wait a bit for the stream to start processing
            async_std::task::sleep(Duration::from_millis(100)).await;

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
