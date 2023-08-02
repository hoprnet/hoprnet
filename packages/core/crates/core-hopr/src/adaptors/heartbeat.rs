use core_network::{PeerId, heartbeat::HeartbeatExternalApi};
use utils_log::error;


#[cfg(feature = "wasm")]
pub(crate) mod wasm {
    use super::*;
    use std::str::FromStr;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WasmHeartbeatApi {
        get_peers: js_sys::Function,
    }

    #[wasm_bindgen]
    impl WasmHeartbeatApi {
        pub(crate) fn new(get_peers: js_sys::Function) -> Self {
            Self { get_peers }
        }
    }

    impl HeartbeatExternalApi for WasmHeartbeatApi {
        fn get_peers(&self, from_timestamp: u64) -> Vec<PeerId> {
            let this = JsValue::null();
            let timestamp = JsValue::from(from_timestamp);

            return match self.get_peers.call1(&this, &timestamp) {
                Ok(v) => {
                    js_sys::Array::from(&v)
                        .to_vec()
                        .into_iter()
                        .filter_map(|v| PeerId::from_str(String::from(js_sys::JsString::from(v)).as_str()).ok())
                        .collect()
                }
                Err(err) => {
                    error!(
                        "Failed to perform on peer offline operation with: {}",
                        err.as_string()
                            .unwrap_or_else(|| { "Unknown error occurred on fetching the peers to ping".to_owned() })
                            .as_str()
                    );
                    Vec::new()
                }
            };
        }
    }
}