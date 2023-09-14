
/// Interval to run heartbeat rounds, must include enough
/// time to traverse NATs
pub const DEFAULT_HEARTBEAT_INTERVAL: u64 = 60000;
/// Time after which the availability of a node gets rechecked
pub const DEFAULT_HEARTBEAT_THRESHOLD: u64 = 60000;
/// Randomization of the heartbeat interval to make sure not
/// all of the nodes start their interval at the same time
pub const DEFAULT_HEARTBEAT_INTERVAL_VARIANCE: u64 = 2000;



/// Network quality threshold from which a node is considered
/// available enough to be used
pub const DEFAULT_NETWORK_QUALITY_THRESHOLD: f64 = 0.5;


