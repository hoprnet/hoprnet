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

use futures_time::task::Sleep;
use pin_project::pin_project;

/// Controller for [`RateLimitedStream`] to allow dynamic controlling of the stream's rate.
#[derive(Debug)]
pub struct RateController(Arc<AtomicU64>);

impl Default for RateController {
    fn default() -> Self {
        Self(Arc::new(AtomicU64::new(0)))
    }
}

fn rate_from_delay(delay_micros: u64) -> f64 {
    if delay_micros > 0 {
        1.0 / Duration::from_micros(delay_micros).as_secs_f64()
    } else {
        0.0
    }
}

#[allow(unused)]
impl RateController {
    const MIN_DELAY: Duration = Duration::from_micros(100);

    pub fn new(elements_per_unit: usize, unit: Duration) -> Self {
        let rc = RateController::default();
        rc.set_rate_per_unit(elements_per_unit, unit);
        rc
    }

    /// Update the rate limit (elements per unit).
    pub fn set_rate_per_unit(&self, elements_per_unit: usize, unit: Duration) {
        assert!(unit > Duration::ZERO, "unit must be greater than zero");

        if elements_per_unit > 0 {
            // Calculate the next allowable time based on the rate
            let rate_per_sec = (elements_per_unit as f64 / unit.as_secs_f64());

            // Convert to duration (seconds per element)
            let new_rate = Duration::from_secs_f64(1.0 / rate_per_sec)
                .max(Self::MIN_DELAY)
                .as_micros()
                .min(u64::MAX as u128) as u64; // Clamp to u64 to avoid overflow

            self.0.store(new_rate, Ordering::Relaxed);
        } else {
            self.0.store(0, Ordering::Relaxed);
        }
    }

    /// Checks whether the rate is set to 0 at this controller.
    pub fn has_zero_rate(&self) -> bool {
        self.0.load(Ordering::Relaxed) == 0
    }

    /// Gets the delay per element (inverse rate).
    pub fn get_delay_per_element(&self) -> Duration {
        Duration::from_micros(self.0.load(Ordering::Relaxed))
    }

    /// Get the current rate limit per time unit.
    pub fn get_rate_per_sec(&self) -> f64 {
        rate_from_delay(self.0.load(Ordering::Relaxed))
    }
}

enum StreamState {
    Read,
    NoRate,
    Wait,
}

/// A stream adapter that yields elements at a controlled rate, with dynamic rate adjustment.
///
/// See [`RateLimitStreamExt::rate_limit_per_unit`].
#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub struct RateLimitedStream<S: futures::Stream> {
    #[pin]
    inner: S,
    item: Option<S::Item>,
    #[pin]
    delay: Option<Sleep>,
    state: StreamState,
    delay_time: Arc<AtomicU64>,
}

impl<S: futures::Stream> RateLimitedStream<S> {
    /// Creates a stream with rate limit controllable using the given controller.
    pub fn new_with_controller(stream: S, controller: &RateController) -> Self {
        Self {
            inner: stream,
            item: None,
            delay: None,
            state: if controller.0.load(Ordering::Relaxed) > 0 {
                StreamState::Read
            } else {
                StreamState::NoRate
            },
            delay_time: controller.0.clone(),
        }
    }

    /// Creates a stream with some initial rate limit of elements per a time unit.
    pub fn new_with_rate_per_unit(stream: S, initial_rate_per_unit: usize, unit: Duration) -> (Self, RateController) {
        let rc = RateController::new(initial_rate_per_unit, unit);
        (Self::new_with_controller(stream, &rc), rc)
    }
}

impl<S, T> futures::Stream for RateLimitedStream<S>
where
    S: futures::Stream<Item = T> + Unpin,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match this.state {
                StreamState::Read => {
                    let yield_start = Instant::now();
                    if let Some(item) = futures::ready!(this.inner.as_mut().poll_next(cx)) {
                        *this.item = Some(item);
                        let delay_time = this.delay_time.load(Ordering::Relaxed);
                        if delay_time > 0 {
                            let wait = Duration::from_micros(delay_time)
                                .saturating_sub(yield_start.elapsed())
                                .max(RateController::MIN_DELAY);
                            *this.delay = Some(futures_time::task::sleep(wait.into()));
                            *this.state = StreamState::Wait;
                        } else {
                            *this.delay = Some(futures_time::task::sleep(Duration::from_millis(100).into()));
                            *this.state = StreamState::NoRate;
                        }
                    } else {
                        return Poll::Ready(None);
                    }
                }
                StreamState::NoRate => {
                    if let Some(mut delay) = this.delay.as_mut().as_pin_mut() {
                        let _ = futures::ready!(delay.as_mut().poll(cx));
                    }
                    let delay_time = this.delay_time.load(Ordering::Relaxed);
                    if delay_time > 0 {
                        *this.delay = Some(futures_time::task::sleep(Duration::from_micros(delay_time).into()));
                        if this.item.is_some() {
                            *this.state = StreamState::Wait;
                        } else {
                            *this.state = StreamState::Read;
                        }
                    } else {
                        *this.delay = Some(futures_time::task::sleep(Duration::from_millis(100).into()));
                        *this.state = StreamState::NoRate;
                    }
                }
                StreamState::Wait => {
                    if let Some(mut delay) = this.delay.as_mut().as_pin_mut() {
                        let _ = futures::ready!(delay.as_mut().poll(cx));
                        *this.state = StreamState::Read;
                        return Poll::Ready(this.item.take());
                    }
                }
            }
        }
    }
}

