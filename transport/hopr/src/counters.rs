//! Process-global routing-resolution and encode-stage diagnostic counters.
//!
//! Plain process-global atomics, incremented on the packet pipeline's hot paths and read (as deltas)
//! by the throughput stress harness. They live in the transport, not in the Session crate, because a
//! relay routes and encodes packets without having any Session.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Cumulative count of outgoing packets dropped because their return route still had no SURB
/// when the resolution wait ran out.
///
/// Separate from [`ROUTING_RESOLUTION_FAILURES`] because it means something different and
/// actionable: a counterparty has stopped replenishing SURBs and is not coming back, rather than a
/// route that could not be computed. A non-zero and growing value names the condition that used to
/// stall a node's entire egress indefinitely.
pub static ROUTING_RESOLUTION_SURB_TIMEOUTS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative count of packets dropped after waiting in vain for a return SURB.
#[inline]
pub fn routing_resolution_surb_timeout_count() -> usize {
    ROUTING_RESOLUTION_SURB_TIMEOUTS.load(Ordering::Relaxed)
}

/// Cumulative count of packets that failed path/routing resolution before encoding.
pub static ROUTING_RESOLUTION_FAILURES: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative routing resolution failure count.
#[inline]
pub fn routing_resolution_failure_count() -> usize {
    ROUTING_RESOLUTION_FAILURES.load(Ordering::Relaxed)
}

/// Cumulative count of packets that successfully entered the routing resolution stage.
pub static ROUTING_RESOLUTION_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative routing resolution attempt count.
#[inline]
pub fn routing_resolution_attempt_count() -> usize {
    ROUTING_RESOLUTION_ATTEMPTS.load(Ordering::Relaxed)
}

/// Cumulative count of packets that entered the SPHINX encode stage (spawn_encode_blocking called).
pub static ENCODE_STAGE_ENTRIES: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative encode stage entry count.
#[inline]
pub fn encode_stage_entry_count() -> usize {
    ENCODE_STAGE_ENTRIES.load(Ordering::Relaxed)
}
