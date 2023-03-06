use std::time::Duration;

use libp2p::PeerId;

const MAX_PARALLEL_HEARTBEATS: u16 = 14;
const HEARTBEAT_ROUND_TIMEOUT: Duration = Duration::from_secs(60);

mod metrics {

}

#[derive(Debug)]
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct HeartbeatPingResult {
    pub destination: PeerId,
    pub last_seen: Option<u64>
}