/// Extension trait to add rate limiting to any stream
pub trait RateLimitStreamExt: futures::Stream + Sized {
    /// Creates a rate-limited stream that yields elements at the given rate.
    ///
    /// The rate can be controlled dynamically during the lifetime of the stream by using
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

    /// Creates a rate-limited stream that yields elements at the given rate.
    ///
    /// The rate can be controlled dynamically during the lifetime of the stream by using
    /// the given [`RateController`].
    ///
    /// If the `controller` has [zero rate](RateController::has_zero_rate),
    /// the stream will not yield until the limit is
    /// changed using the [`RateController`] to a non-zero value.
    fn rate_limit_with_controller(self, controller: &RateController) -> RateLimitedStream<Self> {
        RateLimitedStream::new_with_controller(self, controller)
    }
}

impl<S: futures::Stream + Sized> RateLimitStreamExt for S {}

/// A sink adapter that allows ingesting items at a controlled rate, with dynamic rate adjustment.
///
/// If the underlying Sink is cloneable, this object will be cloneable too, each clone having
/// the same rate-limit. Therefore, the total rate-limit is multiple of the number of clones
/// of this sink.
///
/// See [`RateLimitSinkExt::rate_limit_per_unit`].
#[must_use = "sinks do nothing unless polled"]
#[pin_project]
pub struct RateLimitedSink<S> {
    #[pin]
    inner: S,
    delay_micros: Arc<AtomicU64>,
    tokens: u64,
    last_check: Option<Instant>,
    #[pin]
    sleep: Option<Sleep>,
    state: SinkState,
}

enum SinkState {
    Ready,
    Waiting,
}

impl<S> RateLimitedSink<S> {
    /// Creates a sink with ingestion rate controllable using the given controller.
    pub fn new_with_controller(inner: S, controller: &RateController) -> Self {
        Self {
            inner,
            delay_micros: controller.0.clone(),
            tokens: 0,
            last_check: None,
            sleep: None,
            state: SinkState::Ready,
        }
    }

    /// Creates a sink with some initial rate limit of elements per a time unit.
    pub fn new_with_rate_per_unit(inner: S, initial_rate_per_unit: usize, unit: Duration) -> (Self, RateController) {
        let rc = RateController::new(initial_rate_per_unit, unit);
        (Self::new_with_controller(inner, &rc), rc)
    }
}

impl<S, Item> futures::Sink<Item> for RateLimitedSink<S>
where
    S: futures::Sink<Item> + Unpin,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        let mut this = self.project();

        loop {
            let current_delay = this.delay_micros.load(Ordering::Relaxed);
            let current_rate_limit = rate_from_delay(current_delay);

            if let Some(last_check) = this.last_check.as_mut() {
                *this.tokens += (current_rate_limit * last_check.elapsed().as_secs_f64()).round() as u64;
                *last_check = Instant::now();
            } else {
                // This happens only on the first poll_ready
                *this.last_check = Some(Instant::now());
            }

            match this.state {
                SinkState::Ready => {
                    if *this.tokens > 0 {
                        futures::ready!(this.inner.as_mut().poll_ready(cx))?;

                        tracing::trace!(tokens = *this.tokens, "tokens left");
                        return Poll::Ready(Ok(()));
                    } else {
                        tracing::trace!("no tokens left");
                        *this.state = SinkState::Waiting;
                    }
                }
                SinkState::Waiting => {
                    if let Some(sleep) = this.sleep.as_mut().as_pin_mut() {
                        futures::ready!(sleep.poll(cx));
                        this.sleep.set(None);

                        tracing::trace!("waiting done");
                        *this.state = SinkState::Ready;
                    } else {
                        if current_delay > 0 {
                            // Sleep the minimum amount of time to replenish at least one token
                            *this.sleep = Some(futures_time::task::sleep(futures_time::time::Duration::from_micros(
                                current_delay,
                            )));
                        } else {
                            *this.sleep =
                                Some(futures_time::task::sleep(futures_time::time::Duration::from_millis(50)));
                        }
                    }
                }
            }
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), S::Error> {
        let this = self.project();
        if *this.tokens > 0 {
            *this.tokens -= 1;
            tracing::trace!("token consumed");
            this.inner.start_send(item)
        } else {
            panic!("start_send called without poll_ready");
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<S: Clone> Clone for RateLimitedSink<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            delay_micros: self.delay_micros.clone(),
            tokens: 0,
            last_check: None,
            sleep: None,
            state: SinkState::Ready,
        }
    }
}

