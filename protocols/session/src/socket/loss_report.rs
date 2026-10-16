//! Summarises dropped frames and duplicate segments received by a
//! [`SessionSocket`](super::SessionSocket).
//!
//! Some frames arrive too late or have missing segments. Retried packets can also arrive twice.
//! Simply logging every drop or duplicate can hide other problems. This module counts these
//! events and logs an INFO summary only when frames were dropped or could not be reassembled.
//!
//! A "window" is the time covered by one summary. Each counted event checks whether enough time
//! has passed to finish the window. There is no background timer, so an idle socket waits until
//! the next event or until this report is dropped before logging its remaining counts.

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// Minimum time between summaries, except for the final summary when the report is dropped.
pub(crate) const FRAME_LOSS_REPORT_INTERVAL: Duration = Duration::from_secs(30);

/// Counts collected during one reporting period.
///
/// These counts measure different events. They are not mutually exclusive.
/// A failed reassembly and a later discard can refer to
/// the same frame, and duplicates count segments, not frames. Do not add them to calculate loss.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FrameLossSummary {
    /// Complete frames passed to the reader.
    pub emitted: u64,
    /// Frames skipped while putting received frames in order.
    /// E.g. Frame 10 is missing or too late, so delivery continues with frame 11.
    pub discarded: u64,
    /// Frames that could not be built from the received segments.
    /// E.g. A frame needs 3 segments but only 2 were arrived before timeout.
    pub not_reassembled: u64,
    /// Ignored segments whose frame ID is at or below the last frame passed to the reader.
    pub duplicates: u64,
    /// Time covered by these counts.
    pub window: Duration,
}

impl FrameLossSummary {
    /// Whether any frames were dropped or could not be reassembled.
    pub fn has_loss(&self) -> bool {
        self.discarded > 0 || self.not_reassembled > 0
    }

    fn log(&self, session_id: &str) {
        tracing::info!(
            session_id,
            emitted = self.emitted,
            discarded = self.discarded,
            not_reassembled = self.not_reassembled,
            duplicates = self.duplicates,
            window_s = self.window.as_secs(),
            "session frames dropped in the last window"
        );
    }
}

/// Counts received frames and duplicate segments, and logs summaries when there is frame loss.
///
/// The receive stages share this report. When its last owner drops it, it logs any remaining
/// counts if frames were dropped or could not be reassembled. Duplicate segments alone do not
/// indicate frame loss, but their counts are included as context in loss summaries.
#[derive(Debug)]
pub(crate) struct FrameLossReport {
    session_id: String,
    interval: Duration,
    emitted: AtomicU64,
    discarded: AtomicU64,
    not_reassembled: AtomicU64,
    duplicates: AtomicU64,
    window_start: parking_lot::Mutex<Instant>,
}

impl FrameLossReport {
    pub fn new(session_id: impl std::fmt::Display, interval: Duration) -> Self {
        Self {
            session_id: session_id.to_string(),
            interval,
            emitted: AtomicU64::new(0),
            discarded: AtomicU64::new(0),
            not_reassembled: AtomicU64::new(0),
            duplicates: AtomicU64::new(0),
            window_start: parking_lot::Mutex::new(Instant::now()),
        }
    }

    pub fn frame_emitted(&self) {
        self.emitted.fetch_add(1, Ordering::Relaxed);
        self.report_if_due();
    }

    pub fn frame_discarded(&self) {
        self.discarded.fetch_add(1, Ordering::Relaxed);
        self.report_if_due();
    }

    pub fn frame_not_reassembled(&self) {
        self.not_reassembled.fetch_add(1, Ordering::Relaxed);
        self.report_if_due();
    }

    pub fn duplicate_segment(&self) {
        self.duplicates.fetch_add(1, Ordering::Relaxed);
        self.report_if_due();
    }

    fn report_if_due(&self) {
        if let Some(summary) = self.close_window_if_due(Instant::now())
            && summary.has_loss()
        {
            summary.log(&self.session_id);
        }
    }

