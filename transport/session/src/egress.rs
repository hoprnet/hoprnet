//! Local egress congestion: how long outgoing packets wait before they reach the wire.
//!
//! Measured by the transport, which owns the egress queues, and read by Session-level control
//! loops, which own the traffic that can be held back. It lives here rather than in the transport so
//! the SURB balancer can read it without a dependency cycle.
//!
//! Nothing upstream of the per-peer queues sees the wire rate: the Session sink, the mixer and the
//! wire channel all accept far faster than a constrained uplink drains, so a producer that outruns
//! the uplink fills local queues for seconds before any of them pushes back. Queueing delay at the
//! last queue before the wire is the first place where that becomes visible.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

/// Queueing delay above which a packet counts as having waited behind a standing queue.
///
/// Far above what a draining queue imposes -- a healthy per-peer channel hands packets to the
/// writer within a few milliseconds -- and far below the multi-second delays a saturated uplink
/// builds up.
pub const EGRESS_TARGET_SOJOURN: Duration = Duration::from_millis(50);

/// How long queueing delay must stay above [`EGRESS_TARGET_SOJOURN`] before egress counts as
/// congested.
///
/// A queue that only absorbs bursts drains back below target between them, while one fed faster
/// than the wire takes never does. Requiring the delay to persist is what tells the two apart, as
/// in CoDel.
pub const EGRESS_CONGESTION_INTERVAL: Duration = Duration::from_millis(100);

/// Shared record of local egress congestion.
///
/// Written by any number of egress queues and read by any number of control loops. It keeps the
/// time congestion was last seen rather than a flag: nothing is guaranteed to come back and clear a
/// flag, whereas a timestamp lets each reader decide how recent is recent enough.
#[derive(Debug)]
pub struct EgressPressure {
    origin: Instant,
    /// Nanoseconds after `origin`, plus one, at which congestion was last seen. Zero means never.
    ///
    /// Nanoseconds so the reconstructed instant is the one reported rather than one truncated
    /// towards `origin`, which would end a window early. A `u64` of them spans centuries.
    last_congested_ns: AtomicU64,
    /// Queueing delay of the most recently dequeued packet, in microseconds.
    last_sojourn_us: AtomicU64,
}

impl Default for EgressPressure {
    fn default() -> Self {
        Self::new()
    }
}

impl EgressPressure {
    /// Creates a record in which egress has never been congested.
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            last_congested_ns: AtomicU64::new(0),
            last_sojourn_us: AtomicU64::new(0),
        }
    }

    /// Records that egress was congested at `now`.
    ///
    /// Only ever moves the record forward, so a writer reporting a slightly older instant cannot
    /// hide a more recent congestion another writer already reported.
    pub fn mark_congested(&self, now: Instant) {
        let since_origin = now.saturating_duration_since(self.origin).as_nanos();
        let at = u64::try_from(since_origin).unwrap_or(u64::MAX - 1).saturating_add(1);
        self.last_congested_ns.fetch_max(at, Ordering::Relaxed);
    }

    /// Whether egress was congested at any point during the `window` before `now`.
    pub fn is_congested_within(&self, now: Instant, window: Duration) -> bool {
        match self.last_congested_ns.load(Ordering::Relaxed) {
            0 => false,
            at => {
                let last = self.origin + Duration::from_nanos(at - 1);
                now.saturating_duration_since(last) <= window
            }
        }
    }

    /// Queueing delay of the most recently dequeued packet, from whichever queue dequeued last.
    pub fn last_sojourn(&self) -> Duration {
        Duration::from_micros(self.last_sojourn_us.load(Ordering::Relaxed))
    }

    fn record_sojourn(&self, sojourn: Duration) {
        let micros = u64::try_from(sojourn.as_micros()).unwrap_or(u64::MAX);
        self.last_sojourn_us.store(micros, Ordering::Relaxed);
    }
}

/// Standing-queue detector for a single egress queue, feeding an [`EgressPressure`].
///
/// One per queue, owned by whatever dequeues from it. The run of delayed packets it tracks belongs
/// to one queue: interleaving several queues' packets would let one queue's promptly drained
/// packets hide another's standing queue.
///
/// It only sees packets that leave the queue, so a writer that has stopped completely produces no
/// observations at all. The producer side has to report that case itself, when it finds the queue
/// full.
#[derive(Debug)]
pub struct SojournTracker {
    pressure: Arc<EgressPressure>,
    /// When the current run of above-target packets began, if one is running.
    above_since: Option<Instant>,
}

impl SojournTracker {
    /// Creates a tracker reporting into `pressure`.
    pub fn new(pressure: Arc<EgressPressure>) -> Self {
        Self {
            pressure,
            above_since: None,
        }
    }

