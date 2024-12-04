use futures::Future;
use futures_timer::Delay;
use std::{
    collections::BinaryHeap,
    future::poll_fn,
    sync::{Arc, Mutex},
    task::Poll,
    time::{Duration, Instant},
};

use crate::data::DelayedData;

pub const HOPR_MIXER_CAPACITY: usize = 10000;

struct Channel<T> {
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
            if channel.receiver_active {
                channel
                    .buffer
                    .push((std::time::SystemTime::now() + crate::delay::random_delay(), value).into());

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
            if channel.sender_count > 0 {
                if channel
                    .buffer
                    .peek()
                    .map(|x| x.release < std::time::SystemTime::now())
                    .unwrap_or(false)
                {
                    return Poll::Ready(Some(
                        channel
                            .buffer
                            .pop()
                            .expect("The value should be present within the locked access")
                            .data,
                    ));
                }

                let waker = cx.waker().clone();
                channel.waker = Some(waker);

                if let Some(next) = channel.buffer.peek() {
                    if let Ok(remaining) = next.release.duration_since(std::time::SystemTime::now()) {
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
    let channel = Arc::new(Mutex::new(Channel::<T> {
        buffer: BinaryHeap::new(),
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
    async fn channels_should_pass_an_element() -> anyhow::Result<()> {
        let (tx, mut rx) = channel();
        tx.send(1).expect("Sender should be constructible");
        assert_eq!(rx.recv().await, Some(1));

        Ok(())
    }

    #[async_std::test]
    async fn channels_should_introduce_random_delay() -> anyhow::Result<()> {
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
}
