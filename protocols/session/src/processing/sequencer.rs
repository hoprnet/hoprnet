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

use crate::{
    errors::SessionError,
    processing::reorder::{ABANDONED_MEMORY, SharedReorder},
    protocol::FrameId,
    socket::DEFAULT_SUCCESSOR_TERMINATING_SLACK,
};

/// Floor on a gap timer, so a gap already past its wait still yields to the runtime once.
const MIN_GAP_WAIT: Duration = Duration::from_millis(1);

/// Items that carry the [`FrameId`] they are sequenced by.
pub(crate) trait HasFrameId {
    fn frame_id(&self) -> FrameId;

    /// Whether the item ends the stream.
    fn is_terminating(&self) -> bool {
        false
    }
}

impl HasFrameId for FrameId {
    fn frame_id(&self) -> FrameId {
        *self
    }
}

impl HasFrameId for crate::protocol::OrderedFrame {
    fn frame_id(&self) -> FrameId {
        self.0.frame_id
    }

    fn is_terminating(&self) -> bool {
        self.0.is_terminating
    }
}

/// Tells the frames of a successor stream from late ones of the stream it replaces.
///
/// Until `period` has elapsed from creation (afterwards everything is admitted):
/// - A data id is stale if it exceeds the highest admitted data id by more than `window`. The highest starts at 0, so
///   before the first admitted frame this is "id above the window".
/// - A terminating id is judged by *contiguous progress* of the stream, not by that highest: the tail of a short old
///   stream (say frames 85..90 and its FIN 91) is in flight together, and its data frames pass the window and raise the
///   highest, so a FIN judged against it would be admitted, wait in the buffer and close the successor once its stream
///   reaches 91. Stale frames cannot fill the gap below them, so they cannot move the contiguous progress. The
///   sequencer knows it (its next expected id, which advances over delivered and given-up frames), and admits a FIN
///   only within [`DEFAULT_SUCCESSOR_TERMINATING_SLACK`] of it: the new stream's own FIN can overtake only a bounded
///   number of its last data frames.
/// - The segment gate has no such knowledge and passes `None`: it applies only the data window to a terminating
///   segment, leaving the slack decision to the sequencer. A terminating item never moves the highest.
///
/// The sequencer and the socket's segment gate share this type so the rules cannot drift.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StartGuard {
    window: FrameId,
    deadline: Instant,
    highest: FrameId,
}

impl StartGuard {
    pub(crate) fn new(window: FrameId, period: Duration) -> Self {
        Self {
            window,
            deadline: Instant::now() + period,
            highest: 0,
        }
    }

    /// Data item: returns `false` if `id` is stale and must be dropped.
    pub(crate) fn admit_data(&mut self, id: FrameId) -> bool {
        if self.expired() {
            return true;
        }
        if id > self.highest.saturating_add(self.window) {
            return false;
        }
        self.highest = self.highest.max(id);
        true
    }

    /// Terminating item: returns `false` if `id` is stale and must be dropped.
    ///
    /// `contiguous` is the highest id up to which the stream is known to be complete or given up,
    /// when the caller knows it (the sequencer); `None` applies the data window only.
    pub(crate) fn admit_terminating(&self, id: FrameId, contiguous: Option<FrameId>) -> bool {
        if self.expired() {
            return true;
        }
        id <= self.highest.saturating_add(self.window)
            && contiguous.is_none_or(|c| id <= c.saturating_add(DEFAULT_SUCCESSOR_TERMINATING_SLACK))
    }

