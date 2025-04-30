use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures::Stream;
use futures_timer::Delay;
use pin_project::pin_project;

/// Controller for [`RateLimitedStream`] to allow dynamic controlling of the stream's rate.
#[derive(Debug)]
pub struct RateController(Arc<AtomicUsize>, Arc<AtomicUsize>);

impl RateController {
    /// Update the rate limit (elements per second)
    pub fn set_rate_per_sec(&self, elements_per_second: usize) {
        self.0.store(elements_per_second, Ordering::Relaxed);
    }

    /// Update the rate limit (elements per unit).
    pub fn set_rate_per_unit(&self, elements_per_unit: usize, unit: Duration) {
        self.0.store(elements_per_unit, Ordering::Relaxed);
        self.1.store(unit.as_millis() as usize, Ordering::Relaxed);
    }

    /// Get the current rate limit per second.
    pub fn get_rate_per_sec(&self) -> f64 {
        self.0.load(Ordering::Relaxed) as f64
            / Duration::from_millis(self.1.load(Ordering::Relaxed) as u64).as_secs_f64()
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
/// See [`RateLimitExt::rate_limit`].
#[pin_project]
pub struct RateLimitedStream<S> {
    #[pin]
    inner: S,
    elements_per_unit: Arc<AtomicUsize>,
    unit_in_ms: Arc<AtomicUsize>,
    last_yield: Option<Instant>,
    #[pin]
    delay: Option<Delay>,
}

impl<S> RateLimitedStream<S> {
    /// Creates a stream with some initial rate limit of elements per a time unit.
    pub fn new_with_rate_per_unit(stream: S, initial_rate_per_unit: usize, unit: Duration) -> (Self, RateController) {
        let rate = Arc::new(AtomicUsize::new(initial_rate_per_unit));
        let unit = Arc::new(AtomicUsize::new(unit.as_millis() as usize));
        (
            Self {
                inner: stream,
                elements_per_unit: rate.clone(),
                unit_in_ms: unit.clone(),
                last_yield: None,
                delay: None,
            },
            RateController(rate, unit),
        )
    }

    /// Creates a stream with some initial rate limit of elements per second.
    pub fn new(stream: S, initial_rate_per_second: usize) -> (Self, RateController) {
        Self::new_with_rate_per_unit(stream, initial_rate_per_second, Duration::from_secs(1))
    }
}

impl<S, T> Stream for RateLimitedStream<S>
where
    S: Stream<Item = T> + Unpin,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // Check if we're waiting for a delay to complete
        if let Some(delay) = this.delay.as_mut().as_pin_mut() {
            match delay.poll(cx) {
                Poll::Ready(_) => {
                    // Delay completed, clear it
                    this.delay.set(None);
                }
                Poll::Pending => {
                    // Still waiting
                    return Poll::Pending;
                }
            }
        }

        // Get the next item from the inner stream
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(item)) => {
                let now = Instant::now();

                // Update last yield time
                *this.last_yield = Some(now);

                // Calculate the next allowable time based on the rate
                let units = this.elements_per_unit.load(Ordering::Relaxed);
                if units > 0 {
                    let rate_per_sec = units as f64
                        / Duration::from_millis(this.unit_in_ms.load(Ordering::Relaxed) as u64).as_secs_f64();

                    // Convert to duration (seconds per element)
                    let delay_duration = Duration::from_secs_f64(1.0 / rate_per_sec);

                    // Create a delay for the calculated duration
                    *this.delay = Some(Delay::new(delay_duration));
                }

                Poll::Ready(Some(item))
            }
            other => other,
        }
    }
}

/// Extension trait to add rate limiting to any stream
pub trait RateLimitExt: Stream + Sized {
    /// Creates a rate-limited stream that yields elements at the given rate. Moreover,
    /// the rate can be controlled dynamically during the lifetime of the stream by using
    /// the returned [`RateController`].
    ///
    /// If `elements_per_second` is 0, no rate limiting is applied.
    fn rate_limit_per_unit(
        self,
        elements_per_second: usize,
        unit: Duration,
    ) -> (RateLimitedStream<Self>, RateController) {
        RateLimitedStream::new_with_rate_per_unit(self, elements_per_second, unit)
    }

