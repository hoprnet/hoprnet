//! Exit-side collection of the packet tags that go back to the Entry as forward receipts.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use hopr_protocol_start::MAX_FORWARD_RECEIPT_TAGS;

/// Packets that make a receipt due, together with [`RECEIPT_MIN_INTERVAL`].
pub(crate) const RECEIPT_MIN_PACKETS: usize = 50;
/// The least time between two receipts; a half-full interval is reported at once.
pub(crate) const RECEIPT_MIN_INTERVAL: Duration = Duration::from_millis(250);
/// Quiet time before a packet opens a burst window.
const BURST_QUIET: Duration = Duration::from_secs(1);
/// How long a burst window stays open.
const BURST_WINDOW: Duration = Duration::from_secs(1);
/// The least time between the starts of two windows, so that a trickle of packets does not keep one open.
const BURST_COOLDOWN: Duration = Duration::from_secs(10);
/// Packets and time that make a receipt due inside a burst window.
const BURST_MIN_PACKETS: usize = 10;
const BURST_MIN_INTERVAL: Duration = Duration::from_millis(100);
/// How often the sender polls while a burst window is open.
pub(crate) const BURST_POLL: Duration = Duration::from_millis(50);
/// Tags held per interval; a receipt repeats as many intervals as fit in [`MAX_FORWARD_RECEIPT_TAGS`].
const INTERVAL_TAGS: usize = MAX_FORWARD_RECEIPT_TAGS / 2;
/// Tags in an interval that make a receipt due: the rest is room for the packets that arrive before it is sent.
const WAKE_TAGS: usize = INTERVAL_TAGS / 2;
/// How much `seq` jumps when an interval overflowed, so that the Entry sees the seqs it cannot be told about.
const OVERFLOW_SEQ_JUMP: u32 = 3;

/// A receipt: the tags of the intervals `first_seq..=seq`.
#[derive(Debug)]
pub(crate) struct Receipt {
    pub seq: u32,
    pub first_seq: u32,
    pub tags: Vec<u32>,
}

