use std::collections::BTreeSet;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use futures::FutureExt;

#[derive(Debug)]
struct DelayEntryWrapper<T> {
    item: T,
    at: Instant,
    cancelled: AtomicBool,
}

impl<T: PartialEq> PartialEq for DelayEntryWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.at == other.at
    }
}

impl<T: Eq> Eq for DelayEntryWrapper<T> {}

impl<T: Ord> Ord for DelayEntryWrapper<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.at.cmp(&other.at) {
            std::cmp::Ordering::Equal => self.item.cmp(&other.item),
            x => x,
        }
    }
}

impl<T: Ord> PartialOrd for DelayEntryWrapper<T>{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct SkipDelayQueue<T> {
    entries: BTreeSet<DelayEntryWrapper<T>>,
    next_wakeup: Option<futures_time::task::Sleep>,
    rx_waker: Option<Waker>,
    is_closed: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DelayedEntry<T> {
    New(T, Duration),
    Cancel(T)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Skip;

impl<T> From<(T, Duration)> for DelayedEntry<T> {
    fn from(value: (T, Duration)) -> Self {
        Self::New(value.0, value.1)
    }
}

impl<T> From<(T, Instant)> for DelayedEntry<T> {
    fn from(value: (T, Instant)) -> Self {
        Self::New(value.0, value.1.saturating_duration_since(Instant::now()))
    }
}

impl<T> From<(T, Skip)> for DelayedEntry<T> {
    fn from(value: (T, Skip)) -> Self {
        Self::Cancel(value.0)
    }
}

impl<T> SkipDelayQueue<T> {
    const TOLERANCE: Duration = Duration::from_millis(10);

    pub fn new() -> Self {
        Self {
            entries: BTreeSet::new(),
            next_wakeup: None,
            rx_waker: None,
            is_closed: false,
        }
    }
}

impl<T: Ord> futures::Stream for SkipDelayQueue<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        tracing::trace!("SkipDelayQueue::poll_next");

        // Wait until the timer's done, if any
        if let Some(next_wakeup) = self.next_wakeup.as_mut() {
            tracing::trace!("SkipDelayQueue::poll_next polling timer");
            let _ = futures::ready!(next_wakeup.poll_unpin(cx));
            self.next_wakeup = None;
        }

        tracing::trace!("SkipDelayQueue::poll_next timer finished");

        let now = Instant::now();
        while let Some(e) = self.entries.first() {
            if !e.cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                return if e.at.saturating_duration_since(now) < Self::TOLERANCE {
                    // If the item is already in the past, yield it
                    tracing::trace!("SkipDelayQueue::poll_next ready");
                    Poll::Ready(self.entries.pop_first().map(|e| e.item))
                } else {
                    // The next item is in the future, set up the timer and wake us up to start it
                    tracing::trace!("SkipDelayQueue::poll_next pending new timer");
                    self.next_wakeup = Some(futures_time::task::sleep(e.at.saturating_duration_since(now).into()));
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            } else {
                // If the item has been canceled, remove it and continue
                self.entries.pop_first();
                tracing::trace!("SkipDelayQueue::poll_next item cancelled");
            }
        }

        if !self.is_closed {
            // Need more data, wake us up when some are added
            tracing::trace!("SkipDelayQueue::poll_next pending for data");
            self.rx_waker = Some(cx.waker().clone());
            Poll::Pending
        } else {
            // We're done
            tracing::trace!("SkipDelayQueue::poll_next done");
            Poll::Ready(None)
        }
    }
}

impl<T: Ord> futures::Sink<DelayedEntry<T>> for SkipDelayQueue<T> {
    type Error = std::io::Error;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.is_closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()))
        }
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: DelayedEntry<T>) -> Result<(), Self::Error> {
        if self.is_closed {
            return Err(std::io::ErrorKind::BrokenPipe.into())
        }

        match item {
            DelayedEntry::New(item,d) => {
                tracing::trace!("SkipDelayQueue::start_send inserting");
                let at = Instant::now() + d;
                self.entries.insert(DelayEntryWrapper {
                    item,
                    at,
                    cancelled: AtomicBool::new(false),
                });
                if self.next_wakeup.is_none() {
                    tracing::trace!("SkipDelayQueue::start_send set new timer");
                    self.next_wakeup = Some(futures_time::task::sleep(d.into()));
                }
            },
            DelayedEntry::Cancel(item) => {
                tracing::trace!("SkipDelayQueue::start_send cancelling");
                self.entries
                    .iter()
                    .filter(|e| item == e.item)
                    .for_each(|e| e.cancelled.store(true, std::sync::atomic::Ordering::SeqCst));
            }
        }

        if let Some(waker) = self.rx_waker.take() {
            waker.wake();
        }

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.is_closed = true;
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{pin_mut, SinkExt, StreamExt};

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_yield_items() -> anyhow::Result<()> {
        let (mut tx,rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, Duration::from_millis(100)).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(100));
        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_yielded_items_should_be_apart() -> anyhow::Result<()> {
        let (mut tx,rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now + Duration::from_millis(100)).into()).await?;
        tx.send((2, now + Duration::from_millis(200)).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(100));
        let now = Instant::now();
        assert_eq!(Some(2), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(100));

        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_not_yield_cancelled_items() -> anyhow::Result<()> {
        let (mut tx,rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, Duration::from_millis(100)).into()).await?;
        tx.send((1, Skip).into()).await?;
        tx.close().await?;

        assert_eq!(None, rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(100));

        Ok(())
    }
}