/// Extension trait to add rate limiting to any sink.
pub trait RateLimitSinkExt<T>: futures::Sink<T> + Sized {
    /// Creates a rate-limited sink that allows ingesting items at the given rate.
    ///
    /// The rate can be controlled dynamically during the lifetime of the sink by using
    /// the returned [`RateController`].
    ///
    /// If `elements_per_unit` is 0, the sink will not ingest items until the limit is changed
    /// using the [`RateController`] to a non-zero value.
    fn rate_limit_per_unit(self, elements_per_unit: usize, unit: Duration) -> (RateLimitedSink<Self>, RateController) {
        RateLimitedSink::new_with_rate_per_unit(self, elements_per_unit, unit)
    }

    /// Creates a rate-limited sink that allows ingesting items at the given rate.
    ///
    /// The rate can be controlled dynamically during the lifetime of the sink by using
    /// the given [`RateController`].
    ///
    /// If the `controller` has [zero rate](RateController::has_zero_rate),
    /// the sink will not ingest items until the limit is
    /// changed using the [`RateController`] to a non-zero value.
    fn rate_limit_with_controller(self, controller: &RateController) -> RateLimitedSink<Self> {
        RateLimitedSink::new_with_controller(self, controller)
    }
}

impl<T, S: futures::Sink<T> + Sized> RateLimitSinkExt<T> for S {}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use futures::{
        SinkExt, pin_mut,
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

    #[test_log::test(tokio::test)]
    async fn rate_limited_sink_should_respect_rate() -> anyhow::Result<()> {
        let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
        let (tx, _) = tx.rate_limit_per_unit(5, Duration::from_millis(100));

        let input = (0..10).collect::<Vec<_>>();

        let start = Instant::now();
        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter(input.clone()).map(Ok))
            .timeout(futures_time::time::Duration::from_millis(500))
            .await??;

        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(200),
            "sending took too little {elapsed:?}"
        );

        tx.close().await?;

        let collected = rx.collect::<Vec<_>>().await;
        assert_eq!(input, collected);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn rate_limited_sink_should_replenish_when_idle() -> anyhow::Result<()> {
        let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
        let (tx, _) = tx.rate_limit_per_unit(5, Duration::from_millis(100));

        pin_mut!(tx);

        let input = (0..5).collect::<Vec<_>>();

        let start = Instant::now();
        tx.send_all(&mut futures::stream::iter(input.clone()).map(Ok))
            .timeout(futures_time::time::Duration::from_millis(500))
            .await??;

        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(120),
            "sending took too much {elapsed:?}"
        );

        tokio::time::sleep(Duration::from_millis(100)).await;

        let start = Instant::now();
        tx.send_all(&mut futures::stream::iter(input.clone()).map(Ok))
            .timeout(futures_time::time::Duration::from_millis(500))
            .await??;

        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(120),
            "sending took too much {elapsed:?}"
        );

        tx.close().await?;

        let collected = rx.collect::<Vec<_>>().await;
        assert_eq!(input.into_iter().cycle().take(10).collect::<Vec<_>>(), collected);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn rate_limited_sink_should_not_send_when_zero_rate() -> anyhow::Result<()> {
        let (tx, _) = futures::channel::mpsc::unbounded::<i32>();
        let (tx, _) = tx.rate_limit_per_unit(0, Duration::from_millis(100));

        pin_mut!(tx);

        assert!(
            tx.send(1i32)
                .timeout(futures_time::time::Duration::from_millis(100))
                .await
                .is_err()
        );
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn rate_limited_sink_should_recover_after_rate_is_increased() -> anyhow::Result<()> {
        let (tx, rx) = futures::channel::mpsc::unbounded::<i32>();
        let (tx, ctl) = tx.rate_limit_per_unit(0, Duration::from_millis(100));

        pin_mut!(tx);

        assert!(
            tx.send(1i32)
                .timeout(futures_time::time::Duration::from_millis(100))
                .await
                .is_err()
        );

        ctl.set_rate_per_unit(10, Duration::from_millis(10));

        tx.send(2i32)
            .timeout(futures_time::time::Duration::from_millis(100))
            .await??;

        pin_mut!(rx);
        assert_eq!(
            Some(2),
            rx.next()
                .timeout(futures_time::time::Duration::from_millis(100))
                .await?
        );

        Ok(())
    }
}
