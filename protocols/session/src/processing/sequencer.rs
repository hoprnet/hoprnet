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
    /// Head-of-line bound: abandon the frame due next once the sequence has advanced this far
    /// past it, rather than holding everything for `max_wait`. `None` disables it.
    max_frames_behind_gap: Option<usize>,
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
    fn new(
        inner: S,
        max_wait: Duration,
        capacity: usize,
        max_item_age: Option<Duration>,
        max_frames_behind_gap: Option<usize>,
    ) -> Self {
        assert!(capacity > 0, "capacity should be positive");
        Self {
            inner,
            buffer: BinaryHeap::with_capacity(capacity),
            timer: futures_time::task::sleep(max_wait.max(Duration::from_millis(1)).into()),
            next_id: 1,
            last_emitted: Instant::now(),
            max_wait,
            max_item_age: max_item_age.filter(|age| !age.is_zero()),
            // Zero would abandon the frame due next before anything had arrived to justify it,
            // turning every momentary gap into loss. One later frame is the strictest evidence
            // that still *is* evidence.
            max_frames_behind_gap: max_frames_behind_gap.map(|n| n.max(1)),
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
                            // The sequence has moved far enough past the gap to conclude the
                            // missing frame was lost rather than reordered. Waiting out `max_wait`
                            // cannot change that verdict, and on a session with no retransmission
                            // nothing else can either -- it only holds everything already received
                            // for the full duration. Measured on a cluster: 98.5% of bytes
                            // returning over the wire, 0.60% reaching the application.
                            || this.max_frames_behind_gap.is_some_and(|n| {
                                // Two readings of the same evidence, because each is blind where
                                // the other sees. The count catches a run of frames arriving
                                // behind the gap. The distance catches the case that actually
                                // dominates under loss: the frames in between never completed, so
                                // they never reach the sequencer and the buffer stays nearly
                                // empty however far the sender has moved on -- measured on a
                                // cluster as the same setting delivering 95-97% on some runs and
                                // 0.5% on others, the failing ones back on the frame timeout.
                                this.buffer.len() >= n
                                    || next.gt(&this.next_id.saturating_add(n as FrameId - 1))
                            })
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

/// How a [`Sequencer`] decides to stop waiting for a frame that has not arrived.
///
/// Grouped rather than passed positionally: the two stopping rules below answer different
/// questions, and a bare list of `Duration`/`Option<usize>` arguments at the call site gives no
/// hint which is which.
#[derive(Clone, Copy, Debug)]
pub struct SequencerConfig {
    /// Longest the frame due next is waited for before it is abandoned.
    pub max_wait: Duration,
    /// Maximum number of buffered items.
    pub capacity: usize,
    /// Discards items buffered longer than this instead of emitting them late; `None` disables.
    pub max_item_age: Option<Duration>,
    /// Abandons the frame due next once this many later frames are already waiting behind it.
    ///
    /// `None` waits out [`Self::max_wait`] regardless of how much has piled up, which is correct
    /// only when the missing frame can still be recovered. On a session without retransmission it
    /// cannot: the wait is for something that is never coming, and everything already received is
    /// held behind it for the full duration.
    ///
    /// Counting later frames rather than watching a clock makes the decision on evidence. One or
    /// two frames arriving ahead is ordinary reordering across paths of differing latency; a queue
    /// building up behind a gap is a frame that was lost.
    pub max_frames_behind_gap: Option<usize>,
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
        Sequencer::new(self, timeout, capacity, None, None)
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
        Sequencer::new(self, timeout, capacity, max_item_age, None)
    }

    /// As [`SequencerExt::sequencer`], with every stopping rule stated explicitly.
    fn sequencer_with(self, cfg: SequencerConfig) -> Sequencer<Self>
    where
        Self::Item: Ord + PartialOrd<FrameId>,
        Self: Sized,
    {
        Sequencer::new(
            self,
            cfg.max_wait,
            cfg.capacity,
            cfg.max_item_age,
            cfg.max_frames_behind_gap,
        )
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

    /// Frames waiting behind a gap must not be held for `max_wait`.
    ///
    /// This is the head-of-line stall, measured on a live cluster: with `max_wait` at 3 s a
    /// session returning 98.5 % of its bytes over the wire delivered 0.60 % of them to the
    /// application, and the application-side inter-arrival median sat exactly on the timeout. On a
    /// session with no retransmission the missing frame is never coming, so every second of that
    /// wait is spent on an outcome that cannot change while the frames already received are held.
    #[test_log::test(tokio::test)]
    async fn sequencer_should_abandon_a_gap_once_enough_later_frames_are_waiting() -> anyhow::Result<()> {
        // Far longer than the test should take. Any reliance on the timer is then unmistakable in
        // the elapsed assertion rather than a matter of tuning margins.
        let max_wait = Duration::from_secs(5);
        let (mut seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        // Frame 1 never arrives.
        for v in [2u32, 3, 4] {
            seq_sink.feed(v).await?;
        }
        seq_sink.flush().await?;

        let seq_stream = seq_stream.sequencer_with(SequencerConfig {
            max_wait,
            capacity: 4096,
            max_item_age: None,
            max_frames_behind_gap: Some(2),
        });
        pin_mut!(seq_stream);

        let now = Instant::now();
        assert!(
            matches!(seq_stream.try_next().await, Err(SessionError::FrameDiscarded(1))),
            "the gap must be reported as loss, the signal the consumer already handles"
        );
        assert_eq!(Some(2), seq_stream.try_next().await?);
        assert_eq!(Some(3), seq_stream.try_next().await?);
        assert_eq!(Some(4), seq_stream.try_next().await?);
        assert!(
            now.elapsed() < max_wait / 2,
            "frames already received must not wait on a frame that is never coming; took {:?}",
            now.elapsed()
        );

        // Held open deliberately: closing the sink would drain the buffer through `State::Done`,
        // which has its own emit path and would pass this test without the rule under test.
        drop(seq_sink);
        Ok(())
    }

    /// Under real loss the frames between the gap and the newest arrival mostly never complete, so
    /// they never reach the sequencer at all and the buffer stays nearly empty. A rule counting
    /// buffered frames then never fires and the timeout takes over — which is exactly what a
    /// cluster showed: at an identical setting the scenario delivered 95–97 % on some runs and
    /// 0.5 % on others, the failing ones with an application-side inter-arrival median sitting
    /// back on the 3 s timeout.
    ///
    /// What is available regardless is how far the sequence has advanced past the gap. One frame
    /// arriving at id 40 says as much about frame 1 as forty buffered frames would.
    #[test_log::test(tokio::test)]
    async fn sequencer_should_abandon_a_gap_when_the_sequence_has_advanced_past_it() -> anyhow::Result<()> {
        let max_wait = Duration::from_secs(5);
        let (mut seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        // Frame 1 is missing and frames 2..=39 never completed, so only one frame is buffered --
        // far below any count-based threshold, yet the sequence has clearly moved on.
        seq_sink.feed(40u32).await?;
        seq_sink.flush().await?;

        let seq_stream = seq_stream.sequencer_with(SequencerConfig {
            max_wait,
            capacity: 4096,
            max_item_age: None,
            max_frames_behind_gap: Some(4),
        });
        pin_mut!(seq_stream);

        // Only the frames that the sequence has genuinely left behind. The last `n - 1` before the
        // newest arrival are still inside the reordering window -- the sequence has not advanced
        // far enough past *them* to call them lost -- so they keep the timeout, which is the
        // conservative half of the same rule.
        let now = Instant::now();
        for expected_gap in 1..=36u32 {
            assert!(
                matches!(
                    seq_stream.try_next().await,
                    Err(SessionError::FrameDiscarded(id)) if id == expected_gap
                ),
                "frame {expected_gap} is behind the advanced sequence and must be given up on"
            );
        }
        assert!(
            now.elapsed() < max_wait / 2,
            "a single frame far ahead is evidence enough, with no frames buffered behind the gap \
             to count; took {:?}",
            now.elapsed()
        );

        drop(seq_sink);
        Ok(())
    }

    /// The inverse, so the rule cannot become "never wait". Below the threshold the sequencer must
    /// still hold the gap open — one frame arriving ahead is ordinary reordering across paths of
    /// differing latency, not a loss.
    #[test_log::test(tokio::test)]
    async fn sequencer_should_keep_waiting_while_fewer_frames_are_behind_the_gap() -> anyhow::Result<()> {
        let max_wait = Duration::from_millis(300);
        let (mut seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        // Frame 1 is missing and only one frame is waiting behind it, under the threshold of 3.
        seq_sink.feed(2u32).await?;
        seq_sink.flush().await?;

        let seq_stream = seq_stream.sequencer_with(SequencerConfig {
            max_wait,
            capacity: 4096,
            max_item_age: None,
            max_frames_behind_gap: Some(3),
        });
        pin_mut!(seq_stream);

        let now = Instant::now();
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(1))
        ));
        assert!(
            now.elapsed() >= max_wait,
            "under the threshold the timeout still governs; took {:?}",
            now.elapsed()
        );

        drop(seq_sink);
        Ok(())
    }

    /// A session that *can* recover a missing frame must be unaffected: `None` keeps the timeout
    /// as the only rule, however much piles up behind the gap.
    #[test_log::test(tokio::test)]
    async fn sequencer_without_the_gap_bound_should_still_wait_for_the_timeout() -> anyhow::Result<()> {
        let max_wait = Duration::from_millis(300);
        let (mut seq_sink, seq_stream) = futures::channel::mpsc::unbounded();

        for v in [2u32, 3, 4, 5, 6] {
            seq_sink.feed(v).await?;
        }
        seq_sink.flush().await?;

        let seq_stream = seq_stream.sequencer_with(SequencerConfig {
            max_wait,
            capacity: 4096,
            max_item_age: None,
            max_frames_behind_gap: None,
        });
        pin_mut!(seq_stream);

        let now = Instant::now();
        assert!(matches!(
            seq_stream.try_next().await,
            Err(SessionError::FrameDiscarded(1))
        ));
        assert!(
            now.elapsed() >= max_wait,
            "with no gap bound the timeout is the only rule; took {:?}",
            now.elapsed()
        );

        drop(seq_sink);
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