    /// Returns the counts and starts a new period if enough time has passed.
    ///
    /// If another call holds the lock, return without waiting. The counts stay available for
    /// this summary or the next one.
    fn close_window_if_due(&self, now: Instant) -> Option<FrameLossSummary> {
        let mut window_start = self.window_start.try_lock()?;
        let window = now.saturating_duration_since(*window_start);
        if window < self.interval {
            return None;
        }
        *window_start = now;
        Some(self.take_counts(window))
    }

    fn take_counts(&self, window: Duration) -> FrameLossSummary {
        FrameLossSummary {
            emitted: self.emitted.swap(0, Ordering::Relaxed),
            discarded: self.discarded.swap(0, Ordering::Relaxed),
            not_reassembled: self.not_reassembled.swap(0, Ordering::Relaxed),
            duplicates: self.duplicates.swap(0, Ordering::Relaxed),
            window,
        }
    }
}

impl Drop for FrameLossReport {
    fn drop(&mut self) {
        let window = self.window_start.get_mut().elapsed();
        let summary = self.take_counts(window);
        if summary.has_loss() {
            summary.log(&self.session_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    const INTERVAL: Duration = Duration::from_secs(30);

    #[test]
    fn window_is_not_closed_before_the_interval() {
        let report = FrameLossReport::new("test", INTERVAL);
        report.discarded.fetch_add(3, Ordering::Relaxed);

        let start = *report.window_start.lock();
        assert_eq!(report.close_window_if_due(start + INTERVAL / 2), None);
        assert_eq!(report.discarded.load(Ordering::Relaxed), 3, "counts must be kept");
    }

    #[test]
    fn closing_a_window_returns_and_resets_its_counts() -> anyhow::Result<()> {
        let report = FrameLossReport::new("test", INTERVAL);
        report.emitted.fetch_add(10, Ordering::Relaxed);
        report.discarded.fetch_add(2, Ordering::Relaxed);
        report.not_reassembled.fetch_add(1, Ordering::Relaxed);
        report.duplicates.fetch_add(4, Ordering::Relaxed);

        let start = *report.window_start.lock();
        let summary = report
            .close_window_if_due(start + INTERVAL)
            .context("window must close at the interval")?;

        assert_eq!(
            summary,
            FrameLossSummary {
                emitted: 10,
                discarded: 2,
                not_reassembled: 1,
                duplicates: 4,
                window: INTERVAL,
            }
        );
        assert!(summary.has_loss());

        // Reset the counts and start the next period where this one ended.
        assert_eq!(*report.window_start.lock(), start + INTERVAL);
        let next = report
            .close_window_if_due(start + 2 * INTERVAL)
            .context("second window must close")?;
        assert_eq!(
            next.emitted + next.discarded + next.not_reassembled + next.duplicates,
            0
        );
        Ok(())
    }

    #[test]
    fn a_window_with_only_delivered_frames_has_no_loss() {
        let summary = FrameLossSummary {
            emitted: 100,
            window: INTERVAL,
            ..Default::default()
        };
        assert!(!summary.has_loss());
    }

    #[test]
    fn duplicate_segments_do_not_indicate_frame_loss() {
        let summary = FrameLossSummary {
            emitted: 100,
            duplicates: 4,
            window: INTERVAL,
            ..Default::default()
        };
        assert!(!summary.has_loss());
    }

    #[test]
    fn discarded_or_unreassembled_frames_indicate_loss() {
        for (discarded, not_reassembled) in [(1, 0), (0, 1)] {
            let summary = FrameLossSummary {
                discarded,
                not_reassembled,
                window: INTERVAL,
                ..Default::default()
            };
            assert!(summary.has_loss());
        }
    }

    #[test]
    fn counting_methods_update_their_own_counter() {
        let report = FrameLossReport::new("test", INTERVAL);
        report.frame_emitted();
        report.frame_discarded();
        report.frame_discarded();
        report.frame_not_reassembled();
        report.duplicate_segment();

        let summary = report.take_counts(Duration::ZERO);
        assert_eq!(summary.emitted, 1);
        assert_eq!(summary.discarded, 2);
        assert_eq!(summary.not_reassembled, 1);
        assert_eq!(summary.duplicates, 1);
    }
}
