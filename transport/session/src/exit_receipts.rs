//! Exit-side collection of the packet tags that go back to the Entry as forward receipts.

use std::time::{Duration, Instant};

use hopr_protocol_start::MAX_FORWARD_RECEIPT_TAGS;

/// Packets that make a receipt due, together with [`RECEIPT_MIN_INTERVAL`].
pub(crate) const RECEIPT_MIN_PACKETS: usize = 50;
/// The least time between two receipts; a half-full interval is reported at once.
pub(crate) const RECEIPT_MIN_INTERVAL: Duration = Duration::from_millis(250);
/// Tags held per interval; a receipt carries the previous and the current one.
const INTERVAL_TAGS: usize = MAX_FORWARD_RECEIPT_TAGS / 2;
/// Tags in an interval that make a receipt due: the rest is room for the packets that arrive before it is sent.
const WAKE_TAGS: usize = INTERVAL_TAGS / 2;
/// How much `seq` jumps when an interval overflowed, so that the Entry discards the evidence.
const OVERFLOW_SEQ_JUMP: u32 = 3;

struct State {
    current: Vec<u32>,
    previous: Vec<u32>,
    overflowed: bool,
    seq: u32,
    last_sent: Instant,
}

/// Tags of the Session packets received by an Exit, and when to report them.
pub(crate) struct ExitReceipts {
    state: parking_lot::Mutex<State>,
    wake: parking_lot::Mutex<futures::channel::mpsc::Sender<()>>,
    woken: parking_lot::Mutex<Option<futures::channel::mpsc::Receiver<()>>>,
}

impl ExitReceipts {
    pub(crate) fn new(now: Instant) -> Self {
        let (wake, woken) = futures::channel::mpsc::channel(1);
        Self {
            wake: parking_lot::Mutex::new(wake),
            woken: parking_lot::Mutex::new(Some(woken)),
            state: parking_lot::Mutex::new(State {
                current: Vec::new(),
                previous: Vec::new(),
                overflowed: false,
                seq: 0,
                last_sent: now,
            }),
        }
    }

    /// The signal sent when an interval is half full, so that its receipt does not wait for the poll. Taken once.
    pub(crate) fn take_wake(&self) -> Option<futures::channel::mpsc::Receiver<()>> {
        self.woken.lock().take()
    }

    /// Notes the tag of a received Session packet.
    pub(crate) fn note(&self, tag: u32) {
        let mut state = self.state.lock();
        if state.current.len() >= INTERVAL_TAGS {
            state.overflowed = true;
            return;
        }
        state.current.push(tag);
        let half_full = state.current.len() == WAKE_TAGS;
        drop(state);
        if half_full {
            // A full channel already holds a wake-up
            let _ = self.wake.lock().try_send(());
        }
    }

    /// Returns the next receipt (its `seq` and tags) if one is due at `now`.
    pub(crate) fn take_due(&self, now: Instant) -> Option<(u32, Vec<u32>)> {
        let mut state = self.state.lock();
        let due = state.current.len() >= WAKE_TAGS
            || (state.current.len() >= RECEIPT_MIN_PACKETS
                && now.duration_since(state.last_sent) >= RECEIPT_MIN_INTERVAL);
        if !due {
            return None;
        }

        state.seq = state
            .seq
            .wrapping_add(if state.overflowed { OVERFLOW_SEQ_JUMP } else { 1 });
        let tags = [state.previous.as_slice(), state.current.as_slice()].concat();
        state.previous = std::mem::take(&mut state.current);
        state.overflowed = false;
        state.last_sent = now;
        Some((state.seq, tags))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    fn receipts_at(now: Instant) -> ExitReceipts {
        ExitReceipts::new(now)
    }

    #[test]
    fn a_receipt_should_be_due_after_50_packets_and_250_ms() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        for tag in 0..49 {
            receipts.note(tag);
        }
        assert!(
            receipts.take_due(t0 + Duration::from_secs(1)).is_none(),
            "49 packets are too few"
        );
        receipts.note(49);
        assert!(
            receipts.take_due(t0 + Duration::from_millis(249)).is_none(),
            "too early"
        );
        let (seq, tags) = receipts.take_due(t0 + Duration::from_millis(250)).context("due")?;
        assert_eq!(1, seq);
        assert_eq!(50, tags.len());
        Ok(())
    }

    #[test]
    fn a_half_full_interval_should_make_a_receipt_due_at_once() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        for tag in 0..WAKE_TAGS as u32 {
            receipts.note(tag);
        }
        let (seq, tags) = receipts.take_due(t0).context("due")?;
        assert_eq!((1, WAKE_TAGS), (seq, tags.len()));
        Ok(())
    }

    #[test]
    fn consecutive_receipts_should_overlap_and_count_up() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        let mut now = t0;
        let mut next_tag = 0;
        let mut sent = Vec::new();
        for _ in 0..3 {
            for _ in 0..RECEIPT_MIN_PACKETS {
                receipts.note(next_tag);
                next_tag += 1;
            }
            now += RECEIPT_MIN_INTERVAL;
            sent.push(receipts.take_due(now).context("due")?);
        }
        assert_eq!([1, 2, 3], [sent[0].0, sent[1].0, sent[2].0]);
        assert_eq!((0..50).collect::<Vec<_>>(), sent[0].1);
        assert_eq!((0..100).collect::<Vec<_>>(), sent[1].1, "previous + current");
        assert_eq!((50..150).collect::<Vec<_>>(), sent[2].1, "the oldest interval has gone");
        Ok(())
    }

    #[test]
    fn an_overflow_should_make_the_seq_jump() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        for tag in 0..INTERVAL_TAGS as u32 + 1 {
            receipts.note(tag);
        }
        let (seq, tags) = receipts.take_due(t0).context("due")?;
        assert_eq!(OVERFLOW_SEQ_JUMP, seq);
        assert_eq!(INTERVAL_TAGS, tags.len(), "the tag beyond the interval is not kept");

        for tag in 0..RECEIPT_MIN_PACKETS as u32 {
            receipts.note(tag);
        }
        let (seq, tags) = receipts.take_due(t0 + RECEIPT_MIN_INTERVAL).context("due")?;
        assert_eq!(
            OVERFLOW_SEQ_JUMP + 1,
            seq,
            "a receipt after the overflow counts up again"
        );
        assert!(tags.len() <= MAX_FORWARD_RECEIPT_TAGS);
        Ok(())
    }
}
