use std::time::Duration;

use libp2p::PeerId;

/// Minimum delay will be multiplied by backoff, it will be half the actual minimum value
const MIN_DELAY: Duration = Duration::from_secs(1);
const MAX_DELAY: Duration = Duration::from_secs(300);   // 5 minutes
const BACKOFF_EXPONENT: f32 = 1.5;
const MAX_BACKOFF: f32 = MAX_DELAY.as_millis() as f32 / MIN_DELAY.as_millis() as f32;
/// Default quality for unknown or offline nodes
const BAD_QUALITY: f32 = 0.2;
const IGNORE_TIMEFRAME: Duration = Duration::from_secs(600);   // 10 minutes


// Does not work with enums
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)
#[derive(Debug)]
enum NetworkPeerOrigin {
    Initialization = 0,
    NetworkRegistry = 1,
    IncomingConnection = 2,
    OutgoingConnection = 3,
    StrategyExistingChannel = 4,
    StrategyConsideringChannel = 5,
    StrategyNewChannel = 6,
    ManualPing = 7,
    Testing = 8
}


#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug)]
pub struct Entry {
    id: PeerId,
    heartbeats_sent: u64,
    heartbeats_succeeded: u64,
    last_seen: u64,
    backoff: f32,
    quality: f32,
    origin: NetworkPeerOrigin,
    ignored_at: Option<f32>
}


#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_foo() {
        assert_eq!(1, 2-1);
    }
}




