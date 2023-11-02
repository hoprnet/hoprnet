/// Application version as presented externally using the heartbeat mechanism
pub const APP_VERSION: &str = "2.1.0-rc.1";

/// Name of the metadata key holding the protocol version
pub const PEER_METADATA_PROTOCOL_VERSION: &str = "protocol_version";

#[cfg(feature = "wasm")]
pub mod wasm {
    use js_sys::JsString;
    use wasm_bindgen::prelude::wasm_bindgen;

    use super::*;

    #[wasm_bindgen]
    pub fn app_version() -> JsString {
        JsString::from(APP_VERSION)
    }

    #[wasm_bindgen]
    pub fn peer_metadata_protocol_version_name() -> JsString {
        JsString::from(PEER_METADATA_PROTOCOL_VERSION)
    }
}
