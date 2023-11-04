use async_lock::RwLock;
use core_network::{
    network::{Health, Network, NetworkEvent, NetworkExternalActions, PeerStatus},
    PeerId,
};
use futures::channel::mpsc::Sender;
use std::sync::Arc;
use utils_log::{error, warn};

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;
#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

pub struct ExternalNetworkInteractions {
    emitter: Sender<NetworkEvent>,
}

impl ExternalNetworkInteractions {
    pub fn new(emitter: Sender<NetworkEvent>) -> Self {
        Self { emitter }
    }
}

impl NetworkExternalActions for ExternalNetworkInteractions {
    fn is_public(&self, _: &PeerId) -> bool {
        // NOTE: In the Providence release all nodes are public
        true
    }

    fn emit(&self, event: NetworkEvent) {
        if let Err(e) = self.emitter.clone().start_send(event.clone()) {
            error!("Failed to emit a network status: {}: {}", event, e)
        }
    }
    fn create_timestamp(&self) -> u64 {
        current_timestamp()
    }
}

/// Wrapper object necessary for async wasm function return value
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct WasmHealth {
    h: Health,
}

impl From<Health> for WasmHealth {
    fn from(value: Health) -> Self {
        Self { h: value }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use std::{pin::Pin, str::FromStr};

    use super::*;
    use core_network::network::{Health, PeerOrigin};
    use futures::future::poll_fn;
    use js_sys::JsString;
    use utils_misc::utils::wasm::js_map_to_hash_map;
    use utils_types::primitives::Address;
    use wasm_bindgen::prelude::*;

    /// Object needed only to simplify the iteration over the address and quality pair until
    /// the strategy is migrated into Rust
    #[wasm_bindgen]
    pub struct PeerQuality {
        peers_with_quality: Vec<(Address, f64)>,
    }

    impl PeerQuality {
        pub fn new(peers: Vec<(Address, f64)>) -> Self {
            Self {
                peers_with_quality: peers,
            }
        }

        pub fn take(&self) -> Vec<(Address, f64)> {
            self.peers_with_quality.clone()
        }
    }

    /// Wrapper object for the `Network` functionality to be callable from outside
    /// the WASM environment.
    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct WasmNetwork {
        network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
        change_notifier: Sender<NetworkEvent>,
    }

    impl WasmNetwork {
        pub(crate) fn new(
            network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
            change_notifier: Sender<NetworkEvent>,
        ) -> Self {
            Self {
                network,
                change_notifier,
            }
        }

        pub fn as_counted_ref(&self) -> Arc<RwLock<Network<ExternalNetworkInteractions>>> {
            self.network.clone()
        }
    }

    #[wasm_bindgen]
    impl WasmHealth {
        #[wasm_bindgen]
        pub fn unwrap(&self) -> Health {
            self.h
        }
    }

    #[wasm_bindgen]
    impl WasmNetwork {
        /// Get all registered multiaddresses for a specific peer
        pub async fn get_peer_multiaddresses(&self, peer: JsString) -> js_sys::Array {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => js_sys::Array::from_iter(
                    self.network
                        .read()
                        .await
                        .get_peer_multiaddresses(&p)
                        .into_iter()
                        .map(|ma| JsString::from(ma.to_string())),
                ),
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, network assumes it is not present: {}",
                        peer,
                        err.to_string()
                    );
                    js_sys::Array::new()
                }
            }
        }

        #[wasm_bindgen]
        pub async fn contains(&self, peer: JsString) -> bool {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => {
                    let reader = self.network.read().await;
                    (*reader).has(&p)
                }
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
        pub async fn quality_of(&self, peer: JsString) -> f64 {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => match (*self.network.read().await).get_peer_status(&p) {
                    Some(v) => v.get_quality(),
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
        pub async fn health(&self) -> WasmHealth {
            (*self.network.read().await).health().into()
        }

        #[wasm_bindgen]
        pub async fn all(&self) -> js_sys::Array {
            js_sys::Array::from_iter(
                (*self.network.read().await)
                    .filter(|_| true)
                    .iter()
                    .map(|x| JsValue::from(x.to_base58())),
            )
        }

        #[wasm_bindgen]
        pub async fn get_peer_info(&self, peer: JsString) -> Option<PeerStatus> {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => (*self.network.read().await).get_peer_status(&p),
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

        #[wasm_bindgen]
        pub async fn register(&self, peer: JsString, origin: PeerOrigin) {
            self.register_with_metadata(peer, origin, &js_sys::Map::from(JsValue::undefined()))
                .await
        }

        #[wasm_bindgen]
        pub async fn register_with_metadata(&self, peer: JsString, origin: PeerOrigin, metadata: &js_sys::Map) {
            let mut change_notifier = self.change_notifier.clone();

            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => match poll_fn(|cx| Pin::new(&mut change_notifier).poll_ready(cx)).await {
                    Ok(_) => match change_notifier.start_send(NetworkEvent::Register(
                        p,
                        origin,
                        js_map_to_hash_map(metadata),
                    )) {
                        Ok(_) => {}
                        Err(e) => error!("Failed to sent network update 'register' to the receiver: {}", e),
                    },
                    Err(e) => error!("The receiver for network updates was dropped: {}", e),
                },
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
        pub async fn unregister(&self, peer: JsString) {
            let mut change_notifier = self.change_notifier.clone();

            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => match poll_fn(|cx| Pin::new(&mut change_notifier).poll_ready(cx)).await {
                    Ok(_) => match change_notifier.start_send(NetworkEvent::Unregister(p)) {
                        Ok(_) => {}
                        Err(e) => error!("Failed to sent network update 'unregister' to the receiver: {}", e),
                    },
                    Err(e) => error!("The receiver for network updates was dropped: {}", e),
                },
                Err(err) => {
                    warn!(
                        "Failed to parse peer id {}, network ignores the unregister attempt: {}",
                        peer,
                        err.to_string()
                    );
                }
            }
        }
    }
}