struct State {
    current: Vec<u32>,
    /// The closed intervals that the next receipt may still repeat, oldest first.
    closed: VecDeque<(u32, Vec<u32>)>,
    overflowed: bool,
    seq: u32,
    last_sent: Instant,
    last_note: Option<Instant>,
    /// The end of the burst window: the first receipts after a quiet period are the ones that judge.
    burst_until: Option<Instant>,
    burst_started: Option<Instant>,
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
                closed: VecDeque::new(),
                overflowed: false,
                seq: 0,
                last_sent: now,
                last_note: None,
                burst_until: None,
                burst_started: None,
            }),
        }
    }

    /// The signal sent when an interval is half full or a burst window opens, so that its receipt does not wait
    /// for the poll. Taken once.
    pub(crate) fn take_wake(&self) -> Option<futures::channel::mpsc::Receiver<()>> {
        self.woken.lock().take()
    }

    /// Notes the tag of a received Session packet.
    pub(crate) fn note(&self, tag: u32) {
        self.note_at(tag, Instant::now());
    }

    /// Whether a burst window is open at `now`.
    pub(crate) fn burst_open_at(&self, now: Instant) -> bool {
        self.state.lock().burst_until.is_some_and(|until| now < until)
    }

    fn note_at(&self, tag: u32, now: Instant) {
        let mut state = self.state.lock();
        // A packet after a quiet period (not a Session's first) opens a burst window, but not within the cooldown of
        // the last one
        let burst_opened = state
            .last_note
            .is_some_and(|last| now.saturating_duration_since(last) >= BURST_QUIET)
            && state
                .burst_started
                .is_none_or(|started| now.saturating_duration_since(started) >= BURST_COOLDOWN);
        if burst_opened {
            state.burst_until = Some(now + BURST_WINDOW);
            state.burst_started = Some(now);
        }
        state.last_note = Some(now);
        let full = state.current.len() >= INTERVAL_TAGS;
        if full {
            state.overflowed = true;
        } else {
            state.current.push(tag);
        }
        let wake = burst_opened || (!full && state.current.len() == WAKE_TAGS);
        drop(state);
        if wake {
            // A full channel already holds a wake-up
            let _ = self.wake.lock().try_send(());
        }
    }

    /// Returns the next receipt if one is due at `now`.
    pub(crate) fn take_due(&self, now: Instant) -> Option<Receipt> {
        let mut state = self.state.lock();
        let (min_packets, min_interval) = if state.burst_until.is_some_and(|until| now < until) {
            (BURST_MIN_PACKETS, BURST_MIN_INTERVAL)
        } else {
            (RECEIPT_MIN_PACKETS, RECEIPT_MIN_INTERVAL)
        };
        let due = state.current.len() >= WAKE_TAGS
            || (state.current.len() >= min_packets && now.duration_since(state.last_sent) >= min_interval);
        if !due {
            return None;
        }

        if state.overflowed {
            // Repeating older intervals would cover the skipped seqs and hide the overflow
            state.closed.clear();
            state.seq = state.seq.wrapping_add(OVERFLOW_SEQ_JUMP);
        } else {
            state.seq = state.seq.wrapping_add(1);
        }
        let (seq, current) = (state.seq, std::mem::take(&mut state.current));
        state.closed.push_back((seq, current));
        state.overflowed = false;
        state.last_sent = now;

        // The newest intervals that fit; older ones are not needed by any later receipt either
        let mut total: usize = state.closed.iter().map(|(_, tags)| tags.len()).sum();
        while total > MAX_FORWARD_RECEIPT_TAGS {
            let Some((_, oldest)) = state.closed.pop_front() else {
                break;
            };
            total -= oldest.len();
        }
        let first_seq = state.closed.front().map_or(seq, |(first, _)| *first);
        let tags = state.closed.iter().flat_map(|(_, tags)| tags.iter().copied()).collect();
        Some(Receipt { seq, first_seq, tags })
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
        let receipt = receipts.take_due(t0 + Duration::from_millis(250)).context("due")?;
        assert_eq!((1, 1, 50), (receipt.seq, receipt.first_seq, receipt.tags.len()));
        Ok(())
    }

    #[test]
    fn a_half_full_interval_should_make_a_receipt_due_at_once() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        for tag in 0..WAKE_TAGS as u32 {
            receipts.note(tag);
        }
        let receipt = receipts.take_due(t0).context("due")?;
        assert_eq!((1, WAKE_TAGS), (receipt.seq, receipt.tags.len()));
        Ok(())
    }

    /// `count` receipts of `per_interval` tags each, 250 ms apart.
    fn receipts_of(per_interval: u32, count: u32) -> anyhow::Result<Vec<Receipt>> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        let mut sent = Vec::new();
        for i in 0..count {
            for tag in i * per_interval..(i + 1) * per_interval {
                receipts.note(tag);
            }
            sent.push(receipts.take_due(t0 + RECEIPT_MIN_INTERVAL * (i + 1)).context("due")?);
        }
        Ok(sent)
    }

    /// At the tests' rate (75 tags per interval) a receipt repeats 10 intervals.
    #[test]
    fn a_receipt_should_repeat_as_many_intervals_as_fit() -> anyhow::Result<()> {
        let sent = receipts_of(75, 12)?;
        assert_eq!((1, 1), (sent[0].seq, sent[0].first_seq));
        assert_eq!((3, 1), (sent[2].seq, sent[2].first_seq), "everything so far");
        let last = &sent[11];
        assert_eq!((12, 3), (last.seq, last.first_seq), "10 intervals");
        assert_eq!((150..900).collect::<Vec<_>>(), last.tags);
        Ok(())
    }

    /// At a high rate (192-tag intervals) it repeats 4.
    #[test]
    fn a_receipt_at_a_high_rate_should_repeat_four_intervals() -> anyhow::Result<()> {
        let last = receipts_of(WAKE_TAGS as u32, 8)?.pop().context("receipts")?;
        assert_eq!(
            (8, 5, MAX_FORWARD_RECEIPT_TAGS),
            (last.seq, last.first_seq, last.tags.len())
        );
        Ok(())
    }

    #[test]
    fn an_overflow_should_make_the_seq_jump_and_drop_the_older_intervals() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        for tag in 0..RECEIPT_MIN_PACKETS as u32 {
            receipts.note(tag);
        }
        receipts.take_due(t0 + RECEIPT_MIN_INTERVAL).context("due")?;

        for tag in 0..INTERVAL_TAGS as u32 + 1 {
            receipts.note(tag);
        }
        let receipt = receipts.take_due(t0 + RECEIPT_MIN_INTERVAL * 2).context("due")?;
        assert_eq!(
            (1 + OVERFLOW_SEQ_JUMP, 1 + OVERFLOW_SEQ_JUMP),
            (receipt.seq, receipt.first_seq)
        );
        assert_eq!(
            INTERVAL_TAGS,
            receipt.tags.len(),
            "the tag beyond the interval is not kept"
        );

        for tag in 0..RECEIPT_MIN_PACKETS as u32 {
            receipts.note(tag);
        }
        let next = receipts.take_due(t0 + RECEIPT_MIN_INTERVAL * 3).context("due")?;
        assert_eq!(
            (2 + OVERFLOW_SEQ_JUMP, 1 + OVERFLOW_SEQ_JUMP),
            (next.seq, next.first_seq)
        );
        assert!(next.tags.len() <= MAX_FORWARD_RECEIPT_TAGS);
        Ok(())
    }

    /// 10 packets after a quiet second: the first packet of a Session does not open a window, the next one does.
    #[test]
    fn a_packet_after_a_quiet_second_should_open_a_burst_window() {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        receipts.note_at(0, t0);
        assert!(!receipts.burst_open_at(t0), "a Session's first packet opens none");
        receipts.note_at(1, t0 + Duration::from_millis(900));
        assert!(
            !receipts.burst_open_at(t0 + Duration::from_millis(900)),
            "not quiet for a second"
        );
        let after = t0 + Duration::from_millis(2_500);
        receipts.note_at(2, after);
        assert!(receipts.burst_open_at(after + Duration::from_millis(900)));
        assert!(!receipts.burst_open_at(after + BURST_WINDOW));
    }

    #[test]
    fn a_receipt_should_be_due_at_10_packets_and_100_ms_inside_a_burst_window_only() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let start = t0 + Duration::from_secs(2);
        // The first packet (tag 0) opens no window; the others do, and make 9 or 10 in all
        for (others, due) in [(8, false), (9, true)] {
            let receipts = receipts_at(start);
            receipts.note_at(0, t0);
            for tag in 1..=others {
                receipts.note_at(tag, start);
            }
            assert!(
                receipts.take_due(start + Duration::from_millis(99)).is_none(),
                "too early"
            );
            assert_eq!(due, receipts.take_due(start + Duration::from_millis(100)).is_some());
        }

        // Without a window the 50 packets and 250 ms apply
        let receipts = receipts_at(start);
        for tag in 0..10 {
            receipts.note_at(tag, start);
        }
        assert!(
            receipts.take_due(start + Duration::from_secs(1)).is_none(),
            "10 packets are too few"
        );
        for tag in 0..40 {
            receipts.note_at(tag, start);
        }
        assert!(
            receipts.take_due(start + Duration::from_millis(249)).is_none(),
            "too early"
        );
        assert!(receipts.take_due(start + Duration::from_millis(250)).is_some());
        Ok(())
    }

    /// A packet per 1.1 s opens one window per 10 s, not one per packet.
    #[test]
    fn a_trickle_should_open_at_most_one_window_per_cooldown() {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        let mut windows = 0;
        for i in 0..30 {
            let now = t0 + Duration::from_millis(1_100 * i);
            receipts.note_at(i as u32, now);
            windows += usize::from(receipts.burst_open_at(now));
        }
        assert_eq!(3, windows, "30 packets over 33 s");
    }

    #[test]
    fn a_window_should_wake_the_sender_even_into_a_full_interval() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        let mut woken = receipts.take_wake().context("wake")?;
        for tag in 0..INTERVAL_TAGS as u32 {
            receipts.note_at(tag, t0);
        }
        while woken.try_recv().is_ok() {}
        receipts.note_at(0, t0 + Duration::from_secs(2));
        assert!(woken.try_recv().is_ok(), "a window opens into a full interval");
        Ok(())
    }

    #[test]
    fn a_burst_window_should_wake_the_sender() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let receipts = receipts_at(t0);
        let mut woken = receipts.take_wake().context("wake")?;
        receipts.note_at(0, t0);
        assert!(woken.try_recv().is_err(), "the first packet wakes nothing");
        receipts.note_at(1, t0 + Duration::from_secs(2));
        assert!(woken.try_recv().is_ok(), "a window opens");
        Ok(())
    }
}
