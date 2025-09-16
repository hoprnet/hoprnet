use std::{
    cmp::Ordering,
    collections::BTreeSet,
    pin::Pin,
    sync::{Arc, atomic::AtomicBool},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use futures::FutureExt;
use tracing::instrument;

/// An internal type used by the [`SkipDelayQueue`].
#[derive(Debug)]
struct DelayedEntry<T> {
    item: T,
    at: Instant,
    cancelled: AtomicBool,
}

// The entries are equal only if the items they carry are equal
impl<T: PartialEq> PartialEq for DelayedEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<T: Eq> Eq for DelayedEntry<T> {}

impl<T: Ord> Ord for DelayedEntry<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if other.item != self.item {
            // If items are not equal, the order is determined by the deadline
            match self.at.cmp(&other.at) {
                // If the deadlines are equal, use the natural order of the items.
                // This should be presumably consistent with their PartialEq and won't
                // therefore return Ordering::Equal.
                Ordering::Equal => self.item.cmp(&other.item),
                x => x,
            }
        } else {
            // Be consistent with PartialEq
            Ordering::Equal
        }
    }
}

impl<T: Ord> PartialOrd for DelayedEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Internal type used by the [`skip_delay_channel`].
struct SkipDelayQueue<T> {
    entries: BTreeSet<DelayedEntry<T>>,
    next_wakeup: Option<futures_time::task::SleepUntil>,
    rx_waker: Option<Waker>,
    is_closed: bool,
}

/// An item with a deadline, which can be pushed into the [`SkipDelayQueue`].
///
/// For convenience, the type implements From api-traits from
/// `(T, Instant)`, `(T, Duration)` and `(T, Skip)`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DelayedItem<T> {
    /// Adds new (or replaces an existing) item with a deadline.
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
    const TOLERANCE: Duration = Duration::from_millis(5);

    /// Creates a new instance.
    ///
    /// As a common practice, [`futures::StreamExt::split`] can be called to
    /// get separate sending and receiving part of the queue.
    pub fn new() -> Self {
        Self {
            entries: BTreeSet::new(),
            next_wakeup: None,
            rx_waker: None,
            is_closed: false,
        }
    }
}

/// Receiver part for the [`skip_delay_channel`].
pub struct SkipDelayReceiver<T>(Arc<std::sync::Mutex<SkipDelayQueue<T>>>);

impl<T> Drop for SkipDelayReceiver<T> {
    #[instrument(name = "SkipDelayReceiver::drop", level = "trace", skip(self))]
    fn drop(&mut self) {
        // When the receiver is dropped, clear the poison and mark the queue as closed.
        self.0.clear_poison();
        let mut queue = self.0.lock().expect("cannot panic because poison is cleared");
        queue.is_closed = true;
        queue.rx_waker = None;
    }
}

impl<T: Ord> futures::Stream for SkipDelayReceiver<T> {
    type Item = T;

    #[instrument(name = "SkipDelayReceiver::poll_next", level = "trace", skip(self, cx))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Ok(mut queue) = self.0.lock() else {
            tracing::error!("poisoned mutex");
            return Poll::Ready(None);
        };

        // Wait until the timer is done, if any
        if let Some(next_wakeup) = queue.next_wakeup.as_mut() {
            tracing::trace!("polling timer");
            let _ = futures::ready!(next_wakeup.poll_unpin(cx));
            queue.next_wakeup = None;
        }

        tracing::trace!("timer finished");

        let now = Instant::now();
        while let Some(e) = queue.entries.first() {
            if !e.cancelled.load(std::sync::atomic::Ordering::SeqCst) {
                return if e.at.saturating_duration_since(now) < SkipDelayQueue::<T>::TOLERANCE {
                    // If the item is already in the past, yield it
                    tracing::trace!("ready");
                    Poll::Ready(queue.entries.pop_first().map(|e| e.item))
                } else {
                    // The next item is in the future, set up the timer and wake us up to start it
                    tracing::trace!("pending new timer");
                    queue.next_wakeup = Some(futures_time::task::sleep_until(e.at.into()));
                    cx.waker().wake_by_ref();
                    Poll::Pending
                };
            } else {
                // If the item has been canceled, remove it and continue
                queue.entries.pop_first();
                tracing::trace!("item cancelled");
            }
        }

        if !queue.is_closed {
            // Need more data, wake us up when some are added
            tracing::trace!("pending for data");
            queue.rx_waker = Some(cx.waker().clone());
            Poll::Pending
        } else {
            // We're done
            Poll::Ready(None)
        }
    }
}

