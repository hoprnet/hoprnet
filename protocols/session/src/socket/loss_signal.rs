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

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// Records frame loss as recent activity, for a reader that has to act on it within seconds.
///
/// Shared between the socket that observes the loss and whatever wants to react to it. Recording is
/// two relaxed atomic operations on the loss path, so an observer that never asks costs nothing
/// beyond them.
#[derive(Debug)]
pub struct FrameLossSignal {
    /// The instant the millisecond stamps below count from.
    origin: Instant,
    /// When a frame was last lost, in milliseconds since [`origin`](Self::origin) plus one, so that
    /// `0` means "never". A sentinel rather than an `Option` in an atomic.
    last_loss_ms: AtomicU64,
    /// Frames lost since the socket was built, never reset.
    lost_total: AtomicU64,
}

impl Default for FrameLossSignal {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameLossSignal {
    /// Creates a signal that has observed no loss yet.
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            last_loss_ms: AtomicU64::new(0),
            lost_total: AtomicU64::new(0),
        }
    }

    /// Records that a frame was lost.
    ///
    /// A duplicate segment is not loss: the frame it belongs to was delivered.
    pub fn record(&self) {
        let elapsed = u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX - 1);
        // The previous stamp may be newer than this one when two loss events race; `fetch_max` keeps
        // the most recent instant, which is the one an observer is asking about.
        self.last_loss_ms
            .fetch_max(elapsed.saturating_add(1), Ordering::Relaxed);
        self.lost_total.fetch_add(1, Ordering::Relaxed);
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
    /// Unlike [`FrameLossReport`](super::loss_report)'s counters, this one is never reset by a
    /// window closing.
    pub fn lost_total(&self) -> u64 {
        self.lost_total.load(Ordering::Relaxed)
    }
}

// A per-Session signal is a handle, not a value: two Sessions never share one, and a config that
// carries a different signal is a config for a different Session.
impl PartialEq for FrameLossSignal {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for FrameLossSignal {}

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
            signal.lost_within(Duration::ZERO),
            "a loss recorded now is within a zero window"
        );
        assert!(signal.lost_within(Duration::from_millis(1)));
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
    fn every_loss_counts_and_the_most_recent_wins() {
        let signal = FrameLossSignal::new();
        signal.record();
        signal.record();
        signal.record();
        assert_eq!(3, signal.lost_total());
        assert!(signal.lost_within(Duration::ZERO));
    }
}
