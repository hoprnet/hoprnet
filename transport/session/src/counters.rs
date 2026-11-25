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

/// Cumulative count of `StartSession` repeats this node sent as an Entry.
pub static START_SESSION_RESENDS: AtomicUsize = AtomicUsize::new(0);

/// Cumulative count of repeated `StartSession`s this node received as an Exit, answered or dropped.
pub static START_SESSION_REPEATS_RECEIVED: AtomicUsize = AtomicUsize::new(0);

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

/// Cumulative count of `StartSession` refusal replies not sent because the refusal reply budget was
/// spent; each is a requester left to time out.
pub static START_REFUSAL_REPLY_DROPS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative count of refusal replies dropped for want of budget.
#[inline]
pub fn start_refusal_reply_drop_count() -> usize {
    START_REFUSAL_REPLY_DROPS.load(Ordering::Relaxed)
}

/// Cumulative count of `StartSession` refusals decided, whether or not the reply was sent.
///
/// Against [`START_REFUSAL_REPLY_DROPS`] it shows how much of a flood the budget absorbed.
pub static START_REFUSALS: AtomicUsize = AtomicUsize::new(0);

/// Returns the cumulative count of refused `StartSession`s.
#[inline]
pub fn start_refusal_count() -> usize {
    START_REFUSALS.load(Ordering::Relaxed)
}

/// Cumulative count of forward receipts sent by Exits.
pub static FORWARD_RECEIPTS_SENT: AtomicUsize = AtomicUsize::new(0);

/// Cumulative count of forward receipts received by Entries.
pub static FORWARD_RECEIPTS_RECEIVED: AtomicUsize = AtomicUsize::new(0);
