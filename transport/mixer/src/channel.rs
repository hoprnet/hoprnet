use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    future::poll_fn,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::Poll,
};

use futures::{Stream, StreamExt};
use parking_lot::Mutex;
use tokio::time::Sleep;
use tracing::trace;

use crate::{config::MixerConfig, data::DelayedData};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    pub static ref METRIC_QUEUE_SIZE: hopr_types::telemetry::SimpleGauge =
        hopr_types::telemetry::SimpleGauge::new("hopr_mixer_queue_size", "Current mixer queue size").unwrap();
    pub static ref METRIC_MIXER_AVERAGE_DELAY: hopr_types::telemetry::SimpleGauge = hopr_types::telemetry::SimpleGauge::new(
        "hopr_mixer_average_packet_delay",
        "Average mixer packet delay averaged over a packet window"
    )
    .unwrap();
}

/// Mixing and delaying channel using random delay function.
///
/// Mixing is performed by assigning random delays to the ingress timestamp of data,
/// then storing the values inside a binary heap with reversed ordering (max heap).
/// This effectively creates a min heap behavior, which is required to ensure that
/// data is released in order of their delay expiration.
///
/// When data arrives:
/// 1. A random delay is assigned
/// 2. Data is stored in the heap with its release timestamp
/// 3. The heap maintains ordering so items with earliest release time are at the top
///
/// This channel is **unbounded** by nature using the `capacity` in the configuration
/// to solely pre-allocate the buffer.
///
/// The timer used by the receiver to wait for the next release deadline is **not** stored
/// behind this mutex — it lives on the [`Receiver`] itself. Keeping it out of the shared
/// state is what lets the receiver poll the timer without blocking senders.
struct Channel<T> {
    /// Buffer holding the data with a timestamp ordering to ensure the min heap behavior.
    buffer: BinaryHeap<Reverse<DelayedData<T>>>,
    waker: Option<std::task::Waker>,
    cfg: MixerConfig,
}

/// Channel with sender and receiver counters allowing closure tracking.
struct TrackedChannel<T> {
    channel: Arc<Mutex<Channel<T>>>,
    sender_count: Arc<AtomicUsize>,
    receiver_active: Arc<AtomicBool>,
}

impl<T> Clone for TrackedChannel<T> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
            sender_count: self.sender_count.clone(),
            receiver_active: self.receiver_active.clone(),
        }
    }
}

/// Error returned by the [`Sender`].
#[derive(Clone, Debug, thiserror::Error)]
pub enum SenderError {
    /// The channel is closed due to receiver being dropped.
    #[error("Channel is closed")]
    Closed,
}

/// Sender object interacting with the mixing channel.
pub struct Sender<T> {
    channel: TrackedChannel<T>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let channel = self.channel.clone();
        channel.sender_count.fetch_add(1, Ordering::Relaxed);

        Sender { channel }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        if self.channel.sender_count.fetch_sub(1, Ordering::Relaxed) == 1
            && !self.channel.receiver_active.load(Ordering::Relaxed)
        {
            self.channel.channel.lock().waker = None;
        }
    }
}

impl<T> Sender<T> {
    /// Send one item to the mixing channel.
    pub fn send(&self, item: T) -> Result<(), SenderError> {
        self.push_item(item)
    }

