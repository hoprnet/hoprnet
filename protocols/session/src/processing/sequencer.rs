//! This module defines the [`Sequencer`] stream adaptor.

use std::{
    collections::BinaryHeap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures_time::future::Timer;
use tracing::instrument;

use crate::{errors::SessionError, protocol::FrameId};

/// Buffer entry pairing an item with when it entered the buffer.
///
/// Ordering delegates entirely to `item`, so the heap behaves exactly as before; `buffered_at`
/// exists only to age the entry out.
#[derive(Clone, Copy, Debug)]
struct Buffered<T> {
    item: T,
    buffered_at: Instant,
}

impl<T: PartialEq> PartialEq for Buffered<T> {
    fn eq(&self, other: &Self) -> bool {
        self.item.eq(&other.item)
    }
}

impl<T: Eq> Eq for Buffered<T> {}

impl<T: PartialOrd> PartialOrd for Buffered<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.item.partial_cmp(&other.item)
    }
}

impl<T: Ord> Ord for Buffered<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.item.cmp(&other.item)
    }
}

impl<T: PartialOrd<FrameId>> PartialEq<FrameId> for Buffered<T> {
    fn eq(&self, other: &FrameId) -> bool {
        self.item.partial_cmp(other) == Some(std::cmp::Ordering::Equal)
    }
}

impl<T: PartialOrd<FrameId>> PartialOrd<FrameId> for Buffered<T> {
    fn partial_cmp(&self, other: &FrameId) -> Option<std::cmp::Ordering> {
        self.item.partial_cmp(other)
    }
}

/// Sequencer is an adaptor for streams, that yield elements that have a natural ordering and
/// can be compared with [`FrameId`] and puts them in the correct sequence starting with
/// `FrameId` equal to 1.
///
/// Sequencer internally maintains a `FrameId` to be yielded next, polls the underlying stream
/// and yields elements only when they match the next `FrameId` to be yielded, incrementing the
/// value on each yield.
///
/// The Sequencer takes to arguments: `max_wait` and `capacity`:
///
/// The `max_wait` indicates the maximum amount of time to wait for a certain `FrameId` to
/// be yielded from the underlying stream.
/// If this does not happen, the Segmenter yields an error,
/// indicating that the given frame was discarded.
///
/// The `capacity` parameter sets the maximum number of buffered elements inside the Sequencer.
/// If this value is reached, the Sequencer will stop polling the underlying stream, waiting for the
/// next element to expire.
///
/// By definition, Sequencer is a fallible stream, yielding either `Ok(Item)`, `Err(`[`SessionError::FrameDiscarded`]`)`
/// or `Ok(None)` when the underlying stream is closed and no more elements can be yielded.
///
/// Use [`SequencerExt`] methods to turn a stream into a sequenced stream.
#[must_use = "streams do nothing unless polled"]
#[pin_project::pin_project]
pub struct Sequencer<S: futures::Stream> {
    #[pin]
    inner: S,
    #[pin]
    timer: futures_time::task::Sleep,
    buffer: BinaryHeap<std::cmp::Reverse<Buffered<S::Item>>>,
    next_id: FrameId,
    last_emitted: Instant,
    max_wait: Duration,
    /// Anti-bufferbloat bound: items buffered longer than this are dropped, not emitted, so a
    /// stall shows up as loss rather than a latency tail. `None` disables it.
    max_item_age: Option<Duration>,
    state: State,
}