/// Sender part for the [`skip_delay_channel`].
pub struct SkipDelaySender<T>(Option<Arc<std::sync::Mutex<SkipDelayQueue<T>>>>);

impl<T> Clone for SkipDelaySender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> SkipDelaySender<T> {
    fn ensure_closure(&mut self) {
        if let Some(queue) = self.0.take() {
            let count_holders = Arc::strong_count(&queue);
            tracing::trace!(count_holders, "ensure_closure");

            // Check if the last holders are this instance and (potentially) the receiver
            if count_holders == 2 {
                Self::finalize_closure(queue);
            }
        }
    }

    fn finalize_closure(queue: Arc<std::sync::Mutex<SkipDelayQueue<T>>>) {
        tracing::trace!("finalize_closure");
        queue.clear_poison();
        let mut queue = queue.lock().expect("cannot panic because poison is cleared");
        queue.is_closed = true;
        queue.rx_waker = None;
    }

    /// Forces closure of the queue (regardless of any remaining senders).
    pub fn force_close(&mut self) {
        if let Some(queue) = self.0.take() {
            Self::finalize_closure(queue);
        }
    }
}

impl<T: Ord> SkipDelaySender<T> {
    #[instrument(
        name = "SkipDelaySender::send_internal",
        level = "trace",
        skip(self, items, flush),
        ret
    )]
    fn send_internal<I: Iterator<Item = DelayedItem<T>>>(&self, items: I, flush: bool) -> Result<(), std::io::Error> {
        if let Some(queue) = self.0.as_ref() {
            let mut queue = queue.lock().map_err(|_| std::io::ErrorKind::BrokenPipe)?;

            // This can happen only when the receiver was dropped.
            if queue.is_closed {
                return Err(std::io::ErrorKind::BrokenPipe.into());
            }

            for item in items {
                match item {
                    DelayedItem::New(item, at) => {
                        tracing::trace!(at =  ?at.saturating_duration_since(Instant::now()), "inserting");
                        queue.entries.replace(DelayedEntry {
                            item,
                            at,
                            cancelled: AtomicBool::new(false),
                        });
                    }
                    DelayedItem::Cancel(item) => {
                        tracing::trace!("cancelling");
                        queue
                            .entries
                            .iter()
                            .filter(|e| item == e.item)
                            .for_each(|e| e.cancelled.store(true, std::sync::atomic::Ordering::SeqCst));
                    }
                }
            }

            if flush {
                tracing::trace!("flushing");
                if let Some(waker) = queue.rx_waker.take() {
                    waker.wake();
                }
            }

            Ok(())
        } else {
            Err(std::io::ErrorKind::NotConnected.into())
        }
    }

    /// Sends the given single item and flushes the queue.
    pub fn send_one<I: Into<DelayedItem<T>>>(&mut self, item: I) -> Result<(), std::io::Error> {
        self.send_internal(std::iter::once(item.into()), true)
    }

    /// Sends many items at once and then flushes the queue.
    pub fn send_many<I: IntoIterator<Item = DelayedItem<T>>>(&mut self, items: I) -> Result<(), std::io::Error> {
        self.send_internal(items.into_iter(), true)
    }
}

impl<T> Drop for SkipDelaySender<T> {
    fn drop(&mut self) {
        self.ensure_closure();
    }
}

impl<T: Ord> futures::Sink<DelayedItem<T>> for SkipDelaySender<T> {
    type Error = std::io::Error;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.0.is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(std::io::ErrorKind::NotConnected.into()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: DelayedItem<T>) -> Result<(), Self::Error> {
        self.send_internal(std::iter::once(item), false)
    }