    /// Locked critical section shared between `Sink::start_send` and [`Sender::send`].
    #[tracing::instrument(level = "trace", skip(self, item))]
    fn push_item(&self, item: T) -> Result<(), SenderError> {
        if !self.channel.receiver_active.load(Ordering::Relaxed) {
            return Err(SenderError::Closed);
        }

        let mut channel = self.channel.channel.lock();

        let random_delay = channel.cfg.random_delay();

        trace!(delay_in_ms = random_delay.as_millis(), "generated mixer delay",);

        let delayed_data: DelayedData<T> = (std::time::Instant::now() + random_delay, item).into();
        channel.buffer.push(Reverse(delayed_data));

        if let Some(waker) = channel.waker.as_ref() {
            waker.wake_by_ref();
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        {
            METRIC_QUEUE_SIZE.increment(1.0f64);

            let weight = 1.0f64 / channel.cfg.metric_delay_window as f64;
            METRIC_MIXER_AVERAGE_DELAY.set(
                (weight * random_delay.as_millis() as f64) + ((1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get()),
            );
        }

        Ok(())
    }
}

impl<T> futures::sink::Sink<T> for Sender<T> {
    type Error = SenderError;

    fn poll_ready(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        let is_active = self.channel.receiver_active.load(Ordering::Relaxed);
        if is_active {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(SenderError::Closed))
        }
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.push_item(item)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        // The channel can only be closed by the receiver. The sender can be dropped at any point.
        Poll::Ready(Ok(()))
    }
}

/// Error returned by the [`Receiver`].
#[derive(Debug, thiserror::Error)]
pub enum ReceiverError {
    /// The channel is closed due to receiver being dropped.
    #[error("Channel is closed")]
    Closed,
}

/// Receiver object interacting with the mixer channel.
///
/// The receiver receives already mixed elements without any knowledge of
/// the original order.
///
/// The release-deadline timer lives on the receiver itself (not behind the shared
/// mutex). Senders therefore never wait on this timer's poll — a sender can push into
/// the buffer while the receiver is parked on the timer.
pub struct Receiver<T> {
    channel: TrackedChannel<T>,
    /// Release-deadline timer, lazily created on first use so that constructing a channel does
    /// not require an active runtime. Backed by tokio's in-runtime timer wheel rather than an
    /// out-of-runtime timer thread, so waking at the release deadline does not incur a
    /// cross-thread unpark - this tightens release-time accuracy. It only governs *when* the
    /// receiver wakes to inspect the heap; the per-item random delay and release ordering
    /// (the mixing guarantee) are computed entirely by the sender and are unchanged.
    timer: Option<Pin<Box<Sleep>>>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    #[tracing::instrument(level = "trace", skip(self, cx))]
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let now = std::time::Instant::now();
        let no_senders = self.channel.sender_count.load(Ordering::Relaxed) == 0;

        // Phase 1: under lock, try to pop a due item; otherwise register the waker and
        // capture the duration to sleep. Drop the lock before touching the timer.
        //
        // When all senders have dropped (`no_senders`), we must still drain any items
        // remaining in the buffer before terminating the stream — returning `None`
        // with pending items in flight would silently discard packets the senders
        // enqueued before closing.
        let sleep_for = {
            let mut channel = self.channel.channel.lock();

            if channel.buffer.peek().map(|x| x.0.release_at < now).unwrap_or(false) {
                let data = channel
                    .buffer
                    .pop()
                    .expect("value must be present during the same locked access")
                    .0
                    .item;

                trace!(from = "direct", "yield item");

                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_QUEUE_SIZE.decrement(1.0f64);

                return Poll::Ready(Some(data));
            }

            // Buffer is either empty or every item is still in the future.
            if channel.buffer.is_empty() && no_senders {
                // Nothing left to deliver and nobody is going to enqueue more.
                drop(channel);
                self.channel.receiver_active.store(false, Ordering::Relaxed);
                return Poll::Ready(None);
            }

            match channel.waker.as_mut() {
                Some(existing) => existing.clone_from(cx.waker()),
                None => channel.waker = Some(cx.waker().clone()),
            }

            match channel.buffer.peek() {
                Some(next) => next.0.release_at.duration_since(now),
                None => {
                    // Senders still alive (else we would have returned `None` above) —
                    // wait for one of them to push.
                    trace!(from = "direct", "pending (empty buffer)");
                    return Poll::Pending;
                }
            }
        };

        // Phase 2: poll the timer WITHOUT holding the mutex so senders can keep pushing.
        let this = self.get_mut();
        let deadline = tokio::time::Instant::now() + sleep_for;
        trace!("resetting the timer");
        let timer = this
            .timer
            .get_or_insert_with(|| Box::pin(tokio::time::sleep_until(deadline)));
        timer.as_mut().reset(deadline);
        futures::ready!(timer.as_mut().poll(cx));

        // Phase 3: timer fired. Re-take the lock. Because there is only one receiver,
        // the item at the top is still present; senders can only have added more.
        // A newly-pushed item with an earlier deadline is also safe to pop — it merely
        // yields in a different order than originally expected, which is consistent
        // with the mixer's design of not preserving input order.
        let mut channel = this.channel.channel.lock();
        match channel.buffer.pop() {
            Some(entry) => {
                trace!(from = "timer", "yield item");

                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_QUEUE_SIZE.decrement(1.0f64);

                Poll::Ready(Some(entry.0.item))
            }
            None => {
                trace!(from = "timer", "buffer drained before we re-acquired the lock");
                Poll::Pending
            }
        }
    }
}

