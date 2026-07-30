//! Process-global packet-flow diagnostic counters for the throughput stress harness.
//!
//! These counters were originally defined in `hopr-utils::parallelize`, but they encode
//! HOPR session/routing domain concepts (session inbox, routing resolution, SPHINX encode
//! stage) rather than generic parallelisation primitives. They therefore live here in the
//! transport-session crate instead of the generic `hopr-utilities` crate.
//!
//! Each counter is a plain process-global atomic, incremented on hot paths across the
//! transport/session pipeline and read (as deltas) by the load generator. They are not
//! gated on any feature so they are available in every build, including test builds.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Cumulative count of application data packets dropped because the session inbox
/// channel was full (`try_send` returned `TrySendError::Full`).
pub static SESSION_INBOX_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative session inbox drop count.
#[inline]
pub fn session_inbox_drop_count() -> usize {
    SESSION_INBOX_DROPS.load(Ordering::Relaxed)
}

/// Cumulative count of data packets dropped because no matching session slot was
/// found in the session manager (`UnknownData` / unestablished-session path).
pub static SESSION_UNKNOWN_DATA_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative UnknownData drop count.
#[inline]
pub fn session_unknown_data_drop_count() -> usize {
    SESSION_UNKNOWN_DATA_DROPS.load(Ordering::Relaxed)
}

/// Cumulative count of data packets dispatched as "unrelated" — reached dispatch_message
/// but matched neither the session protocol tag nor any session application tag.
pub static SESSION_UNRELATED_DATA_DISPATCHES: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative unrelated dispatch count.
#[inline]
pub fn session_unrelated_dispatch_count() -> usize {
    SESSION_UNRELATED_DATA_DISPATCHES.load(Ordering::Relaxed)
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

/// Cumulative count of calls to `smgr.dispatch_message` in SessionsManagement(0).
/// Non-zero means packets are reaching the session manager dispatcher.
pub static DISPATCH_MESSAGE_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative dispatch_message call count.
#[inline]
pub fn dispatch_message_call_count() -> usize {
    DISPATCH_MESSAGE_CALLS.load(Ordering::Relaxed)
}

/// Cumulative count of packets dropped by `forward_to_timeout(app_incoming)` at the ingress
/// pipeline because `tx_from_protocol` was full for longer than `QUEUE_SEND_TIMEOUT` (50 ms).
pub static APP_INCOMING_TIMEOUT_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative app-incoming timeout drop count.
#[inline]
pub fn app_incoming_timeout_drop_count() -> usize {
    APP_INCOMING_TIMEOUT_DROPS.load(Ordering::Relaxed)
}
