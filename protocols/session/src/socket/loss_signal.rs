//! A live signal of frame loss that a Session's owner can read.
//!
//! [`FrameLossReport`](super::loss_report) counts and logs loss per window. That is the right shape
//! for a log line and the wrong one for a decision: the window closes every
//! [`FRAME_LOSS_REPORT_INTERVAL`](super::loss_report::FRAME_LOSS_REPORT_INTERVAL), so a reader would
//! learn about a lost frame up to half a minute after the fact, and a Session whose loss stopped
//! would still look lossy for the rest of the window.
//!
//! This signal is that reader's view, and it exists because the Exit has a reason to act on loss
//! within seconds: while it is not reassembling frames, the only replies that reach the Entry are its
//! own SURB-level reports, and one sent every `surb_balance_notify_period` (15 s by default) puts the
//! Entry's verdict on a dead relayer twelve to twenty-eight seconds after the kill instead of about
//! eleven. Reporting faster while the Exit is losing frames is what closes that gap, and this is what
//! tells it that it is.
//!
//! Unlike the log window, the signal is never reset: a loss is remembered until it ages past whatever
//! window an observer asks about.
//!
//! # Stalls
//!
//! A loss is known late. A frame is lost when its reassembly times out, `frame_timeout` after its last
//! segment arrived, and while segments keep arriving the reassembler looks for timed-out frames only
//! once per timeout: three to six seconds at the Exit's 3 s default before the first loss of a forward
//! path that went dark. The signal therefore also follows the frames as they arrive. Segments of two
//! new frames since a frame last completed or was lost, with none completing for a while, is a forward
//! path that delivers segments but no frames, and that shows within a second ([`stalled_for`]). A single
//! frame arriving slowly is not a stall: the next frame starts only once its own segments are all sent.
//! A sender that stops while frames are stalled does not end the stall, since a sender commonly stops
//! exactly because nothing comes back; the frames' timeouts end it, as a loss.
//!
//! [`stalled_for`]: FrameLossSignal::stalled_for

