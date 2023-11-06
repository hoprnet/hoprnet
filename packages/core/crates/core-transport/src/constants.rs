/// Application version as presented externally using the heartbeat mechanism
pub const APP_VERSION: &str = "2.1.0-rc.1";

/// Name of the metadata key holding the protocol version
pub const PEER_METADATA_PROTOCOL_VERSION: &str = "protocol_version";

pub const MAX_PARALLEL_PINGS: usize = 14;

pub const PACKET_SIZE: usize = 500;
pub const PACKET_QUEUE_TIMEOUT_MILLISECONDS: u64 = 15000;

pub(crate) const MAXIMUM_NETWORK_UPDATE_EVENT_QUEUE_SIZE: usize = 4000;

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::prelude::wasm_bindgen;

    use super::*;

    #[wasm_bindgen]
    pub fn peer_metadata_protocol_version_name() -> JsString {
        JsString::from(PEER_METADATA_PROTOCOL_VERSION)
    }
}