impl<S> Sequencer<S>
where
    S: futures::Stream,
    S::Item: Ord + PartialOrd<FrameId>,
{
    /// Creates a new instance, wrapping the given `inner` Segment sink.
    ///
    /// The `frame_size` value will be clamped into the `[C, (C - SessionMessage::SEGMENT_OVERHEAD) * SeqIndicator::MAX
    /// + 1]` interval.
    fn new(inner: S, max_wait: Duration, capacity: usize, max_item_age: Option<Duration>) -> Self {
        assert!(capacity > 0, "capacity should be positive");
        Self {
            inner,
            buffer: BinaryHeap::with_capacity(capacity),
            timer: futures_time::task::sleep(max_wait.max(Duration::from_millis(1)).into()),
            next_id: 1,
            last_emitted: Instant::now(),
            max_wait,
            max_item_age: max_item_age.filter(|age| !age.is_zero()),
            state: State::Polling,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Polling,
    BufferUpdated,
    Done,
}

impl<S> futures::Stream for Sequencer<S>
where
    S: futures::Stream,
    S::Item: Ord + PartialOrd<FrameId>,
{
    type Item = Result<S::Item, SessionError>;

    #[instrument(name = "Sequencer::poll_next", level = "trace", skip(self, cx), fields(next_frame_id = self.next_id, state = ?self.state))]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        if *this.next_id == 0 {
            tracing::debug!("end of frame sequence reached");
            return Poll::Ready(None);
        }

        loop {
            match *this.state {
                State::Polling => {
                    if this.buffer.len() < this.buffer.capacity() {
                        // We still have capacity available, poll the underlying stream
                        let stream_poll = this.inner.as_mut().poll_next(cx);

                        // Only poll timer if there's something in the buffer
                        let timer_poll = if !this.buffer.is_empty() {
                            let poll = this.timer.as_mut().poll(cx);
                            if poll.is_ready() {
                                this.timer.as_mut().reset_timer();
                            }
                            poll
                        } else {
                            Poll::Pending
                        };

                        match (stream_poll, timer_poll) {
                            (Poll::Pending, Poll::Pending) => {
                                tracing::trace!("pending");
                                *this.state = State::Polling;
                                return Poll::Pending;
                            }
                            (Poll::Ready(Some(item)), _) => {
                                // We have to reset the last emitted timestamp if
                                // the buffer was empty until now
                                if this.buffer.is_empty() {
                                    *this.last_emitted = Instant::now();
                                }

                                if item.lt(this.next_id) {
                                    // Do not accept older frame ids
                                    tracing::error!("old item");
                                    *this.state = State::Polling;
                                } else {
                                    // Push new item to the buffer
                                    tracing::trace!("new item");
                                    this.buffer.push(std::cmp::Reverse(Buffered {
                                        item,
                                        buffered_at: Instant::now(),
                                    }));
                                    *this.state = State::BufferUpdated;
                                }
                            }
                            (Poll::Ready(None), _) => {
                                tracing::trace!(len = this.buffer.len(), "stream is done");
                                *this.state = State::Done
                            }
                            (_, Poll::Ready(_)) => {
                                // Simulate buffer update when the timer elapses
                                tracing::trace!("timer elapsed");
                                *this.state = State::BufferUpdated;
                            }
                        }
                    } else {
                        // Simulate buffer update when at capacity
                        tracing::warn!("sequencer buffer is full");
                        *this.state = State::BufferUpdated;
                    }
                }
                State::BufferUpdated => {
                    // The buffer has been updated, check if we can yield something
                    if let Some(next) = this.buffer.peek().map(|item| &item.0) {
                        if next.eq(this.next_id) {
                            let stale = this
                                .max_item_age
                                .is_some_and(|max_age| next.buffered_at.elapsed() >= max_age);

                            *this.next_id = this.next_id.wrapping_add(1);
                            *this.last_emitted = Instant::now();
                            *this.state = State::BufferUpdated;

                            // Anti-bufferbloat: the frame is the one due next, but it has been held
                            // so long that delivering it would only add latency. Report it as
                            // discarded — the same signal the consumer already handles for a lost
                            // frame — instead of emitting it late.
                            if stale {
                                let discarded = this.next_id.wrapping_sub(1);
                                this.buffer.pop();
                                tracing::trace!(discarded, "discard frame that exceeded max age");
                                return Poll::Ready(Some(Err(SessionError::FrameDiscarded(discarded))));
                            }

                            tracing::trace!("emit next frame");

                            return Poll::Ready(this.buffer.pop().map(|item| Ok(item.0.item)));
                        } else if this.last_emitted.elapsed() >= *this.max_wait
                            || this.buffer.len() == this.buffer.capacity()
                        {
                            let discarded = *this.next_id;
                            *this.next_id = this.next_id.wrapping_add(1);
                            // `last_emitted` is intentionally NOT reset here: it is only reset
                            // when an actual frame is emitted. Resetting it per discarded id would
                            // drain a contiguous gap of K missing frames at 1 frame per `max_wait`
                            // (a K x max_wait delivery stall of frames already sitting in the
                            // buffer), instead of flushing the whole gap once `max_wait` elapses.
                            *this.state = State::BufferUpdated;

                            tracing::trace!(discarded, "discard frame");

                            return Poll::Ready(Some(Err(SessionError::FrameDiscarded(discarded))));
                        }
                    } else {
                        tracing::trace!("buffer is empty");
                    }

                    // Nothing to yield, keep on polling
                    *this.state = State::Polling;
                }
                State::Done => {
                    // The underlying stream is done, drain what we have in the internal buffer
                    return if let Some(next) = this.buffer.peek().map(|item| &item.0) {
                        if next.lt(this.next_id) {
                            tracing::error!("old item");
                            this.buffer.pop();
                            continue;
                        } else if next.eq(this.next_id) {
                            *this.next_id = this.next_id.wrapping_add(1);
                            tracing::trace!("emit next frame when done");

                            Poll::Ready(this.buffer.pop().map(|item| Ok(item.0.item)))
                        } else {
                            let discarded = *this.next_id;
                            *this.next_id = this.next_id.wrapping_add(1);
                            tracing::trace!(discarded, "discard frame when done");

                            Poll::Ready(Some(Err(SessionError::FrameDiscarded(discarded))))
                        }
                    } else {
                        tracing::trace!("buffer is empty and done");
                        Poll::Ready(None)
                    };
                }
            }
        }
    }
}