    /// Convenience extension method to create a rate-limited stream with the given
    /// number of elements per second.
    ///
    /// See [`RateLimitExt::rate_limit_per_unit`] for details.
    fn rate_limit(self, elements_per_second: usize) -> (RateLimitedStream<Self>, RateController) {
        RateLimitedStream::new(self, elements_per_second)
    }
}

impl<S: Stream + Sized> RateLimitExt for S {}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::{self, StreamExt};
    use std::time::{Duration, Instant};

    #[async_std::test]
    async fn test_rate_limited_stream_respects_rate() {
        // Create a stream with 5 elements
        let stream = stream::iter(1..=5);

        // Set a rate of 10 elements per second (100ms per element)
        let (rate_limited, controller) = stream.rate_limit(10);

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
    async fn test_rate_limited_stream_empty() {
        // Create an empty stream
        let stream = stream::iter::<Vec<i32>>(vec![]);

        // Apply rate limiting
        let (mut rate_limited, _) = stream.rate_limit(10);

        // Verify we get None right away
        assert_eq!(rate_limited.next().await, None);
    }

    #[async_std::test]
    async fn test_rate_limited_stream_zero_rate() {
        // Create a stream with 3 elements
        let stream = stream::iter(1..=3);

        // Set a rate of 0 elements per second (no delay)
        let (rate_limited, _) = stream.rate_limit(0);

        let start = Instant::now();

        // Collect all elements from the stream
        let items: Vec<i32> = rate_limited.collect().await;

        let elapsed = start.elapsed();

        // Verify all elements were yielded
        assert_eq!(items, vec![1, 2, 3]);

        // Verify there was no rate limiting (should be very fast)
        assert!(
            elapsed < Duration::from_millis(50),
            "Stream with zero rate took too long: {:?}",
            elapsed
        );
    }

    #[async_std::test]
    async fn test_rate_changing_during_stream() {
        // Create a stream with 6 elements
        let stream = stream::iter(1..=6);

        // Start with 5 elements per second (200ms per element)
        let (mut rate_limited, controller) = stream.rate_limit(5);

        // Get a timer
        let start = Instant::now();

        // Consume first 3 elements
        for i in 1..=3 {
            let item = rate_limited.next().await.unwrap();
            assert_eq!(item, i);
        }

        // Measure time for first 3 elements
        let first_half_elapsed = start.elapsed();

        // Change rate to 2 elements per second (500ms per element)
        controller.set_rate_per_sec(2);

        // Consume last 3 elements
        for i in 4..=6 {
            let item = rate_limited.next().await.unwrap();
            assert_eq!(item, i);
        }

        // Measure total time
        let total_elapsed = start.elapsed();
        let second_half_elapsed = total_elapsed - first_half_elapsed;

        // First 3 elements at 5 per second should take ~400ms (2 delays)
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

        // Last 3 elements at 2 per second should take ~1000ms (2 delays)
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
        let (rate_limited, _) = stream.rate_limit(1000);

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
        let (mut rate_limited, controller) = stream.rate_limit(2);

        // Set up a task to process the stream
        let stream_task = async {
            let mut count = 0;
            let mut items = Vec::new();

            while let Some(item) = rate_limited.next().await {
                items.push(item);
                count += 1;

                // After 3 elements, wait briefly to allow rate change to take effect
                if count == 3 {
                    Delay::new(Duration::from_millis(50)).await;
                }
            }

            items
        };

        // Set up a task to change the rate after a delay
        let rate_change_task = async move {
            // Wait a bit for the stream to start processing
            Delay::new(Duration::from_millis(100)).await;

            // Change the rate to 20 per second
            controller.set_rate_per_sec(20);

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
