use core_network::{PeerId, ping::PingExternalAPI, types::Result};
use utils_log::error;

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Debug, Clone)]
    pub struct WasmPingApi {
        on_finished_ping_cb: js_sys::Function,
    }

    impl PingExternalAPI for WasmPingApi {
        fn on_finished_ping(&self, peer: &PeerId, result: Result) {
            let this = JsValue::null();
            let peer = JsValue::from(peer.to_base58());
            let res = {
                if let Ok(v) = result {
                    JsValue::from(v as f64)
                } else {
                    JsValue::undefined()
                }
            };

            if let Err(err) = self.on_finished_ping_cb.call2(&this, &peer, &res) {
                error!(
                    "Failed to perform on peer offline operation with: {}",
                    err.as_string()
                        .unwrap_or_else(|| { "Unspecified error occurred on registering the ping result".to_owned() })
                        .as_str()
                )
            };
        }
    }

}