/// Stream extensions methods for item sequencing.
pub trait SequencerExt: futures::Stream {
    /// Attaches a [`Sequencer`] to the underlying stream, given the item `timeout` and `capacity`
    /// of items.
    fn sequencer(self, timeout: Duration, capacity: usize) -> Sequencer<Self>
    where
        Self::Item: Ord + PartialOrd<FrameId>,
        Self: Sized,
    {
        Sequencer::new(self, timeout, capacity, None)
    }

    /// As [`SequencerExt::sequencer`], but discards items buffered longer than `max_item_age`
    /// instead of emitting them late.
    fn sequencer_with_max_age(
        self,
        timeout: Duration,
        capacity: usize,
        max_item_age: Option<Duration>,
    ) -> Sequencer<Self>
    where
        Self::Item: Ord + PartialOrd<FrameId>,
        Self: Sized,
    {
        Sequencer::new(self, timeout, capacity, max_item_age)
    }
}

impl<T: ?Sized> SequencerExt for T where T: futures::Stream {}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt, TryStreamExt, pin_mut};
    use futures_time::future::FutureExt;

    use super::*;

    #[test_log::test(tokio::test)]
    async fn sequencer_should_return_entries_in_order() -> anyhow::Result<()> {
        let mut expected = vec![4u32, 1, 5, 7, 8, 6, 2, 3];

        let actual: Vec<u32> = futures::stream::iter(expected.clone())
            .sequencer(Duration::from_secs(5), 4096)
            .try_collect()
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;

        expected.sort();
        assert_eq!(expected, actual);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entries_that_exceeded_the_max_age() -> anyhow::Result<()> {
        let (seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        // `max_wait` is long, so nothing here is discarded for being *missing* — only for being stale.
        let seq_stream =
            seq_stream.sequencer_with_max_age(Duration::from_secs(30), 4096, Some(Duration::from_millis(100)));

        pin_mut!(seq_sink);
        pin_mut!(seq_stream);

        // Frame 2 arrives first and waits in the buffer for the missing frame 1 — a transport stall.
        seq_sink.send(2u32).await?;

        // Drive the sequencer so frame 2 actually lands in its buffer; until it is polled the item
        // only sits in the channel and its buffered-at clock has not started.
        assert!(
            seq_stream
                .try_next()
                .timeout(futures_time::time::Duration::from_millis(50))
                .await
                .is_err(),
            "nothing is emitted while frame 1 is missing"
        );

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;

        // Frame 1 arrives fresh and is delivered; frame 2 is now 250 ms stale and must be
        // reported as discarded rather than handed over a quarter of a second late.
        seq_sink.send(1u32).await?;

        assert_eq!(Some(1), seq_stream.try_next().await?, "the fresh frame is delivered");
        assert!(
            matches!(seq_stream.try_next().await, Err(SessionError::FrameDiscarded(2))),
            "the stale frame must be discarded, not delivered late"
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_deliver_entries_within_the_max_age() -> anyhow::Result<()> {
        let (seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        let seq_stream =
            seq_stream.sequencer_with_max_age(Duration::from_secs(30), 4096, Some(Duration::from_secs(30)));

        pin_mut!(seq_sink);
        pin_mut!(seq_stream);

        // Same out-of-order arrival and the same buffering delay, but comfortably inside the
        // bound: nothing is dropped.
        seq_sink.send(2u32).await?;
        assert!(
            seq_stream
                .try_next()
                .timeout(futures_time::time::Duration::from_millis(50))
                .await
                .is_err()
        );
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        seq_sink.send(1u32).await?;

        assert_eq!(Some(1), seq_stream.try_next().await?);
        assert_eq!(Some(2), seq_stream.try_next().await?);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_not_allow_emitted_entries() -> anyhow::Result<()> {
        let (seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        let seq_stream = seq_stream.sequencer(Duration::from_secs(1), 4096);

        pin_mut!(seq_sink);
        pin_mut!(seq_stream);

        seq_sink.send(1u32).await?;
        assert_eq!(Some(1), seq_stream.try_next().await?);

        seq_sink.send(2u32).await?;
        assert_eq!(Some(2), seq_stream.try_next().await?);

        seq_sink.send(2u32).await?;
        seq_sink.send(1u32).await?;

        seq_sink.send(3u32).await?;
        assert_eq!(Some(3), seq_stream.try_next().await?);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entry_on_timeout() -> anyhow::Result<()> {
        let timeout = Duration::from_millis(25);
        let (mut seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        let input = vec![2u32, 1, 4, 5, 8, 7, 9, 11, 10];

        let input_clone = input.clone();
        let jh = hopr_utils::runtime::prelude::spawn(async move {
            for v in input_clone {
                seq_sink
                    .feed(v)
                    .delay(futures_time::time::Duration::from_millis(5))
                    .await?;
            }
            seq_sink.flush().await?;
            seq_sink.close().await
        });

        let seq_stream = seq_stream.sequencer(timeout, 4096);

        pin_mut!(seq_stream);

        assert_eq!(Some(1), seq_stream.try_next().await?);
        assert_eq!(Some(2), seq_stream.try_next().await?);

        let now = Instant::now();
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(3))
        ));
        assert!(now.elapsed() >= timeout);

        assert_eq!(Some(4), seq_stream.try_next().await?);
        assert_eq!(Some(5), seq_stream.try_next().await?);

        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(6))
        ));

        assert_eq!(Some(7), seq_stream.try_next().await?);
        assert_eq!(Some(8), seq_stream.try_next().await?);
        assert_eq!(Some(9), seq_stream.try_next().await?);
        assert_eq!(Some(10), seq_stream.try_next().await?);
        assert_eq!(Some(11), seq_stream.try_next().await?);

        assert_eq!(None, seq_stream.try_next().await?);

        let _ = jh.await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entry_close() -> anyhow::Result<()> {
        let (seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        let input = vec![2u32, 1, 3, 5, 4, 8, 11];

        hopr_utils::runtime::prelude::spawn(futures::stream::iter(input.clone()).map(Ok).forward(seq_sink)).await??;

        let seq_stream = seq_stream.sequencer(Duration::from_millis(25), 4096);

        pin_mut!(seq_stream);

        assert_eq!(Some(1), seq_stream.try_next().await?);
        assert_eq!(Some(2), seq_stream.try_next().await?);
        assert_eq!(Some(3), seq_stream.try_next().await?);
        assert_eq!(Some(4), seq_stream.try_next().await?);
        assert_eq!(Some(5), seq_stream.try_next().await?);
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(6))
        ));
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(7))
        ));
        assert_eq!(Some(8), seq_stream.try_next().await?);
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(9))
        ));
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(10))
        ));
        assert_eq!(Some(11), seq_stream.try_next().await?);
        assert_eq!(None, seq_stream.try_next().await?);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entry_when_inner_stream_pending() -> anyhow::Result<()> {
        let sent = vec![4u32, 1, 7, 8, 6, 2, 3];
        let (tx, rx) = futures::channel::mpsc::unbounded();

        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter(sent.clone()).map(Ok)).await?;

        let rx = rx.sequencer(Duration::from_millis(10), 4096);
        pin_mut!(rx);

        assert!(matches!(rx.next().await, Some(Ok(1))));
        assert!(matches!(rx.next().await, Some(Ok(2))));
        assert!(matches!(rx.next().await, Some(Ok(3))));
        assert!(matches!(rx.next().await, Some(Ok(4))));
        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(5)))));
        assert!(matches!(rx.next().await, Some(Ok(6))));
        assert!(matches!(rx.next().await, Some(Ok(7))));
        assert!(matches!(rx.next().await, Some(Ok(8))));

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_discard_entry_when_capacity_is_reached() -> anyhow::Result<()> {
        let sent = vec![4u32, 5, 7, 8, 2, 6, 3];
        let (tx, rx) = futures::channel::mpsc::unbounded();

        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter(sent.clone()).map(Ok)).await?;

        let rx = rx.sequencer(Duration::from_millis(10), 4);
        pin_mut!(rx);

        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(1)))));
        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(2)))));
        assert!(matches!(rx.next().await, Some(Err(SessionError::FrameDiscarded(3)))));
        assert!(matches!(rx.next().await, Some(Ok(4))));
        assert!(matches!(rx.next().await, Some(Ok(5))));
        assert!(matches!(rx.next().await, Some(Ok(6))));
        assert!(matches!(rx.next().await, Some(Ok(7))));
        assert!(matches!(rx.next().await, Some(Ok(8))));

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_drain_contiguous_gap_within_single_timeout_window() -> anyhow::Result<()> {
        let timeout = Duration::from_millis(50);
        let (tx, rx) = futures::channel::mpsc::unbounded();

        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter([1u32, 2, 10, 11, 12]).map(Ok))
            .await?;

        let rx = rx.sequencer(timeout, 4096);
        pin_mut!(rx);

        assert_eq!(Some(1), rx.try_next().await?);
        assert_eq!(Some(2), rx.try_next().await?);

        let now = Instant::now();
        for expected in 3u32..=9 {
            assert!(matches!(
                rx.next().await,
                Some(Err(SessionError::FrameDiscarded(id))) if id == expected
            ));
        }
        assert_eq!(Some(10), rx.try_next().await?);
        assert_eq!(Some(11), rx.try_next().await?);
        assert_eq!(Some(12), rx.try_next().await?);

        // The 7-frame gap must be flushed after one timeout window,
        // not at a rate of one frame per window.
        assert!(
            now.elapsed() < 3 * timeout,
            "gap drain took {:?}, expected well under {:?}",
            now.elapsed(),
            7 * timeout
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_must_terminate_on_last_frame_id() -> anyhow::Result<()> {
        let (tx, rx) = futures::channel::mpsc::unbounded();

        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter([FrameId::MAX - 1, FrameId::MAX, 1, 2]).map(Ok))
            .await?;

        let mut rx = rx.sequencer(Duration::from_millis(10), 1024);
        rx.next_id = FrameId::MAX - 1;
        pin_mut!(rx);

        const LAST_ID: FrameId = FrameId::MAX - 1;
        assert!(matches!(rx.next().await, Some(Ok(LAST_ID))));
        assert!(matches!(rx.next().await, Some(Ok(FrameId::MAX))));
        assert!(rx.next().await.is_none());

        Ok(())
    }

    #[test_log::test(tokio::test(flavor = "multi_thread"))]
    async fn sequencer_must_not_discard_frames_when_buffer_was_empty_after_timeout() -> anyhow::Result<()> {
        let (tx, rx) = futures::channel::mpsc::unbounded();

        let jh = tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(2)).await;
            pin_mut!(tx);
            tx.send_all(&mut futures::stream::iter([3, 1, 2, 4]).map(Ok)).await?;

            tokio::time::sleep(Duration::from_millis(150)).await;

            tx.send_all(&mut futures::stream::iter([6, 5, 7]).map(Ok)).await?;

            anyhow::Ok(())
        });

        let chunks = rx
            .sequencer(Duration::from_millis(50), 1024)
            .try_ready_chunks(10)
            .try_collect::<Vec<Vec<_>>>()
            .await?;

        assert_eq!(chunks, vec![vec![1, 2, 3, 4], vec![5, 6, 7]]);
        jh.await??;

        Ok(())
    }
}
