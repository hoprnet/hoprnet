use std::time::Duration;

/// The maximum waiting time for a message send to produce a half-key challenge reply
pub const PACKET_QUEUE_TIMEOUT_MILLISECONDS: std::time::Duration = std::time::Duration::from_millis(15000);

/// The maximum queue size for the network update events
pub(crate) const MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE: usize = 4000;

pub(crate) const MAXIMUM_MSG_OUTGOING_BUFFER_SIZE: usize = 20000;

/// Time within Start protocol must finish session initiation.
/// This base value is always multiplied by the (max) number of hops, times 2 (for both-ways).
pub(crate) const SESSION_INITIATION_TIMEOUT_BASE: Duration = Duration::from_secs(5);
