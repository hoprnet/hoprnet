use futures::{Future, SinkExt, Stream, StreamExt};
use futures_timer::Delay;
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    future::poll_fn,
    sync::{Arc, Mutex},
    task::Poll,
    time::Duration,
};
use tracing::trace;

use crate::data::DelayedData;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleGauge;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_QUEUE_SIZE: SimpleGauge =
        SimpleGauge::new("hopr_mixer_queue_size", "Current mixer queue size").unwrap();
    static ref METRIC_MIXER_AVERAGE_DELAY: SimpleGauge = SimpleGauge::new(
        "hopr_mixer_average_packet_delay",
        "Average mixer packet delay averaged over a packet window"
    )
    .unwrap();
}

pub const METRIC_DELAY_WINDOW: usize = 100;
pub const HOPR_MIXER_CAPACITY: usize = 10000;

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
/// timer reset and shared across all operations.
struct Channel<T> {
    /// Buffer holding the data with a timestamp ordering to ensure the min heap behavior.
    buffer: BinaryHeap<Reverse<DelayedData<T>>>,
    timer: futures_timer::Delay,
    waker: Option<std::task::Waker>,
    sender_count: usize,
    receiver_active: bool,
}

/// Error returned by the [`Sender`].
#[derive(Debug, thiserror::Error)]
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
    channel: Arc<Mutex<Channel<T>>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut channel = self.channel.lock().unwrap();
        channel.sender_count += 1;

        Sender {
            channel: self.channel.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut channel = self.channel.lock().unwrap();
        channel.sender_count -= 1;
        if channel.sender_count == 0 && !channel.receiver_active {
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
        let is_active = self.channel.lock().map_err(|_| SenderError::Lock)?.receiver_active;
        if is_active {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(SenderError::Closed))
        }
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let mut channel = self.channel.lock().map_err(|_| SenderError::Lock)?;
        if channel.receiver_active {
            let random_delay = crate::delay::random_delay();

            trace!(delay_in_ms = random_delay.as_millis(), "generated mixer delay",);

            let delayed_data: DelayedData<T> = (std::time::SystemTime::now() + random_delay, item).into();
            channel.buffer.push(Reverse(delayed_data));

            if let Some(waker) = channel.waker.as_ref() {
                waker.wake_by_ref();
            }

            #[cfg(all(feature = "prometheus", not(test)))]
            {
                METRIC_QUEUE_SIZE.increment(1.0f64);

                let weight = 1.0f64 / METRIC_DELAY_WINDOW as f64;
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
    channel: Arc<Mutex<Channel<T>>>,
}

impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let mut channel = self.channel.lock().unwrap();
        let now = std::time::SystemTime::now();
        if channel.sender_count > 0 {
            if channel.buffer.peek().map(|x| x.0.release_at < now).unwrap_or(false) {
                let data = channel
                    .buffer
                    .pop()
                    .expect("The value should be present within the same locked access")
                    .0
                    .item;

                trace!("yielding a value");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_QUEUE_SIZE.decrement(1.0f64);

                return Poll::Ready(Some(data));
            }

            let waker = cx.waker().clone();
            channel.waker = Some(waker);

            if let Some(next) = channel.buffer.peek() {
                if let Ok(remaining) = next.0.release_at.duration_since(now) {
                    trace!("reseting the timer");
                    channel.timer.reset(remaining);
                    let timer = std::pin::pin!(&mut channel.timer);
                    let res = timer.poll(cx);

                    return match res {
                        Poll::Ready(_) => {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_QUEUE_SIZE.decrement(1.0f64);

                            Poll::Ready(Some(
                                channel
                                    .buffer
                                    .pop()
                                    .expect("The value should be present within the locked access")
                                    .0
                                    .item,
                            ))
                        }
                        Poll::Pending => Poll::Pending,
                    };
                } else {
                    unreachable!("the previous block would've yielded the value");
                }
            }

            Poll::Pending
        } else {
            channel.receiver_active = false;
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
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    #[cfg(all(feature = "prometheus", not(test)))]
    {
        // Initialize the lazy statics here
        lazy_static::initialize(&METRIC_QUEUE_SIZE);
        lazy_static::initialize(&METRIC_MIXER_AVERAGE_DELAY);
    }

    let mut buffer = BinaryHeap::new();
    let mixer_capacity = std::env::var("HOPR_INTERNAL_MIXER_CAPACITY")
        .map(|v| v.trim().parse::<usize>().unwrap_or(10_000))
        .unwrap_or(10_000);
    buffer.reserve(mixer_capacity);

    let channel = Arc::new(Mutex::new(Channel::<T> {
        buffer,
        timer: Delay::new(Duration::from_secs(0)),
        waker: None,
        sender_count: 1,
        receiver_active: true,
    }));
    (
        Sender {
            channel: channel.clone(),
        },
        Receiver { channel },
    )
}

#[cfg(test)]
mod tests {
    use async_std::prelude::FutureExt;
    use futures::StreamExt;

    use super::*;

    const PROCESSING_LEEWAY: Duration = Duration::from_millis(20);

    #[async_std::test]
    async fn mixer_channel_should_pass_an_element() -> anyhow::Result<()> {
        let (tx, mut rx) = channel();
        tx.send(1)?;
        assert_eq!(rx.recv().await, Some(1));

        Ok(())
    }

    #[async_std::test]
    async fn mixer_channel_should_introduce_random_delay() -> anyhow::Result<()> {
        let start = std::time::SystemTime::now();

        let (tx, mut rx) = channel();
        tx.send(1)?;
        assert_eq!(rx.recv().await, Some(1));

        let elapsed = start.elapsed()?;

        assert!(
            elapsed
                < Duration::from_millis(
                    crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS
                        + crate::delay::HOPR_MIXER_DEFAULT_DELAY_DIFFERENCE_IN_MS
                ) + PROCESSING_LEEWAY
        );
        Ok(assert!(
            elapsed > Duration::from_millis(crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS)
        ))
    }

    #[async_std::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_batch_on_sending_emulating_concurrency() -> anyhow::Result<()> {
        const ITERATIONS: usize = 10;

        let (tx, mut rx) = channel();

        let start = std::time::SystemTime::now();

        for i in 0..ITERATIONS {
            tx.send(i)?;
        }
        for _ in 0..ITERATIONS {
            let data = rx.next().await;
            assert!(data.is_some());
        }

        let elapsed = start.elapsed()?;

        assert!(
            elapsed
                < Duration::from_millis(
                    crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS
                        + crate::delay::HOPR_MIXER_DEFAULT_DELAY_DIFFERENCE_IN_MS
                ) + PROCESSING_LEEWAY
        );
        Ok(assert!(
            elapsed > Duration::from_millis(crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS)
        ))
    }

    #[async_std::test]
    // #[tracing_test::traced_test]
    async fn mixer_channel_should_produce_mixed_output_from_the_supplied_input() -> anyhow::Result<()> {
        const ITERATIONS: usize = 20; // highly unlikely that this produces the same order on the input given the size

        let (tx, rx) = channel();

        let input = (0..ITERATIONS).collect::<Vec<_>>();

        for i in input.iter() {
            tx.send(*i)?;
        }

        let mixed_output = rx
            .take(ITERATIONS)
            .collect::<Vec<_>>()
            .timeout(std::time::Duration::from_millis(400))
            .await?;

        tracing::info!(?input, ?mixed_output, "asserted data");
        Ok(assert_ne!(input, mixed_output))
    }
}