use std::{
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// Records frame loss as recent activity, for a reader that has to act on it within seconds.
///
/// Shared between the socket that observes the loss and whatever wants to react to it. Recording is
/// two relaxed atomic operations on the loss path, so an observer that never asks costs nothing
/// beyond them.
///
/// # One writer, any readers
///
/// The readers, [`lost_within`](Self::lost_within), [`lost_total`](Self::lost_total) and
/// [`stalled_for`](Self::stalled_for), may be called from anywhere at any time. The writers,
/// [`segment_received`](Self::segment_received), [`frame_completed`](Self::frame_completed) and
/// [`record`](Self::record), must be called by one writer at a time. The stall state is three atomics
/// that a writer updates one after another, and `segment_received` decides from one of them before it
/// updates the next, so two writers interleaving could leave a stall that no frame matches, reported
/// until the next completion or loss. A socket calls all three from its one sequentially polled stream
/// chain, which is that single writer. A reader that catches a writer midway sees a stall begin or end
/// one update late, no worse.
#[derive(Debug, smart_default::SmartDefault)]
pub struct FrameLossSignal {
    /// The instant the millisecond stamps below count from.
    #[default(Instant::now())]
    origin: Instant,
    /// When a frame was last lost, in milliseconds since [`origin`](Self::origin) plus one, so that
    /// `0` means "never". A sentinel rather than an `Option` in an atomic.
    last_loss_ms: AtomicU64,
    /// Frames lost since the socket was built, never reset.
    lost_total: AtomicU64,
    /// The highest frame id a segment has carried so far.
    highest_frame_id: AtomicU32,
    /// Frames whose first segment arrived since a frame last completed or was lost.
    frames_started: AtomicU32,
    /// When the second of [`frames_started`](Self::frames_started) began, in milliseconds since
    /// [`origin`](Self::origin) plus one; `0` while fewer than two have.
    stalled_since_ms: AtomicU64,
}

impl FrameLossSignal {
    /// Creates a signal that has observed no loss yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Milliseconds since [`origin`](Self::origin), plus one so that a stamp is never `0`.
    fn stamp(&self) -> u64 {
        u64::try_from(self.origin.elapsed().as_millis())
            .unwrap_or(u64::MAX - 1)
            .saturating_add(1)
    }

    /// Records that a frame was lost, which also ends a stall: its frames have timed out.
    ///
    /// A duplicate segment is not loss: the frame it belongs to was delivered.
    pub fn record(&self) {
        let elapsed = u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX - 1);
        // The previous stamp may be newer than this one when two loss events race; `fetch_max` keeps
        // the most recent instant, which is the one an observer is asking about.
        self.last_loss_ms
            .fetch_max(elapsed.saturating_add(1), Ordering::Relaxed);
        self.lost_total.fetch_add(1, Ordering::Relaxed);
        self.end_stall();
    }

    /// Whether a frame was lost no longer than `window` ago.
    ///
    /// A loss recorded on the millisecond `window` back still counts; the granularity is a
    /// millisecond, because that is all this signal keeps.
    pub fn lost_within(&self, window: Duration) -> bool {
        let last = self.last_loss_ms.load(Ordering::Relaxed);
        if last == 0 {
            return false;
        }
        let now = u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX - 1);
        // The stamp carries a one-millisecond offset, so the instant of the loss is `last - 1`.
        now.saturating_sub(last - 1) <= u64::try_from(window.as_millis()).unwrap_or(u64::MAX)
    }

    /// Frames lost since this signal was created.
    ///
    /// Unlike the socket's per-window log summary, this count is never reset: it is the total for
    /// the life of the socket.
    pub fn lost_total(&self) -> u64 {
        self.lost_total.load(Ordering::Relaxed)
    }

    /// Records that a segment of `frame_id` arrived (a duplicate of a delivered frame is not one).
    ///
    /// The first segment of a frame newer than any seen so far starts that frame; the second frame to
    /// start since a frame last completed starts a stall (see [`stalled_for`](Self::stalled_for)).
    pub fn segment_received(&self, frame_id: u32) {
        if self.highest_frame_id.fetch_max(frame_id, Ordering::Relaxed) < frame_id
            && self.frames_started.fetch_add(1, Ordering::Relaxed) == 1
        {
            self.stalled_since_ms.store(self.stamp(), Ordering::Relaxed);
        }
    }

    /// Records that a frame completed its reassembly, which ends a stall.
    pub fn frame_completed(&self) {
        self.end_stall();
    }

    fn end_stall(&self) {
        self.frames_started.store(0, Ordering::Relaxed);
        self.stalled_since_ms.store(0, Ordering::Relaxed);
    }

    /// Whether frames have stalled for at least `after`: segments of two frames started since a frame
    /// last completed or was lost, the second of them at least `after` ago, and none has completed or
    /// been lost since.
    ///
    /// A stall lasts until a frame completes or one is lost, however long the segments pause: a sender
    /// commonly stops because nothing comes back, and the frames' timeouts end the stall as a loss.
    pub fn stalled_for(&self, after: Duration) -> bool {
        let since = self.stalled_since_ms.load(Ordering::Relaxed);
        since != 0 && self.stamp().saturating_sub(since) >= u64::try_from(after.as_millis()).unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_signal_has_no_loss() {
        let signal = FrameLossSignal::new();
        assert!(!signal.lost_within(Duration::from_secs(1)));
        assert!(!signal.lost_within(Duration::from_secs(3600)));
        assert_eq!(0, signal.lost_total());
    }

    #[test]
    fn a_recorded_loss_is_seen_inside_its_window() {
        let signal = FrameLossSignal::new();
        signal.record();
        assert!(
            signal.lost_within(Duration::from_millis(500)),
            "a loss recorded now is within a small window"
        );
        assert!(signal.lost_within(Duration::from_millis(1000)));
        assert!(signal.lost_within(Duration::from_secs(3)));
        assert_eq!(1, signal.lost_total());
    }

    #[test]
    fn a_loss_ages_out_of_the_window() {
        let window = Duration::from_millis(30);
        let signal = FrameLossSignal::new();
        signal.record();
        std::thread::sleep(window * 2);
        assert!(
            !signal.lost_within(window),
            "a loss older than the window must not be reported as recent"
        );
        assert_eq!(1, signal.lost_total(), "aging out does not forget the loss");
    }

    #[test]
    fn frames_that_do_not_complete_while_segments_arrive_are_a_stall() {
        let after = Duration::from_millis(30);
        let signal = FrameLossSignal::new();
        signal.segment_received(1);
        signal.segment_received(2);
        assert!(!signal.stalled_for(after), "not before `after` has passed");

        std::thread::sleep(after + Duration::from_millis(5));
        signal.segment_received(3);
        assert!(
            signal.stalled_for(after),
            "two frames started, none completed for `after`"
        );
        assert_eq!(0, signal.lost_total(), "a stall is not a loss");
    }

    #[test]
    fn a_completed_frame_ends_a_stall() {
        let after = Duration::from_millis(30);
        let signal = FrameLossSignal::new();
        signal.segment_received(1);
        signal.segment_received(2);
        std::thread::sleep(after + Duration::from_millis(5));
        signal.segment_received(2);
        assert!(signal.stalled_for(after));

        signal.frame_completed();
        assert!(!signal.stalled_for(after));
        signal.segment_received(3);
        assert!(
            !signal.stalled_for(after),
            "one new frame since the completion is not a stall"
        );
    }

    #[test]
    fn a_single_slow_frame_is_not_a_stall() {
        let after = Duration::from_millis(30);
        let signal = FrameLossSignal::new();
        signal.segment_received(1);
        std::thread::sleep(after + Duration::from_millis(5));
        signal.segment_received(1);
        assert!(!signal.stalled_for(after), "segments of one frame only");
    }

    #[test]
    fn a_stall_outlasts_a_pause_and_ends_with_a_loss() {
        let after = Duration::from_millis(30);
        let signal = FrameLossSignal::new();
        signal.segment_received(1);
        signal.segment_received(2);
        std::thread::sleep(after * 3);
        assert!(
            signal.stalled_for(after),
            "no segment since, but nothing completed either: the sender stopped because nothing came back"
        );

        signal.record();
        assert!(!signal.stalled_for(after), "the stalled frames timed out, as a loss");
        assert!(signal.lost_within(after));
    }

    #[test]
    fn every_loss_counts_and_the_most_recent_wins() {
        let signal = FrameLossSignal::new();
        signal.record();
        signal.record();
        signal.record();
        assert_eq!(3, signal.lost_total());
        assert!(signal.lost_within(Duration::from_millis(500)));
    }
}
