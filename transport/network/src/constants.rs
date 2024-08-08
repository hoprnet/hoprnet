/// Interval to run heartbeat rounds, must include enough
/// time to traverse NATs
pub const DEFAULT_HEARTBEAT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);

/// Time after which the availability of a node gets rechecked
pub const DEFAULT_HEARTBEAT_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(60);

/// Randomization of the heartbeat interval to make sure not
/// all the nodes start their interval at the same time
pub const DEFAULT_HEARTBEAT_INTERVAL_VARIANCE: std::time::Duration = std::time::Duration::from_secs(2);