    fn expired(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

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
/// be yielded from the underlying stream. Each gap is timed from when the frame behind it was
/// buffered, so gaps queued behind one another drain together rather than one `max_wait` each.
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
    max_wait: Duration,
    /// Anti-bufferbloat bound: items buffered longer than this are dropped, not emitted, so a
    /// stall shows up as loss rather than a latency tail. `None` disables it.
    max_item_age: Option<Duration>,
    /// Head-of-line bound: abandon the frame due next once the sequence has advanced this far
    /// past it, rather than holding everything for `max_wait`. `None` disables it.
    max_frames_behind_gap: Option<usize>,
    /// Waits on a gap only as long as the reordering seen supports, when set.
    reorder: Option<SharedReorder>,
    /// Successor jump guard: see [`SequencerConfig::successor_start_window`]. `None` disables it.
    guard: Option<StartGuard>,
    /// Number of frames dropped as stale by the guard.
    stale_dropped: u64,
    state: State,
}

impl<S> Sequencer<S>
where
    S: futures::Stream,
    S::Item: Ord + PartialOrd<FrameId> + HasFrameId,
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
        reorder: Option<SharedReorder>,
        guard: Option<StartGuard>,
    ) -> Self {
        assert!(capacity > 0, "capacity should be positive");
        Self {
            inner,
            buffer: BinaryHeap::with_capacity(capacity),
            timer: futures_time::task::sleep(max_wait.max(MIN_GAP_WAIT).into()),
            next_id: 1,
            max_wait,
            max_item_age: max_item_age.filter(|age| !age.is_zero()),
            // Zero would abandon the frame due next before anything had arrived to justify it,
            // turning every momentary gap into loss. One later frame is the strictest evidence
            // that still *is* evidence.
            max_frames_behind_gap: max_frames_behind_gap.map(|n| n.max(1)),
            reorder,
            guard,
            stale_dropped: 0,
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
    S::Item: Ord + PartialOrd<FrameId> + HasFrameId,
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
                                if let Some(guard) = this.guard.as_mut()
                                    && !if item.is_terminating() {
                                        // Contiguous progress: everything below `next_id` is delivered or given up
                                        guard.admit_terminating(item.frame_id(), Some(this.next_id.saturating_sub(1)))
                                    } else {
                                        guard.admit_data(item.frame_id())
                                    }
                                {
                                    // Not the new stream (a terminating frame included): drop it without
                                    // touching `next_id`, the buffer or the gap timer.
                                    *this.stale_dropped += 1;
                                    tracing::debug!(
                                        frame_id = item.frame_id(),
                                        stale_dropped = *this.stale_dropped,
                                        "dropping frame too far ahead of the successor stream"
                                    );
                                    *this.state = State::Polling;
                                    continue;
                                }
                                if item.lt(this.next_id) {
                                    // Do not accept older frame ids. A frame whose gap was given up
                                    // is reordering the wait did not cover, and lengthens it.
                                    let now = Instant::now();
                                    match this.reorder.as_ref().and_then(|reorder| {
                                        (1..=ABANDONED_MEMORY as FrameId)
                                            .map(|back| this.next_id.wrapping_sub(back))
                                            .find(|id| item.eq(id))
                                            .and_then(|id| reorder.late(id, now))
                                            .map(|late| (reorder, late))
                                    }) {
                                        Some((reorder, late)) => {
                                            tracing::debug!(
                                                late_ms = late.as_millis() as u64,
                                                "frame came after its gap was given up"
                                            );
                                            reorder.record(now, late);
                                        }
                                        None => tracing::error!("old item"),
                                    }
                                    *this.state = State::Polling;
                                } else {
                                    // The frame due next closes its gap: how long it came after the
                                    // earliest frame waiting behind it is the reordering it took.
                                    if let Some(reorder) = this.reorder.as_ref()
                                        && item.eq(this.next_id)
                                        && let Some(behind_since) = this.buffer.iter().map(|b| b.0.buffered_at).min()
                                    {
                                        let now = Instant::now();
                                        reorder.record(now, now.duration_since(behind_since));
                                    }
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
                        let max_wait = this
                            .reorder
                            .as_ref()
                            .map_or(*this.max_wait, |reorder| reorder.wait(Instant::now(), *this.max_wait));
                        if next.eq(this.next_id) {
                            let stale = this
                                .max_item_age
                                .is_some_and(|max_age| next.buffered_at.elapsed() >= max_age);

                            *this.next_id = this.next_id.wrapping_add(1);
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
                        // Each gap waits `max_wait` from the arrival of the frame right behind it, so
                        // emissions in between don't push it back: K gaps left by one outage drain
                        // together instead of one per `max_wait`.
                        } else if next.buffered_at.elapsed() >= max_wait
                            || this.buffer.len() == this.buffer.capacity()
                            // The sequence has moved far enough past the gap to conclude the
                            // missing frame was lost rather than reordered. Waiting out `max_wait`
                            // cannot change that verdict, and on a session with no retransmission
                            // nothing else can either.
                            || this.max_frames_behind_gap.is_some_and(|n| {
                                // Two readings of the same evidence, because each is blind where
                                // the other sees. The count catches a run of frames arriving
                                // behind the gap. The distance catches the case that actually
                                // dominates under loss: the frames in between never completed, so
                                // they never reach the sequencer and the buffer stays nearly
                                // empty however far the sender has moved on.
                                this.buffer.len() >= n
                                    // Saturating, not `as`: the bound is configuration, and a value past `FrameId`'s
                                // range would wrap into a small threshold rather than a large one.
                                || next.gt(&this.next_id.saturating_add(
                                    FrameId::try_from(n).unwrap_or(FrameId::MAX).saturating_sub(1),
                                ))
                            })
                        {
                            let discarded = *this.next_id;
                            *this.next_id = this.next_id.wrapping_add(1);
                            *this.state = State::BufferUpdated;

                            tracing::trace!(discarded, "discard frame");
                            let behind_since = next.buffered_at;
                            if let Some(reorder) = this.reorder.as_ref() {
                                reorder.note_abandoned(discarded, behind_since);
                            }

                            return Poll::Ready(Some(Err(SessionError::FrameDiscarded(discarded))));
                        }
                        let remaining = max_wait.saturating_sub(next.buffered_at.elapsed());
                        // The floor only stands in for a deadline already past, so a sub-millisecond
                        // `max_wait` is kept as configured.
                        let wait = if remaining.is_zero() { MIN_GAP_WAIT } else { remaining };
                        this.timer.set(futures_time::task::sleep(wait.into()));
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
    /// Waits on a gap only as long as the reordering this sequence has shown supports: twice the
    /// deepest of the last minute, at least 250 ms, at most [`Self::max_wait`], and all of
    /// [`Self::max_wait`] before any reordering is seen.
    ///
    /// For a sequence that is not retransmitted. `max_wait` is sized for the worst a deployment
    /// may see, so a frame lost on a sequence whose frames arrive within a few hundred
    /// milliseconds of each other holds everything behind it for seconds, waiting for something
    /// that is never coming. A frame that comes after its gap was given up lengthens the wait.
    pub adapt_to_reordering: bool,
    /// Guards a successor stream against frames of the stream it replaces; `None` (the default)
    /// disables the guard.
    ///
    /// For a socket that takes over a pseudonym from a previous Session: frames still in flight
    /// from the old stream carry high ids, and arrive before *and after* the new stream's first
    /// frames. The sequencer would hold such a frame, give up the gap in front of it and skip to
    /// it, black-holing the new stream's lower ids. While the guard is active (for
    /// [`Self::successor_guard_period`] from creation) a frame is stale if its id exceeds the
    /// highest admitted one by more than this window (0 at the start). Stale frames, terminating
    /// ones included, are dropped: they do not move the expected id, do not start the gap timer
    /// and are not a gap or a loss. Afterwards everything is admitted.
    ///
    /// A terminating frame is judged against this sequencer's contiguous progress (its next
    /// expected id) instead, with the much tighter bound
    /// [`DEFAULT_SUCCESSOR_TERMINATING_SLACK`](crate::DEFAULT_SUCCESSOR_TERMINATING_SLACK).
    ///
    /// Trade-off: during the guard, a legitimate jump of more than the window (that many
    /// consecutive frames lost or reordered at once) is dropped until the guard ends, and an old
    /// stream whose ids lie within the window of the new stream's cannot be told apart.
    /// See [`DEFAULT_SUCCESSOR_START_WINDOW`](crate::DEFAULT_SUCCESSOR_START_WINDOW).
    pub successor_start_window: Option<u32>,
    /// How long the [`Self::successor_start_window`] guard lasts, from creation of the sequencer.
    /// Without a window it has no effect.
    pub successor_guard_period: Duration,
}

/// Stream extensions methods for item sequencing.
pub trait SequencerExt: futures::Stream {
    /// Attaches a [`Sequencer`] to the underlying stream, given the item `timeout` and `capacity`
    /// of items.
    fn sequencer(self, timeout: Duration, capacity: usize) -> Sequencer<Self>
    where
        Self::Item: Ord + PartialOrd<FrameId> + HasFrameId,
        Self: Sized,
    {
        Sequencer::new(self, timeout, capacity, None, None, None, None)
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
        Self::Item: Ord + PartialOrd<FrameId> + HasFrameId,
        Self: Sized,
    {
        Sequencer::new(self, timeout, capacity, max_item_age, None, None, None)
    }

    /// As [`SequencerExt::sequencer`], with every stopping rule stated explicitly.
    fn sequencer_with(self, cfg: SequencerConfig) -> Sequencer<Self>
    where
        Self::Item: Ord + PartialOrd<FrameId> + HasFrameId,
        Self: Sized,
    {
        self.sequencer_sharing(cfg, None)
    }

    /// As [`SequencerExt::sequencer_with`], adapting the gap wait to a window of reordering that
    /// others record into as well (the Session's reassembler) when
    /// [`SequencerConfig::adapt_to_reordering`] is set; with `reorder` `None`, the window holds this
    /// sequencer's own samples alone.
    fn sequencer_sharing(self, cfg: SequencerConfig, reorder: Option<SharedReorder>) -> Sequencer<Self>
    where
        Self::Item: Ord + PartialOrd<FrameId> + HasFrameId,
        Self: Sized,
    {
        Sequencer::new(
            self,
            cfg.max_wait,
            cfg.capacity,
            cfg.max_item_age,
            cfg.max_frames_behind_gap,
            cfg.adapt_to_reordering.then(|| reorder.unwrap_or_default()),
            cfg.successor_start_window
                .map(|window| StartGuard::new(window, cfg.successor_guard_period)),
        )
    }
}

impl<T: ?Sized> SequencerExt for T where T: futures::Stream {}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt, TryStreamExt, pin_mut};
    use futures_time::future::FutureExt;

    use super::*;
    use crate::processing::reorder::REORDER_FLOOR;

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
        // Far longer than the test should take, so any reliance on the timer trips the bound below
        // rather than becoming a matter of tuning margins.
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
            adapt_to_reordering: false,
            successor_start_window: None,
            successor_guard_period: Duration::ZERO,
        });
        pin_mut!(seq_stream);

        // The bound is the assertion: without the rule the sequencer holds everything for the full
        // `max_wait`, so a regression trips the timeout instead of blocking the suite for 5 s.
        let released: Vec<u32> = tokio::time::timeout(max_wait / 2, async {
            assert!(
                matches!(seq_stream.try_next().await, Err(SessionError::FrameDiscarded(1))),
                "the gap must be reported as loss, the signal the consumer already handles"
            );
            seq_stream.by_ref().take(3).try_collect().await
        })
        .await
        .map_err(|_| anyhow::anyhow!("frames already received waited on a frame that is never coming"))??;

        assert_eq!(vec![2, 3, 4], released, "everything behind the gap must follow it out");

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
            adapt_to_reordering: false,
            successor_start_window: None,
            successor_guard_period: Duration::ZERO,
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
            "a single frame far ahead is evidence enough, with no frames buffered behind the gap to count; took {:?}",
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
            adapt_to_reordering: false,
            successor_start_window: None,
            successor_guard_period: Duration::ZERO,
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
            adapt_to_reordering: false,
            successor_start_window: None,
            successor_guard_period: Duration::ZERO,
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
    async fn sequencer_should_drain_interleaved_gaps_within_single_timeout_window() -> anyhow::Result<()> {
        let timeout = Duration::from_millis(50);
        let (tx, rx) = futures::channel::mpsc::unbounded();

        pin_mut!(tx);
        tx.send_all(&mut futures::stream::iter([1u32, 3, 5, 7, 9, 11]).map(Ok))
            .await?;

        let rx = rx.sequencer(timeout, 4096);
        pin_mut!(rx);

        assert_eq!(Some(1), rx.try_next().await?);

        let now = Instant::now();
        for frame in [3u32, 5, 7, 9, 11] {
            assert!(matches!(
                rx.next().await,
                Some(Err(SessionError::FrameDiscarded(id))) if id == frame - 1
            ));
            assert_eq!(Some(frame), rx.try_next().await?);
        }

        assert!(
            now.elapsed() < 2 * timeout,
            "interleaved gaps took {:?} to drain, expected about one {:?} window",
            now.elapsed(),
            timeout
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_should_not_discard_gap_filled_within_timeout() -> anyhow::Result<()> {
        let timeout = Duration::from_millis(100);
        let (tx, rx) = futures::channel::mpsc::unbounded();

        let rx = rx.sequencer(timeout, 4096);
        pin_mut!(tx);
        pin_mut!(rx);

        tx.send(1u32).await?;
        tx.send(3u32).await?;
        assert_eq!(Some(1), rx.try_next().await?);
        // Frame 3 is only buffered, and its wait started, once the sequencer is polled again.
        assert!(futures::poll!(rx.next()).is_pending());

        tokio::time::sleep(timeout / 2).await;
        tx.send(2u32).await?;
        assert_eq!(Some(2), rx.try_next().await?);
        assert_eq!(Some(3), rx.try_next().await?);

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

    fn windowed(window: u32, max_wait: Duration) -> SequencerConfig {
        SequencerConfig {
            max_wait,
            capacity: 4096,
            max_item_age: None,
            max_frames_behind_gap: None,
            adapt_to_reordering: false,
            successor_start_window: Some(window),
            successor_guard_period: Duration::from_secs(60),
        }
    }

    /// Feeds `frames` and collects what the sequencer yields until the stream is closed.
    async fn run_windowed(
        window: u32,
        max_wait: Duration,
        frames: Vec<u32>,
    ) -> anyhow::Result<(Vec<Result<u32, SessionError>>, u64)> {
        let mut seq = futures::stream::iter(frames).sequencer_with(windowed(window, max_wait));
        let mut out = Vec::new();
        while let Some(item) = seq.next().timeout(futures_time::time::Duration::from_secs(5)).await? {
            out.push(item);
        }
        Ok((out, seq.stale_dropped))
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_ignore_stale_frames_before_lock_on() -> anyhow::Result<()> {
        // Old high ids from the previous stream, then the new stream. No gap wait may be incurred.
        let (out, dropped) = run_windowed(100, Duration::from_secs(30), vec![5000, 5001, 7777, 1, 2, 3, 4]).await?;
        assert_eq!(vec![1, 2, 3, 4], out.into_iter().collect::<Result<Vec<_>, _>>()?);
        assert_eq!(3, dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_ignore_a_stale_terminating_frame() -> anyhow::Result<()> {
        // The terminating bit is not visible to the sequencer; the id alone decides. An id of
        // `FrameId::MAX` is what the sequencer would otherwise end the sequence on.
        let (out, dropped) = run_windowed(100, Duration::from_secs(30), vec![FrameId::MAX, 1, 2, 3]).await?;
        assert_eq!(vec![1, 2, 3], out.into_iter().collect::<Result<Vec<_>, _>>()?);
        assert_eq!(1, dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_order_new_frames_interleaved_with_stale_ones() -> anyhow::Result<()> {
        // Stale frames arrive before and between the new stream's frames, which are themselves out
        // of order. The guard stays on after lock-on, so 9002 is dropped as well.
        let mut seq = futures::stream::iter(vec![9000u32, 9001, 3, 9002, 1, 2, 4])
            .sequencer_with(windowed(100, Duration::from_secs(30)));
        let mut delivered = Vec::new();
        for _ in 0..4 {
            delivered.push(
                seq.next()
                    .timeout(futures_time::time::Duration::from_secs(5))
                    .await?
                    .expect("stream not ended")?,
            );
        }
        assert_eq!(vec![1, 2, 3, 4], delivered);
        assert_eq!(3, seq.stale_dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_lock_on_via_a_later_in_window_frame() -> anyhow::Result<()> {
        // Frames 1 and 2 are lost; 3 is within the window and locks on. The normal gap handling
        // then reports 1 and 2 as discarded, with no stale frame counted as loss.
        let (out, dropped) = run_windowed(100, Duration::from_millis(50), vec![9000, 3, 4, 5]).await?;
        assert_eq!(1, dropped);
        let discarded: Vec<_> = out
            .iter()
            .filter_map(|r| match r {
                Err(SessionError::FrameDiscarded(id)) => Some(*id),
                _ => None,
            })
            .collect();
        assert_eq!(vec![1, 2], discarded);
        let delivered: Vec<_> = out.into_iter().filter_map(Result::ok).collect();
        assert_eq!(vec![3, 4, 5], delivered);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_drop_stale_frames_arriving_after_lock_on() -> anyhow::Result<()> {
        // The transport's gap bound: were 5000 admitted at next_id 4, the gap would be given up at once.
        let mut cfg = windowed(256, Duration::from_secs(30));
        cfg.max_frames_behind_gap = Some(256);
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mut seq = rx.sequencer_with(cfg);
        for id in [1u32, 2, 3] {
            tx.unbounded_send(id)?;
            assert_eq!(Some(id), seq.try_next().await?);
        }
        tx.unbounded_send(5000)?;
        tx.unbounded_send(4)?;
        tx.unbounded_send(5)?;
        assert_eq!(Some(4), seq.try_next().await?, "no gap give-up, the new stream goes on");
        assert_eq!(Some(5), seq.try_next().await?);
        assert_eq!(1, seq.stale_dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_admit_everything_after_the_guard_period() -> anyhow::Result<()> {
        let mut cfg = windowed(256, Duration::from_secs(30));
        cfg.max_frames_behind_gap = Some(256);
        cfg.successor_guard_period = Duration::from_millis(100);
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mut seq = rx.sequencer_with(cfg);
        tx.unbounded_send(1u32)?;
        assert_eq!(Some(1), seq.try_next().await?);
        tokio::time::sleep(Duration::from_millis(150)).await;
        // As without the option: the far-ahead frame is held, and the gap in front of it given up.
        tx.unbounded_send(5000)?;
        assert!(matches!(seq.try_next().await, Err(SessionError::FrameDiscarded(2))));
        assert_eq!(0, seq.stale_dropped);
        Ok(())
    }

    fn frame(id: FrameId, terminating: bool) -> crate::protocol::OrderedFrame {
        crate::protocol::OrderedFrame(crate::protocol::Frame {
            frame_id: id,
            data: Box::new([]),
            is_terminating: terminating,
        })
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_drop_the_terminating_frame_of_a_short_old_stream() -> anyhow::Result<()> {
        // The old stream's tail (data 85..90) is in flight with its FIN 91. The data passes the window
        // and raises the highest admitted id, but cannot move the contiguous progress the FIN is judged by.
        let stale_tail = || (85..=90).map(|id| frame(id, false)).chain([frame(91, true)]);
        let mut input = vec![frame(1, false), frame(2, false)];
        input.extend(stale_tail());
        input.extend((3..=5).map(|id| frame(id, false)));
        input.push(frame(91, true)); // and again, a little later
        input.extend((6..=200).map(|id| frame(id, false)));
        input.push(frame(201, true));
        let mut seq = futures::stream::iter(input).sequencer_with(windowed(1024, Duration::from_secs(30)));
        let mut items = Vec::new();
        while let Some(item) = seq.next().timeout(futures_time::time::Duration::from_secs(5)).await? {
            items.push(item?);
        }
        let ids: Vec<_> = items.iter().map(|f| f.0.frame_id).collect();
        assert!(ids.windows(2).all(|w| w[0] < w[1]), "increasing ids: {ids:?}");
        let (last, rest) = items.split_last().expect("frames");
        assert!(rest.iter().all(|f| !f.0.is_terminating), "no early termination");
        assert_eq!(201, last.0.frame_id);
        assert!(last.0.is_terminating, "the end comes from the new stream's own FIN");
        assert_eq!(2, seq.stale_dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_honour_its_own_terminating_frame_ahead_within_the_slack() -> anyhow::Result<()>
    {
        // The FIN overtook its two preceding data frames.
        let mut input: Vec<_> = (1..=197).map(|id| frame(id, false)).collect();
        input.extend([frame(200, true), frame(198, false), frame(199, false)]);
        let mut seq = futures::stream::iter(input).sequencer_with(windowed(1024, Duration::from_secs(30)));
        let mut last = None;
        while let Some(item) = seq.next().timeout(futures_time::time::Duration::from_secs(5)).await? {
            last = Some(item?);
        }
        let last = last.expect("frames");
        assert_eq!(200, last.0.frame_id);
        assert!(last.0.is_terminating);
        assert_eq!(0, seq.stale_dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_honour_a_terminating_frame_after_the_guard_period() -> anyhow::Result<()> {
        let mut cfg = windowed(1024, Duration::from_secs(30));
        cfg.successor_guard_period = Duration::from_millis(100);
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let mut seq = rx.sequencer_with(cfg);
        tx.unbounded_send(frame(1, false))?;
        assert_eq!(1, seq.try_next().await?.expect("frame").0.frame_id);
        tokio::time::sleep(Duration::from_millis(150)).await;
        tx.unbounded_send(frame(2, true))?;
        assert!(seq.try_next().await?.expect("frame").0.is_terminating);
        assert_eq!(0, seq.stale_dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_should_honour_a_terminating_frame_within_the_window() -> anyhow::Result<()> {
        // Frame 3 (in the window) is the new stream's last; the sequencer treats it as an ordinary
        // frame, so the socket aborts on it. Here we check it is not dropped.
        let (out, dropped) = run_windowed(3, Duration::from_secs(30), vec![9, 3, 2, 1]).await?;
        assert_eq!(vec![1, 2, 3], out.into_iter().collect::<Result<Vec<_>, _>>()?);
        assert_eq!(1, dropped);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn successor_sequencer_stale_frames_must_not_start_the_gap_timer() -> anyhow::Result<()> {
        let (seq_sink, seq_stream) = futures::channel::mpsc::unbounded();
        let seq_stream = seq_stream.sequencer_with(windowed(100, Duration::from_millis(50)));
        pin_mut!(seq_sink);
        pin_mut!(seq_stream);

        seq_sink.send(5000u32).await?;
        // Well past `max_wait`: a held stale frame would have been skipped to by now.
        assert!(
            seq_stream
                .try_next()
                .timeout(futures_time::time::Duration::from_millis(200))
                .await
                .is_err(),
            "a stale frame yields nothing"
        );
        assert_eq!(1, seq_stream.next_id, "a stale frame must not move the expected id");

        seq_sink.send(1u32).await?;
        assert_eq!(Some(1), seq_stream.try_next().await?);
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn sequencer_without_a_start_window_should_behave_as_before() -> anyhow::Result<()> {
        // With `None`, a high id is held, waited for and then skipped to, and the new stream's
        // low ids are then dropped as old: the very behaviour the window exists to avoid.
        let mut seq = futures::stream::iter(vec![5u32, 1, 2]).sequencer_with(SequencerConfig {
            max_wait: Duration::from_millis(20),
            capacity: 4096,
            max_item_age: None,
            max_frames_behind_gap: None,
            adapt_to_reordering: false,
            successor_start_window: None,
            successor_guard_period: Duration::ZERO,
        });
        let mut out = Vec::new();
        while let Some(item) = seq.next().await {
            out.push(item);
        }
        assert_eq!(0, seq.stale_dropped);
        // 1 and 2 are delivered in order (they were buffered), then 3 and 4 are reported lost, and 5 is emitted.
        let delivered: Vec<_> = out.iter().filter_map(|r| r.as_ref().ok().copied()).collect();
        assert_eq!(vec![1, 2, 5], delivered);
        assert_eq!(2, out.iter().filter(|r| r.is_err()).count());
        Ok(())
    }

    fn adaptive(max_wait: Duration) -> SequencerConfig {
        SequencerConfig {
            max_wait,
            capacity: 4096,
            max_item_age: None,
            max_frames_behind_gap: None,
            adapt_to_reordering: true,
            successor_start_window: None,
            successor_guard_period: Duration::ZERO,
        }
    }

    /// Feeds `late` after `early`, `apart` apart, as two frames that swapped places on the way.
    async fn reorder<S>(
        sink: &mut futures::channel::mpsc::UnboundedSender<u32>,
        seq: &mut std::pin::Pin<&mut S>,
        early: u32,
        late: u32,
        apart: Duration,
    ) -> anyhow::Result<()>
    where
        S: futures::Stream<Item = Result<u32, SessionError>>,
    {
        sink.unbounded_send(early)?;
        // The sequencer buffers `early` here and holds it behind the gap.
        assert!(
            tokio::time::timeout(apart, seq.next()).await.is_err(),
            "{early} must wait for {late}"
        );
        sink.unbounded_send(late)?;
        assert_eq!(Some(late), seq.next().await.transpose()?);
        assert_eq!(Some(early), seq.next().await.transpose()?);
        Ok(())
    }

    /// How long the sequencer takes to give up on `missing` once `behind` is waiting.
    async fn time_to_abandon<S>(
        sink: &mut futures::channel::mpsc::UnboundedSender<u32>,
        seq: &mut std::pin::Pin<&mut S>,
        missing: u32,
        behind: u32,
    ) -> anyhow::Result<Duration>
    where
        S: futures::Stream<Item = Result<u32, SessionError>>,
    {
        sink.unbounded_send(behind)?;
        let start = Instant::now();
        assert!(matches!(seq.next().await, Some(Err(SessionError::FrameDiscarded(id))) if id == missing));
        let took = start.elapsed();
        assert_eq!(Some(behind), seq.next().await.transpose()?);
        Ok(took)
    }

    /// Before any reordering is seen there is nothing to size the wait by, so a gap waits all of
    /// `max_wait`, as without the adaptation.
    #[test_log::test(tokio::test)]
    async fn an_adaptive_gap_wait_should_be_max_wait_before_any_reordering() -> anyhow::Result<()> {
        let max_wait = Duration::from_millis(600);
        let (mut sink, stream) = futures::channel::mpsc::unbounded();
        let seq = stream.sequencer_with(adaptive(max_wait));
        pin_mut!(seq);

        let took = time_to_abandon(&mut sink, &mut seq, 1, 2).await?;
        assert!(took >= max_wait - Duration::from_millis(20), "{took:?}");
        Ok(())
    }

    /// Frames that came 100 ms out of order: the next gap is given up after the 250 ms floor
    /// rather than the 5 s `max_wait`.
    #[test_log::test(tokio::test)]
    async fn an_adaptive_gap_wait_should_follow_the_reordering_seen() -> anyhow::Result<()> {
        let max_wait = Duration::from_secs(5);
        let (mut sink, stream) = futures::channel::mpsc::unbounded();
        let seq = stream.sequencer_sharing(adaptive(max_wait), Some(SharedReorder::warmed()));
        pin_mut!(seq);

        reorder(&mut sink, &mut seq, 2, 1, Duration::from_millis(100)).await?;
        let took = time_to_abandon(&mut sink, &mut seq, 3, 4).await?;
        assert!(took >= REORDER_FLOOR - Duration::from_millis(20), "{took:?}");
        assert!(took < Duration::from_secs(1), "{took:?}");
        Ok(())
    }

    /// Twice the deepest reordering, above the floor: 400 ms apart, an 800 ms wait.
    #[test_log::test(tokio::test)]
    async fn an_adaptive_gap_wait_should_be_twice_the_deepest_reordering() -> anyhow::Result<()> {
        let max_wait = Duration::from_secs(5);
        let (mut sink, stream) = futures::channel::mpsc::unbounded();
        let window = SharedReorder::warmed();
        window.record(Instant::now(), Duration::from_millis(400));
        let seq = stream.sequencer_sharing(adaptive(max_wait), Some(window));
        pin_mut!(seq);

        let took = time_to_abandon(&mut sink, &mut seq, 1, 2).await?;
        assert!(took >= Duration::from_millis(780), "{took:?}");
        assert!(took < Duration::from_millis(1500), "{took:?}");
        Ok(())
    }

    /// A frame that comes after its gap was given up is reordering the wait did not cover: it is
    /// dropped as before, and the wait grows to cover it.
    #[test_log::test(tokio::test)]
    async fn a_frame_after_its_abandoned_gap_should_lengthen_the_wait() -> anyhow::Result<()> {
        let max_wait = Duration::from_secs(5);
        let (mut sink, stream) = futures::channel::mpsc::unbounded();
        let seq = stream.sequencer_sharing(adaptive(max_wait), Some(SharedReorder::warmed()));
        pin_mut!(seq);

        reorder(&mut sink, &mut seq, 2, 1, Duration::from_millis(100)).await?;
        // Frame 3 is given up after the floor ...
        let took = time_to_abandon(&mut sink, &mut seq, 3, 4).await?;
        assert!(took < Duration::from_secs(1), "{took:?}");
        // ... and turns up 600 ms after frame 4 began waiting: dropped, but remembered.
        tokio::time::sleep(Duration::from_millis(600).saturating_sub(took)).await;
        sink.unbounded_send(3)?;
        assert!(
            tokio::time::timeout(Duration::from_millis(50), seq.next())
                .await
                .is_err(),
            "a late frame is not emitted"
        );

        let took = time_to_abandon(&mut sink, &mut seq, 5, 6).await?;
        assert!(took >= Duration::from_millis(1150), "{took:?}");
        assert!(took < Duration::from_secs(3), "{took:?}");
        Ok(())
    }
}
