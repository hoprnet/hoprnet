use futures::FutureExt;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

#[derive(Debug)]
struct DelayedEntry<T> {
    item: T,
    at: Instant,
    cancelled: AtomicBool,
}

impl<T: PartialEq> PartialEq for DelayedEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<T: Eq> Eq for DelayedEntry<T> {}

impl<T: Ord> Ord for DelayedEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.item != self.item {
            match self.at.cmp(&other.at) {
                Ordering::Equal => self.item.cmp(&other.item),
                x => x,
            }
        } else {
            Ordering::Equal
        }
    }
}

impl<T: Ord> PartialOrd for DelayedEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A queue of [items](DelayedItem) with attached [`Instant`] that determines a deadline
/// at which it should be yielded from the [`Stream`](futures::Stream) side of the queue.
/// The queue also has the cancellation ability: an item that has been pushed into the
/// queue earlier can be canceled before it meets its deadline.
/// A canceled item will then be skipped in the output stream.
///
/// The items are internally sorted based on their deadline.
/// In a case when two items have equal deadlines, they are sorted according
/// to their values; therefore, items must implement [`Ord`].
pub struct SkipDelayQueue<T> {
    entries: BTreeSet<DelayedEntry<T>>,
    next_wakeup: Option<futures_time::task::SleepUntil>,
    rx_waker: Option<Waker>,
    is_closed: bool,
}

/// An item with deadline, which can be pushed into the [`SkipDelayQueue`].
///
/// For convenience, the type implements From traits from
/// `(T, Instant)`, `(T, Duration)` and `(T, Skip)`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DelayedItem<T> {
    /// Adds a new item with a deadline.
    New(T, Instant),
    /// Cancel a previously added item.
    Cancel(T),
}

/// A marker type for canceling items pushed into the [`SkipDelayQueue`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Skip;

impl<T> From<(T, Duration)> for DelayedItem<T> {
    fn from(value: (T, Duration)) -> Self {
        Self::New(value.0, Instant::now() + value.1)
    }
}

impl<T> From<(T, Instant)> for DelayedItem<T> {
    fn from(value: (T, Instant)) -> Self {
        Self::New(value.0, value.1)
    }
}

impl<T> From<(T, Skip)> for DelayedItem<T> {
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
                    self.next_wakeup = Some(futures_time::task::sleep_until(e.at.into()));
                    cx.waker().wake_by_ref();
                    Poll::Pending
                };
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

impl<T: Ord + std::fmt::Debug> futures::Sink<DelayedItem<T>> for SkipDelayQueue<T> {
    type Error = std::io::Error;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.is_closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: DelayedItem<T>) -> Result<(), Self::Error> {
        if self.is_closed {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        }

        match item {
            DelayedItem::New(item, at) => {
                tracing::trace!("SkipDelayQueue::start_send inserting");
                self.entries.insert(DelayedEntry {
                    item,
                    at,
                    cancelled: AtomicBool::new(false),
                });
            }
            DelayedItem::Cancel(item) => {
                tracing::trace!("SkipDelayQueue::start_send cancelling");
                self.entries
                    .iter()
                    .filter(|e| item == e.item)
                    .for_each(|e| e.cancelled.store(true, std::sync::atomic::Ordering::SeqCst));
            }
        }

        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.is_closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        tracing::trace!("SkipDelayQueue::poll_flush");
        if let Some(waker) = self.rx_waker.take() {
            waker.wake();
        }

        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.is_closed {
            return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
        }

        tracing::trace!("SkipDelayQueue::poll_close");
        self.is_closed = true;
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{pin_mut, SinkExt, StreamExt};

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_yield_items() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now + Duration::from_millis(100)).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(100));
        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_yielded_items_should_be_apart() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now1 = Instant::now();
        tx.send((1, now1 + Duration::from_millis(100)).into()).await?;
        let now2 = Instant::now();
        tx.send((2, now2 + Duration::from_millis(200)).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert!(now1.elapsed() >= Duration::from_millis(100));
        assert_eq!(Some(2), rx.next().await);
        assert!(now2.elapsed() >= Duration::from_millis(200));

        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_not_yield_cancelled_items() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now + Duration::from_millis(100)).into()).await?;
        tx.send((1, Skip).into()).await?;
        tx.close().await?;

        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_yield_past_items_immediately() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now).into()).await?;
        tx.send((2, now).into()).await?;
        tx.close().await?;

        let now = Instant::now();
        assert_eq!(Some(1), rx.next().await);
        assert_eq!(Some(2), rx.next().await);
        assert_eq!(None, rx.next().await);

        assert!(now.elapsed() < Duration::from_millis(25));

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_not_yield_future_cancelled_items() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now).into()).await?;
        tx.send((2, now + Duration::from_millis(100)).into()).await?;
        tx.send((2, Skip).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert_eq!(None, rx.next().await);
        assert!(now.elapsed() < Duration::from_millis(50));

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_discard_duplicate_entries() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now).into()).await?;
        tx.send((1, now).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_yield_items_in_order() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((2, now).into()).await?;
        tx.send((1, now).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert_eq!(Some(2), rx.next().await);
        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_yield_fed_items_in_order() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();
        pin_mut!(rx);

        let now = Instant::now();
        tx.feed((2, now).into()).await?;
        tx.feed((1, now).into()).await?;
        tx.flush().await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert_eq!(Some(2), rx.next().await);
        assert_eq!(None, rx.next().await);

        Ok(())
    }


    #[test_log::test(async_std::test)]
    async fn skip_delay_queue_should_continuously_yield_items() -> anyhow::Result<()> {
        let (mut tx, rx) = SkipDelayQueue::new().split();

        let items = [5,2,1,4,3];

        let now = Instant::now();
        let timed_items = (0..5)
            .map(|i| (items[i], now + Duration::from_millis(200) * (i as u32)))
            .collect::<Vec<_>>();

        let timed_items_clone = timed_items.clone();
        let jh = async_std::task::spawn(async move {
            for (n, time) in timed_items_clone {
                tx.send((n, time).into()).await?;
                async_std::task::sleep(Duration::from_millis(100)).await;
            }
            tx.close().await?;
            Ok::<_, std::io::Error>(())
        });

        let collected = rx.map(|item| (item, Instant::now())).collect::<Vec<_>>().await;

        assert_eq!(timed_items.len(), collected.len());

        for (i, (item, received_at)) in collected.into_iter().enumerate() {
            assert_eq!(timed_items[i].0, item);
            if received_at < timed_items[i].1 {
                assert!(timed_items[i].1.saturating_duration_since(received_at) < Duration::from_millis(20));
            } else {
                assert!(received_at.saturating_duration_since(timed_items[i].1) < Duration::from_millis(20));
            }
        }

        jh.await?;
        Ok(())
    }
}
