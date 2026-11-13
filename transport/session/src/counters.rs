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

/// Cumulative count of application data packets dropped because the session inbox
/// channel was closed (`try_send` returned `TrySendError::Disconnected`): the session
/// data sink was dropped while the slot was still registered — a benign teardown race,
/// not the backpressure that [`SESSION_INBOX_DROPS`] counts.
pub static SESSION_INBOX_CLOSED_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative count of packets dropped because the session inbox was closed.
#[inline]
pub fn session_inbox_closed_drop_count() -> usize {
    SESSION_INBOX_CLOSED_DROPS.load(Ordering::Relaxed)
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

/// Cumulative count of calls to `smgr.dispatch_message` in SessionsManagement(0).
/// Non-zero means packets are reaching the session manager dispatcher.
pub static DISPATCH_MESSAGE_CALLS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative dispatch_message call count.
#[inline]
pub fn dispatch_message_call_count() -> usize {
    DISPATCH_MESSAGE_CALLS.load(Ordering::Relaxed)
}

/// Cumulative count of Start-protocol messages dropped at dispatch because they name no state on
/// this node they could be handled by: neither a Session slot nor a pending initiation under the
/// pseudonym they arrive under — including one naming a registered Session under a mismatched
/// sender pseudonym.
///
/// These are messages nothing here could ever have handled — the cheapest possible way for a peer
/// to occupy the shared Start-protocol worker pool. Each drop emits only a `trace!`, which is
/// invisible at default log levels, so the counter is what makes the volume observable without the
/// volume becoming the log.
pub static SESSION_STALE_START_MESSAGES: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative count of Start-protocol messages rejected as naming no local state.
#[inline]
pub fn session_stale_start_message_count() -> usize {
    SESSION_STALE_START_MESSAGES.load(Ordering::Relaxed)
}

/// Cumulative count of Start-tagged packets that failed to decode.
///
/// Anyone can send these for free, so each drop logs only a `trace!` and the volume is read here.
pub static SESSION_MALFORMED_START_MESSAGES: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative malformed Start-protocol message count.
#[inline]
pub fn session_malformed_start_message_count() -> usize {
    SESSION_MALFORMED_START_MESSAGES.load(Ordering::Relaxed)
}

/// Cumulative count of Start-protocol messages that never reached a processing lane: the lane was
/// full, or the manager was shutting down (`try_send` failed).
///
/// Distinct from [`SESSION_STALE_START_MESSAGES`]: these named real local state and were lost to
/// backpressure or to shutdown rather than to the admission filter. Before the filter existed this
/// was the counter the DoS drove.
pub static START_PROTOCOL_SEND_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative Start-protocol queue-drop count.
#[inline]
pub fn start_protocol_send_drop_count() -> usize {
    START_PROTOCOL_SEND_DROPS.load(Ordering::Relaxed)
}

/// Cumulative count of control-lane Start-protocol messages lost to a full queue.
///
/// Split from [`START_PROTOCOL_SEND_DROPS`] because a lost `SsaCommit` is unrecoverable: a
/// `StartSession` flood advancing the shared counter could otherwise hide it entirely.
pub static START_CONTROL_SEND_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative control-lane queue-drop count.
#[inline]
pub fn start_control_send_drop_count() -> usize {
    START_CONTROL_SEND_DROPS.load(Ordering::Relaxed)
}

/// Cumulative count of packets dropped by `forward_to_timeout(app_incoming)` at the ingress
/// pipeline because `tx_from_protocol` was full for longer than `QUEUE_SEND_TIMEOUT` (50 ms).
pub static APP_INCOMING_TIMEOUT_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative app-incoming timeout drop count.
#[inline]
pub fn app_incoming_timeout_drop_count() -> usize {
    APP_INCOMING_TIMEOUT_DROPS.load(Ordering::Relaxed)
}

/// Cumulative count of refusal replies to `StartSession` that were not sent because the node's
/// refusal reply budget was spent.
///
/// Each one is a requester left to time out. It drives the rate-limited warning in the Start
/// protocol handler, and is what a flood looks like from outside.
pub static START_REFUSAL_REPLY_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative count of refusal replies dropped for want of budget.
#[inline]
pub fn start_refusal_reply_drop_count() -> usize {
    START_REFUSAL_REPLY_DROPS.load(Ordering::Relaxed)
}

/// Cumulative count of `StartSession` refusals decided by this node, whether or not the reply was
/// sent.
///
/// Counted per decision, so against [`START_REFUSAL_REPLY_DROPS`] it says how much of a flood the
/// refusal reply budget absorbed. It drives the rate-limited summary warning in the Start protocol
/// handler.
pub static START_REFUSALS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative count of refused `StartSession`s.
#[inline]
pub fn start_refusal_count() -> usize {
    START_REFUSALS.load(Ordering::Relaxed)
}
