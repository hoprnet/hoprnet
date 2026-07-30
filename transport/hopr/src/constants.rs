use std::time::Duration;

/// The maximum waiting time for a message send to produce a half-key challenge reply
pub const PACKET_QUEUE_TIMEOUT_MILLISECONDS: std::time::Duration = std::time::Duration::from_millis(15000);

/// Maximum number of outgoing application-layer packets buffered before the writer
/// observes backpressure (`Poll::Pending` on `poll_write`).
///
/// Caps the burst submitted to `then_concurrent(8×N_cpus)` so tail packets never
/// exceed `PACKET_ENCODING_TIMEOUT` (150 ms). With Sphinx encoding at ~21 ms/packet
/// and 256 slots, the worst-case Rayon queue depth stays well under the timeout on
/// machines with up to ~36 cores. Safe formula: `≤ 7 × available_parallelism()`.
pub(crate) const MAXIMUM_MSG_OUTGOING_BUFFER_SIZE: usize = 256;

/// Time within Start protocol must finish session initiation.
/// This base value is always multiplied by the (max) number of hops, times 2 (for both-ways).
pub(crate) const SESSION_INITIATION_TIMEOUT_BASE: Duration = Duration::from_secs(5);
