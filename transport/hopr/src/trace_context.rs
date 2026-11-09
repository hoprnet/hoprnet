//! Local packet identifiers and compatibility exports for queue tracing context.

use std::sync::atomic::{AtomicU64, Ordering};

pub use hopr_utils_tracing::{ContextSender, ContextSink, WithContext};

pub(crate) fn next_packet_id() -> u64 {
    static NEXT_PACKET_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_PACKET_ID.fetch_add(1, Ordering::Relaxed)
}