    #[instrument(name = "SkipDelaySender::poll_flush", level = "trace", skip(self), ret)]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Some(queue) = self.0.as_ref() {
            let Ok(mut queue) = queue.lock() else {
                return Poll::Ready(Err(std::io::ErrorKind::BrokenPipe.into()));
            };

            tracing::trace!("flushing");
            if let Some(waker) = queue.rx_waker.take() {
                waker.wake();
            }

            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(std::io::ErrorKind::NotConnected.into()))
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.0.is_none() {
            return Poll::Ready(Err(std::io::ErrorKind::NotConnected.into()));
        }

        self.ensure_closure();
        Poll::Ready(Ok(()))
    }
}

/// A MPSC queue of [items](DelayedItem) with attached [`Instant`] that determines a deadline
/// at which it should be yielded from the [`Stream`](futures::Stream) side of the queue.
/// The queue also has the cancellation ability: an item that has been pushed into the
/// queue earlier can be canceled before it meets its deadline.
/// A canceled item will then be skipped in the output stream.
///
/// The items are internally sorted based on their deadline.
/// In a case when two items have equal deadlines, they are sorted according
/// to their values; therefore, items must implement [`Ord`].
///
/// If equal items are inserted, the deadline of the earlier one inserted is updated.
pub fn skip_delay_channel<T: Ord>() -> (SkipDelaySender<T>, SkipDelayReceiver<T>) {
    let queue = Arc::new(std::sync::Mutex::new(SkipDelayQueue::new()));
    (SkipDelaySender(Some(queue.clone())), SkipDelayReceiver(queue))
}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt, pin_mut};

    use super::*;

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_yield_items() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now + Duration::from_millis(100)).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(100));
        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_replace_and_yield_items() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now + Duration::from_millis(100)).into()).await?;
        tx.send((1, now + Duration::from_millis(200)).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(200));
        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_yield_items_from_multiple_senders() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
        pin_mut!(rx);

        let mut tx2 = tx.clone();

        let now = Instant::now();
        tx.send((2, now + Duration::from_millis(100)).into()).await?;
        tx.close().await?;

        tx2.send((1, now + Duration::from_millis(150)).into()).await?;
        tx2.close().await?;

        assert_eq!(Some(2), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(100));
        assert_eq!(Some(1), rx.next().await);
        assert!(now.elapsed() >= Duration::from_millis(150));

        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_yielded_items_should_be_apart() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
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

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_not_yield_cancelled_items() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now + Duration::from_millis(100)).into()).await?;
        tx.send((1, Skip).into()).await?;
        tx.close().await?;

        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_yield_past_items_immediately() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
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

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_not_yield_future_cancelled_items() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
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

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_discard_duplicate_entries() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
        pin_mut!(rx);

        let now = Instant::now();
        tx.send((1, now).into()).await?;
        tx.send((1, now).into()).await?;
        tx.close().await?;

        assert_eq!(Some(1), rx.next().await);
        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_yield_items_in_order() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
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

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_yield_fed_items_in_order() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
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

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_not_send_items_when_closed() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();
        pin_mut!(rx);
        tx.close().await?;

        let now = Instant::now();
        tx.send((1, now).into()).await.unwrap_err();
        tx.close().await.unwrap_err();

        assert_eq!(None, rx.next().await);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn skip_delay_queue_should_continuously_yield_items() -> anyhow::Result<()> {
        let (mut tx, rx) = skip_delay_channel();

        let items = [5, 2, 1, 4, 3];

        let now = Instant::now();
        let timed_items = (0..5)
            .map(|i| (items[i], now + Duration::from_millis(100) * (i as u32)))
            .collect::<Vec<_>>();

        let timed_items_clone = timed_items.clone();
        let jh = hopr_async_runtime::prelude::spawn(async move {
            for (n, time) in timed_items_clone {
                tx.send((n, time).into()).await?;
                hopr_async_runtime::prelude::sleep(Duration::from_millis(50)).await;
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

        jh.await??;
        Ok(())
    }
}
