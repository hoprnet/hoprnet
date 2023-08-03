use std::sync::Arc;

use async_lock::RwLock;

use core_network::network::Network;

#[cfg(feature = "wasm")]
pub(crate) mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WasmNetwork {
        network: Arc<RwLock<Network>>,
    }

    #[wasm_bindgen]
    impl WasmNetwork {
        pub(crate) fn new(network: Network) -> Self {
            Self { network: Arc::new(RwLock::new(network)) }
        }
    }

    impl WasmNetwork {
        #[wasm_bindgen]
        pub fn peers_to_ping(&self, threshold: u64) -> Vec<JsString> {
            self.networkfind_peers_to_ping(threshold)
                .iter()
                .map(|x| x.to_base58().into())
                .collect::<Vec<JsString>>()
        }

        #[wasm_bindgen]
        pub fn contains(&self, peer: JsString) -> bool {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => self.has(&p),
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, network assumes it is not present: {}",
                        peer,
                        err.to_string()
                    );
                    false
                }
            }
        }

        #[wasm_bindgen]
        pub fn register(&mut self, peer: JsString, origin: PeerOrigin) {
            self.register_with_metadata(peer, origin, &js_sys::Map::from(JsValue::undefined()))
        }

        #[wasm_bindgen]
        pub fn register_with_metadata(&mut self, peer: JsString, origin: PeerOrigin, metadata: &js_sys::Map) {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => self.add_with_metadata(&p, origin, js_map_to_hash_map(metadata)),
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, network ignores the register attempt: {}",
                        peer,
                        err.to_string()
                    );
                }
            }
        }

        #[wasm_bindgen]
        pub fn unregister(&mut self, peer: JsString) {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => self.remove(&p),
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, network ignores the unregister attempt: {}",
                        peer,
                        err.to_string()
                    );
                }
            }
        }

        #[wasm_bindgen]
        pub fn refresh(&mut self, peer: JsString, timestamp: JsValue) {
            self.refresh_with_metadata(peer, timestamp, &js_sys::Map::from(JsValue::undefined()))
        }

        #[wasm_bindgen]
        pub fn refresh_with_metadata(&mut self, peer: JsString, timestamp: JsValue, metadata: &js_sys::Map) {
            let peer: String = peer.into();
            let result: crate::types::Result = if timestamp.is_undefined() {
                Err(())
            } else {
                timestamp.as_f64().map(|v| v as u64).ok_or(())
            };
            match PeerId::from_str(&peer) {
                Ok(p) => self.update_with_metadata(&p, result, js_map_to_hash_map(metadata)),
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, network ignores the regresh attempt: {}",
                        peer,
                        err.to_string()
                    );
                }
            }
        }

        #[wasm_bindgen]
        pub fn quality_of(&self, peer: JsString) -> f64 {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => match self.get_peer_status(&p) {
                    Some(v) => v.quality,
                    _ => 0.0f64,
                },
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, using lowest possible quality: {}",
                        peer,
                        err.to_string()
                    );
                    0.0f64
                }
            }
        }

        #[wasm_bindgen]
        pub fn all(&self) -> Vec<JsString> {
            self.filter(|_| true)
                .iter()
                .map(|x| x.to_base58().into())
                .collect::<Vec<JsString>>()
        }

        #[wasm_bindgen]
        pub fn get_peer_info(&self, peer: JsString) -> Option<PeerStatus> {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => self.get_peer_status(&p),
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, peer info unavailable: {}",
                        peer,
                        err.to_string()
                    );
                    None
                }
            }
        }
    }
}