impl<T> Receiver<T> {
    /// Receive a single delayed mixed item.
    pub async fn recv(&mut self) -> Option<T> {
        poll_fn(|cx| self.poll_next_unpin(cx)).await
    }
}

/// Instantiate a mixing channel and return the sender and receiver end of the channel.
pub fn channel<T>(cfg: crate::config::MixerConfig) -> (Sender<T>, Receiver<T>) {
    #[cfg(all(feature = "telemetry", not(test)))]
    {
        // Initialize the lazy statics here
        lazy_static::initialize(&METRIC_QUEUE_SIZE);
        lazy_static::initialize(&METRIC_MIXER_AVERAGE_DELAY);
    }

    let mut buffer = BinaryHeap::new();
    buffer.reserve(cfg.capacity);

    let channel = TrackedChannel {
        channel: Arc::new(Mutex::new(Channel::<T> {
            buffer,
            waker: None,
            cfg,
        })),
        sender_count: Arc::new(AtomicUsize::new(1)),
        receiver_active: Arc::new(AtomicBool::new(true)),
    };
    (
        Sender {
            channel: channel.clone(),
        },
        Receiver { channel, timer: None },
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::{SinkExt, StreamExt};
    use tokio::time::timeout;

    use super::*;

    const PROCESSING_LEEWAY: Duration = Duration::from_millis(250);
    const MAXIMUM_SINGLE_DELAY_DURATION: Duration = Duration::from_millis(
        crate::config::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS + crate::config::HOPR_MIXER_DEFAULT_DELAY_RANGE_IN_MS,
    );

    #[tokio::test]
    async fn mixer_channel_should_pass_an_element() -> anyhow::Result<()> {
        let (tx, mut rx) = channel(MixerConfig::default());
        tx.send(1)?;
        assert_eq!(rx.recv().await, Some(1));

        Ok(())
    }

    #[tokio::test]
    async fn mixer_channel_should_introduce_random_delay() -> anyhow::Result<()> {
        let start = std::time::SystemTime::now();

        let (tx, mut rx) = channel(MixerConfig::default());
        tx.send(1)?;
        assert_eq!(rx.recv().await, Some(1));

        let elapsed = start.elapsed()?;

        assert!(elapsed < MAXIMUM_SINGLE_DELAY_DURATION + PROCESSING_LEEWAY);
        assert!(elapsed > Duration::from_millis(crate::config::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS));
        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_batch_on_sending_emulating_concurrency() -> anyhow::Result<()> {
        const ITERATIONS: usize = 10;

        let (tx, mut rx) = channel(MixerConfig::default());

        let start = std::time::SystemTime::now();

        for i in 0..ITERATIONS {
            tx.send(i)?;
        }
        for _ in 0..ITERATIONS {
            let data = timeout(MAXIMUM_SINGLE_DELAY_DURATION, rx.next()).await?;
            assert!(data.is_some());
        }

        let elapsed = start.elapsed()?;

        assert!(elapsed < MAXIMUM_SINGLE_DELAY_DURATION + PROCESSING_LEEWAY);
        assert!(elapsed > Duration::from_millis(crate::config::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS));
        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_work_concurrently_and_properly_closed_channels() -> anyhow::Result<()> {
        const ITERATIONS: usize = 1000;

        let (tx, mut rx) = channel(MixerConfig::default());

        let recv_task = tokio::task::spawn(async move {
            while let Some(_item) = timeout(2 * MAXIMUM_SINGLE_DELAY_DURATION, rx.next())
                .await
                .expect("receiver should not fail")
            {}
        });

        let send_task =
            tokio::task::spawn(async move { futures::stream::iter(0..ITERATIONS).map(Ok).forward(tx).await });

        let (_recv, send) = futures::try_join!(
            timeout(MAXIMUM_SINGLE_DELAY_DURATION, recv_task),
            timeout(MAXIMUM_SINGLE_DELAY_DURATION, send_task)
        )?;

        send??;

        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_produce_mixed_output_from_the_supplied_input_using_sync_send() -> anyhow::Result<()> {
        const ITERATIONS: usize = 20; // highly unlikely that this produces the same order on the input given the size

        let (tx, rx) = channel(MixerConfig::default());

        let input = (0..ITERATIONS).collect::<Vec<_>>();

        for i in input.iter() {
            tx.send(*i)?;
        }

        let mixed_output = timeout(
            2 * MAXIMUM_SINGLE_DELAY_DURATION,
            rx.take(ITERATIONS).collect::<Vec<_>>(),
        )
        .await?;

        tracing::info!(?input, ?mixed_output, "asserted data");
        assert_ne!(input, mixed_output);
        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_produce_mixed_output_from_the_supplied_input_using_async_send() -> anyhow::Result<()>
    {
        const ITERATIONS: usize = 20; // highly unlikely that this produces the same order on the input given the size

        let (mut tx, rx) = channel(MixerConfig::default());

        let input = (0..ITERATIONS).collect::<Vec<_>>();

        for i in input.iter() {
            SinkExt::send(&mut tx, *i).await?;
        }

        let mixed_output = timeout(
            2 * MAXIMUM_SINGLE_DELAY_DURATION,
            rx.take(ITERATIONS).collect::<Vec<_>>(),
        )
        .await?;

        tracing::info!(?input, ?mixed_output, "asserted data");
        assert_ne!(input, mixed_output);
        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_produce_mixed_output_from_the_supplied_input_using_async_feed() -> anyhow::Result<()>
    {
        const ITERATIONS: usize = 20; // highly unlikely that this produces the same order on the input given the size

        let (mut tx, rx) = channel(MixerConfig::default());

        let input = (0..ITERATIONS).collect::<Vec<_>>();

        for i in input.iter() {
            SinkExt::feed(&mut tx, *i).await?;
        }
        SinkExt::flush(&mut tx).await?;

        let mixed_output = timeout(
            2 * MAXIMUM_SINGLE_DELAY_DURATION,
            rx.take(ITERATIONS).collect::<Vec<_>>(),
        )
        .await?;

        tracing::info!(?input, ?mixed_output, "asserted data");
        assert_ne!(input, mixed_output);
        Ok(())
    }

    #[tokio::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_not_mix_the_order_if_the_min_delay_and_delay_range_is_0() -> anyhow::Result<()> {
        const ITERATIONS: usize = 40; // highly unlikely that this produces the same order on the input given the size

        let (tx, rx) = channel(MixerConfig {
            min_delay: Duration::from_millis(0),
            delay_range: Duration::from_millis(0),
            ..MixerConfig::default()
        });

        let input = (0..ITERATIONS).collect::<Vec<_>>();

        for i in input.iter() {
            tx.send(*i)?;
            tokio::time::sleep(std::time::Duration::from_micros(10)).await; // ensure we don't send too fast
        }

        let mixed_output = timeout(
            2 * MAXIMUM_SINGLE_DELAY_DURATION,
            rx.take(ITERATIONS).collect::<Vec<_>>(),
        )
        .await?;

        tracing::info!(?input, ?mixed_output, "asserted data");
        assert_eq!(input, mixed_output);

        Ok(())
    }

    #[tokio::test]
    async fn sender_should_return_closed_when_receiver_inactive() -> anyhow::Result<()> {
        let (tx, _rx) = channel::<i32>(MixerConfig::default());

        // Simulate the receiver marking itself inactive (normally happens
        // in poll_next when sender_count drops to 0).
        tx.channel.receiver_active.store(false, Ordering::Relaxed);

        let result = tx.send(42);
        assert!(
            matches!(result, Err(SenderError::Closed)),
            "send with inactive receiver should return Closed, got: {result:?}"
        );
        Ok(())
    }

    /// Regression test for #7947 item 2: the receiver must not hold the channel
    /// mutex across its timer poll, otherwise a sender trying to push a new item
    /// would block until the original timer deadline expired.
    #[tokio::test]
    async fn sender_can_push_while_receiver_is_parked_on_timer() -> anyhow::Result<()> {
        // Mixer with a long minimum delay — the receiver's first poll will park on the
        // timer for at least this duration.
        let cfg = MixerConfig {
            min_delay: Duration::from_millis(500),
            delay_range: Duration::from_millis(1),
            ..MixerConfig::default()
        };
        let (tx, mut rx) = channel::<u32>(cfg);

        // Prime: push one item so the receiver's timer branch activates.
        tx.send(0)?;

        // Drive the receiver far enough to install the timer for the first item, then
        // try to push again from another task. If the mutex were held across the timer
        // poll, this second send would block for ~500 ms.
        let rx_task = tokio::task::spawn(async move {
            // Collect both items back.
            let first = rx.next().await;
            let second = rx.next().await;
            (first, second)
        });

        // Give the receiver a moment to enter the timer branch (it will have locked,
        // seen the not-yet-due item, stored the waker, unlocked, and started polling
        // the release timer).
        tokio::time::sleep(Duration::from_millis(50)).await;

        // The crucial assertion: this `send()` must return promptly — not block for
        // ~450 ms. We measure the latency of the send itself.
        let send_started = std::time::Instant::now();
        tx.send(1)?;
        let send_latency = send_started.elapsed();
        assert!(
            send_latency < Duration::from_millis(100),
            "sender was blocked by receiver's timer poll: send took {send_latency:?}"
        );

        // Sanity: both items eventually arrive.
        let (first, second) = timeout(Duration::from_millis(1500), rx_task)
            .await?
            .expect("receiver task should finish");
        assert!(first.is_some(), "first item must be received");
        assert!(second.is_some(), "second item must be received");
        Ok(())
    }

    /// Regression test: if the last sender drops after enqueueing items, the receiver
    /// must drain them before closing. Pre-fix the receiver returned `None` as soon as
    /// `sender_count == 0`, silently discarding anything still in the buffer.
    #[tokio::test]
    async fn receiver_drains_buffered_items_after_last_sender_drops() -> anyhow::Result<()> {
        let cfg = MixerConfig {
            min_delay: Duration::from_millis(0),
            delay_range: Duration::from_millis(1),
            ..MixerConfig::default()
        };
        let (tx, mut rx) = channel::<u32>(cfg);

        const ITERATIONS: usize = 16;
        for i in 0..ITERATIONS as u32 {
            tx.send(i)?;
        }

        // Drop the only sender. sender_count goes to 0, but buffer still has ITERATIONS items.
        drop(tx);

        // Must still yield every item before terminating.
        let mut received = 0usize;
        while let Some(_item) = timeout(Duration::from_millis(500), rx.next()).await? {
            received += 1;
        }
        assert_eq!(
            received,
            ITERATIONS,
            "receiver dropped {} pending items on sender shutdown",
            ITERATIONS - received
        );
        Ok(())
    }

    /// Regression test: all `Sender` clones push into the same shared heap,
    /// so items from different clones are drained by the single `Receiver`.
    ///
    /// Pre-fix (`MixerSink::clone` with per-clone heap): each clone has its own
    /// `BinaryHeap`; a separate receiver per clone would be required to observe
    /// all items. With a single receiver the other clone's items are silently lost.
    ///
    /// Post-fix (`Sender`/`Receiver` channel): one shared heap, one receiver
    /// sees every item regardless of which clone pushed it.
    #[tokio::test]
    async fn sender_clones_share_heap() -> anyhow::Result<()> {
        let cfg = MixerConfig {
            min_delay: Duration::from_millis(50),
            delay_range: Duration::from_millis(1),
            ..MixerConfig::default()
        };
        let (tx_a, mut rx) = channel::<u32>(cfg);
        let tx_b = tx_a.clone();

        tx_a.send(1)?;
        tx_b.send(2)?;
        drop(tx_a);
        drop(tx_b);

        // Single receiver must drain both items — proves the heap is shared.
        let mut got = vec![
            timeout(MAXIMUM_SINGLE_DELAY_DURATION + PROCESSING_LEEWAY, rx.next())
                .await?
                .expect("first item"),
            timeout(MAXIMUM_SINGLE_DELAY_DURATION + PROCESSING_LEEWAY, rx.next())
                .await?
                .expect("second item"),
        ];
        got.sort();
        assert_eq!(got, vec![1, 2]);
        assert!(rx.next().await.is_none(), "expected channel closed with no more items");
        Ok(())
    }
}
