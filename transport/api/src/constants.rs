/// The maximum waiting time for a message send to produce a half key challenge reply
pub const PACKET_QUEUE_TIMEOUT_MILLISECONDS: std::time::Duration = std::time::Duration::from_millis(15000);

/// The maximum queue size for the network update events
pub(crate) const MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE: usize = 4000;

/// The upper limit value for the session reserved tag range.
///
/// The reserved tags are from range <[`RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT`], [`RESERVED_SESSION_TAG_UPPER_LIMIT`]) and are
/// specifically dedicated for the internal use of the protocol.
pub const RESERVED_SESSION_TAG_UPPER_LIMIT: u16 = 1024;

/// The upper limit value for subprotocol reserved tag range.
///
/// The reserved tags are from range <0,[`RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT`]) and are
/// specifically dedicated for the internal use by the subprotocols.
pub const RESERVED_SUBPROTOCOL_TAG_UPPER_LIMIT: u16 = 16;
