/// Interval to run heartbeat rounds, must include enough
/// time to traverse NATs
pub const DEFAULT_HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_millis(60_000);
/// Time after which the availability of a node gets rechecked
pub const DEFAULT_HEARTBEAT_THRESHOLD: std::time::Duration = std::time::Duration::from_millis(60_000);
/// Randomization of the heartbeat interval to make sure not
/// all of the nodes start their interval at the same time
pub const DEFAULT_HEARTBEAT_INTERVAL_VARIANCE: std::time::Duration = std::time::Duration::from_millis(2_000);

/// Network quality threshold from which a node is considered
/// available enough to be used
pub const DEFAULT_NETWORK_QUALITY_THRESHOLD: f64 = 0.5;
