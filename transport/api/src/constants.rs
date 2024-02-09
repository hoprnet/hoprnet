/// Application version as presented externally by the node
pub const APP_VERSION: &str = "2.1.0-rc.1";

/// Name of the metadata key holding the protocol version
pub const PEER_METADATA_PROTOCOL_VERSION: &str = "protocol_version";

/// The maximum waiting time for a message send to produce a half key challenge reply
pub const PACKET_QUEUE_TIMEOUT_MILLISECONDS: std::time::Duration = std::time::Duration::from_millis(15000);

/// The maximum queue size for the network update events
pub(crate) const MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE: usize = 4000;