    /// Records one packet that entered the queue at `enqueued_at` and left it at `now`.
    ///
    /// Returns how long the packet waited.
    pub fn observe(&mut self, enqueued_at: Instant, now: Instant) -> Duration {
        let sojourn = now.saturating_duration_since(enqueued_at);
        self.pressure.record_sojourn(sojourn);

        if sojourn < EGRESS_TARGET_SOJOURN {
            self.above_since = None;
        } else {
            let since = *self.above_since.get_or_insert(now);
            if now.saturating_duration_since(since) >= EGRESS_CONGESTION_INTERVAL {
                self.pressure.mark_congested(now);
            }
        }

        sojourn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: Duration = Duration::from_secs(1);

    fn tracker() -> (Arc<EgressPressure>, SojournTracker) {
        let pressure = Arc::new(EgressPressure::new());
        (pressure.clone(), SojournTracker::new(pressure))
    }

    /// Feeds packets dequeued `spacing` apart starting at `start`, each having waited `sojourn`.
    fn dequeue_run(tracker: &mut SojournTracker, start: Instant, count: u32, spacing: Duration, sojourn: Duration) {
        for i in 0..count {
            let now = start + spacing * i;
            tracker.observe(now - sojourn, now);
        }
    }

    #[test]
    fn egress_should_not_start_out_congested() {
        let pressure = EgressPressure::new();
        assert!(!pressure.is_congested_within(Instant::now(), WINDOW));
    }

    #[test]
    fn a_congestion_mark_should_lapse_after_the_window() {
        let pressure = EgressPressure::new();
        let at = Instant::now();
        pressure.mark_congested(at);

        assert!(pressure.is_congested_within(at, WINDOW));
        assert!(pressure.is_congested_within(at + WINDOW, WINDOW));
        assert!(!pressure.is_congested_within(at + WINDOW + Duration::from_millis(1), WINDOW));
    }

    /// Two queues report independently; the earlier report arriving last must not win.
    #[test]
    fn a_late_older_mark_should_not_hide_a_newer_one() {
        let pressure = EgressPressure::new();
        let newer = Instant::now() + Duration::from_secs(10);
        pressure.mark_congested(newer);
        pressure.mark_congested(newer - Duration::from_secs(5));

        assert!(pressure.is_congested_within(newer + WINDOW, WINDOW));
    }

    /// The case this exists for: a queue fed faster than the wire drains, so every packet keeps
    /// waiting well past target for as long as the overload lasts.
    #[test]
    fn a_standing_queue_should_mark_congestion() {
        let (pressure, mut tracker) = tracker();
        let start = Instant::now();
        dequeue_run(
            &mut tracker,
            start,
            12,
            Duration::from_millis(10),
            Duration::from_millis(200),
        );

        assert!(pressure.is_congested_within(start + Duration::from_millis(110), WINDOW));
        assert_eq!(Duration::from_millis(200), pressure.last_sojourn());
    }

    /// A burst that briefly outruns the writer is what queues are for, not congestion.
    #[test]
    fn a_short_burst_of_delay_should_not_mark_congestion() {
        let (pressure, mut tracker) = tracker();
        let start = Instant::now();
        dequeue_run(
            &mut tracker,
            start,
            5,
            Duration::from_millis(10),
            Duration::from_millis(200),
        );

        assert!(!pressure.is_congested_within(start + Duration::from_millis(40), WINDOW));
    }

    /// One packet that got through promptly proves the queue drained, so the run starts over.
    #[test]
    fn a_prompt_packet_should_restart_the_run() {
        let (pressure, mut tracker) = tracker();
        let start = Instant::now();
        let spacing = Duration::from_millis(10);
        let late = Duration::from_millis(200);

        dequeue_run(&mut tracker, start, 8, spacing, late);
        let prompt = start + spacing * 8;
        tracker.observe(prompt, prompt);
        dequeue_run(&mut tracker, prompt + spacing, 8, spacing, late);

        assert!(
            !pressure.is_congested_within(prompt + spacing * 9, WINDOW),
            "neither run lasted a full interval on its own"
        );
    }

    #[test]
    fn a_delay_just_under_target_should_never_mark_congestion() {
        let (pressure, mut tracker) = tracker();
        let start = Instant::now();
        let just_under = EGRESS_TARGET_SOJOURN - Duration::from_millis(1);
        dequeue_run(&mut tracker, start, 100, Duration::from_millis(10), just_under);

        assert!(!pressure.is_congested_within(start + Duration::from_secs(1), WINDOW));
    }
}
