use futures::Future;
use futures_timer::Delay;
use std::{
    collections::BinaryHeap,
    future::poll_fn,
    sync::{Arc, Mutex},
    task::Poll,
    time::Duration,
};

use crate::data::DelayedData;

pub const HOPR_MIXER_CAPACITY: usize = 10000;

/// Mixing and delaying channel usinng the random delay function.
///
/// Mixing is performed by assigning random delays to the ingress timestamp of the data,
/// then storing the value inside the
struct Channel<T> {
    /// Buffer holding the data with a timestamp ordering to ensure the min heap behavior.
    buffer: BinaryHeap<DelayedData<T>>,
    timer: futures_timer::Delay,
    waker: Option<std::task::Waker>,
    sender_count: usize,
    receiver_active: bool,
}

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
    pub fn send(&self, value: T) -> Result<(), ()> {
        if let Ok(mut channel) = self.channel.lock() {
            let random_delay = crate::delay::random_delay();
            let delayed_data: DelayedData<T> = (std::time::SystemTime::now() + random_delay, value).into();
            if channel.receiver_active {
                tracing::info!("=> delay: {:?}", random_delay.as_millis());
                channel.buffer.push(delayed_data);

                if let Some(waker) = channel.waker.as_ref() {
                    waker.wake_by_ref();
                }

                return Ok(());
            }
        }

        Err(())
    }
}

pub struct Receiver<T> {
    channel: Arc<Mutex<Channel<T>>>,
}

impl<T> Receiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        return poll_fn(|cx| {
            let mut channel = self.channel.lock().unwrap();
            let now = std::time::SystemTime::now();
            if channel.sender_count > 0 {
                let next = channel.buffer.peek();
                if channel.buffer.peek().map(|x| x.release < now).unwrap_or(false) {
                    let data = channel
                        .buffer
                        .pop()
                        .expect("The value should be present within the locked access")
                        .data;

                    return Poll::Ready(Some(data));
                }

                let waker = cx.waker().clone();
                channel.waker = Some(waker);

                if let Some(next) = channel.buffer.peek() {
                    if let Ok(remaining) = next.release.duration_since(now) {
                        channel.timer.reset(remaining);
                        let timer = std::pin::pin!(&mut channel.timer);
                        let res = timer.poll(cx);
                        return match res {
                            Poll::Ready(_) => Poll::Ready(Some(
                                channel
                                    .buffer
                                    .pop()
                                    .expect("The value should be present within the locked access")
                                    .data,
                            )),
                            Poll::Pending => Poll::Pending,
                        };
                    }
                }

                Poll::Pending
            } else {
                channel.receiver_active = false;
                Poll::Ready(None)
            }
        })
        .await;
    }
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let mut buffer = BinaryHeap::new();
    buffer.reserve(HOPR_MIXER_CAPACITY);

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
    use super::*;

    #[async_std::test]
    async fn mixer_channel_should_pass_an_element() -> anyhow::Result<()> {
        let (tx, mut rx) = channel();
        tx.send(1).expect("Sender should be constructible");
        assert_eq!(rx.recv().await, Some(1));

        Ok(())
    }

    #[async_std::test]
    async fn mixer_channel_should_introduce_random_delay() -> anyhow::Result<()> {
        let start = std::time::SystemTime::now();

        let (tx, mut rx) = channel();
        tx.send(1).expect("Sender should be constructible");
        assert_eq!(rx.recv().await, Some(1));

        let elapsed = start.elapsed()?;

        assert!(
            elapsed
                < Duration::from_millis(
                    crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS
                        + crate::delay::HOPR_MIXER_DEFAULT_DELAY_DIFFERENCE_IN_MS
                )
        );
        Ok(assert!(
            elapsed > Duration::from_millis(crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS)
        ))
    }

    #[async_std::test]
    #[tracing_test::traced_test]
    async fn mixer_channel_should_batch_on_sending_emulating_concurrency() -> anyhow::Result<()> {
        const ITERATIONS: usize = 10;

        let (tx, mut rx) = channel();

        let start = std::time::SystemTime::now();

        for i in 0..ITERATIONS {
            tx.send(i).expect("Sender should be constructible");
        }
        for _ in 0..ITERATIONS {
            let data = rx.recv().await;
            tracing::info!("<= data: {:?}", data);
            assert!(data.is_some());
        }

        let elapsed = start.elapsed()?;

        assert!(
            elapsed
                < Duration::from_millis(
                    crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS
                        + crate::delay::HOPR_MIXER_DEFAULT_DELAY_DIFFERENCE_IN_MS
                )
        );
        Ok(assert!(
            elapsed > Duration::from_millis(crate::delay::HOPR_MIXER_MINIMUM_DEFAULT_DELAY_IN_MS)
        ))
    }

    const EMPTY_DATA: [u8; 500] = [5; 500];

    #[async_std::test]
    #[tracing_test::traced_test]
    async fn bench_sending_10000_messages() -> anyhow::Result<()> {
        const ITERATIONS: usize = 2 * 1000;
        let mut data = Vec::new();

        for _ in 0..ITERATIONS {
            data.push(Box::new(EMPTY_DATA));
        }

        tracing::info!("=> Data generated");

        let (tx, mut rx) = channel();

        let start = std::time::SystemTime::now();

        for i in data.into_iter() {
            tx.send(i).expect("Sender should be constructible");
        }
        for _ in 0..ITERATIONS {
            assert!(rx.recv().await.is_some());
        }

        let elapsed = start.elapsed()?;

        assert_eq!(elapsed.as_millis(), 10000);

        Ok(())
    }
}
