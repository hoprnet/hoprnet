use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    future::poll_fn,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::Poll,
    time::Duration,
};

use futures::{FutureExt, SinkExt, Stream, StreamExt};
use futures_timer::Delay;
use tracing::{error, trace};

use crate::{config::MixerConfig, data::DelayedData};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    pub static ref METRIC_QUEUE_SIZE: hopr_metrics::SimpleGauge =
        hopr_metrics::SimpleGauge::new("hopr_mixer_queue_size", "Current mixer queue size").unwrap();
    pub static ref METRIC_MIXER_AVERAGE_DELAY: hopr_metrics::SimpleGauge = hopr_metrics::SimpleGauge::new(
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
/// The channel uses a single timer thread that is instantiated on the first
/// timer reset and shared across all operations. This channel is **unbounded** by nature
/// using the `capacity` in the configuration to solely pre-allocate the buffer.
struct Channel<T> {
    /// Buffer holding the data with a timestamp ordering to ensure the min heap behavior.
    buffer: BinaryHeap<Reverse<DelayedData<T>>>,
    timer: futures_timer::Delay,
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

    /// The mutex lock over the channel failed.
    #[error("Channel lock failed")]
    Lock,
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
            let mut channel = self.channel.channel.lock().unwrap_or_else(|e| {
                self.channel.channel.clear_poison();
                e.into_inner()
            });

            channel.waker = None;
        }
    }
}

impl<T> Sender<T> {
    /// Send one item to the mixing channel.
    pub fn send(&self, item: T) -> Result<(), SenderError> {
        let mut sender = self.clone();
        sender.start_send_unpin(item)
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

    #[tracing::instrument(level = "trace", skip(self, item))]
    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let is_active = self.channel.receiver_active.load(Ordering::Relaxed);

        if is_active {
            let mut channel = self.channel.channel.lock().map_err(|_| SenderError::Lock)?;

            let random_delay = channel.cfg.random_delay();

            trace!(delay_in_ms = random_delay.as_millis(), "generated mixer delay",);

            let delayed_data: DelayedData<T> = (std::time::Instant::now() + random_delay, item).into();
            channel.buffer.push(Reverse(delayed_data));

            if let Some(waker) = channel.waker.as_ref() {
                waker.wake_by_ref();
            }

            #[cfg(all(feature = "prometheus", not(test)))]
            {
                METRIC_QUEUE_SIZE.increment(1.0f64);

                let weight = 1.0f64 / channel.cfg.metric_delay_window as f64;
                METRIC_MIXER_AVERAGE_DELAY.set(
                    (weight * random_delay.as_millis() as f64) + ((1.0f64 - weight) * METRIC_MIXER_AVERAGE_DELAY.get()),
                );
            }

            Ok(())
        } else {
            Err(SenderError::Closed)
        }
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

    /// The mutex lock over the channel failed.
    #[error("Channel lock failed")]
    Lock,
}

/// Receiver object interacting with the mixer channel.
///
/// The receiver receives already mixed elements without any knowledge of
/// the original order.
pub struct Receiver<T> {
    channel: TrackedChannel<T>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    #[tracing::instrument(level = "trace", skip(self, cx))]
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let now = std::time::Instant::now();
        if self.channel.sender_count.load(Ordering::Relaxed) > 0 {
            let Ok(mut channel) = self.channel.channel.lock() else {
                error!("mutex is poisoned, terminating stream");
                return Poll::Ready(None);
            };

            if channel.buffer.peek().map(|x| x.0.release_at < now).unwrap_or(false) {
                let data = channel
                    .buffer
                    .pop()
                    .expect("The value should be present within the same locked access")
                    .0
                    .item;

                trace!(from = "direct", "yield item");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_QUEUE_SIZE.decrement(1.0f64);

                return Poll::Ready(Some(data));
            }

            if let Some(waker) = channel.waker.as_mut() {
                waker.clone_from(cx.waker());
            } else {
                let waker = cx.waker().clone();
                channel.waker = Some(waker);
            }

            if let Some(next) = channel.buffer.peek() {
                let remaining = next.0.release_at.duration_since(now);

                trace!("reseting the timer");
                channel.timer.reset(remaining);

                futures::ready!(channel.timer.poll_unpin(cx));

                trace!(from = "timer", "yield item");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_QUEUE_SIZE.decrement(1.0f64);

                return Poll::Ready(Some(
                    channel
                        .buffer
                        .pop()
                        .expect("The value should be present within the locked access")
                        .0
                        .item,
                ));
            }

            trace!(from = "direct", "pending");
            Poll::Pending
        } else {
            self.channel.receiver_active.store(false, Ordering::Relaxed);
            Poll::Ready(None)
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
    #[cfg(all(feature = "prometheus", not(test)))]
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
            timer: Delay::new(Duration::from_secs(0)),
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
        Receiver { channel },
    )
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use tokio::time::timeout;

    use super::*;

    const PROCESSING_LEEWAY: Duration = Duration::from_millis(20);
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
}
