use std::sync::Arc;

use async_lock::RwLock;
use futures::channel::mpsc::Sender;

use core_network::{
    PeerId,
    network::{Network, NetworkExternalActions, PeerStatus, NetworkEvent}
};
use utils_log::{warn,error};

pub struct ExternalNetworkInteractions {
    emitter: Sender<NetworkEvent>
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

    fn emit(&mut self, event: NetworkEvent) {
        match self.emitter.start_send(event.clone()) {
            Ok(_) => {},
            Err(_) => error!("Failed to emit a network status: {}", event)
        }
    }
}


#[cfg(feature = "wasm")]
pub(crate) mod wasm {
    use std::{str::FromStr, pin::Pin};

    use super::*;
    use core_network::network::PeerOrigin;
    use futures::future::poll_fn;
    use js_sys::JsString;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct WasmNetwork {
        network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
        change_notifier: Sender<NetworkEvent>
    }

    impl WasmNetwork {
        pub(crate) fn new(network: Arc<RwLock<Network<ExternalNetworkInteractions>>>, change_notifier: Sender<NetworkEvent>) -> Self {
            Self { network, change_notifier }
        }
    }

    #[wasm_bindgen]
    impl WasmNetwork {
        #[wasm_bindgen]
        pub async fn contains(&self, peer: JsString) -> bool {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => {
                    let reader = self.network.read().await;
                    (*reader).has(&p)
                },
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
        pub async fn all(&self) -> js_sys::Array {
            js_sys::Array::from_iter((*self.network.read().await)
                .filter(|_| true)
                .iter()
                .map(|x| JsValue::from(x.to_base58())))
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
        pub async fn register(&mut self, peer: JsString, origin: PeerOrigin) {
            self.register_with_metadata(peer, origin, &js_sys::Map::from(JsValue::undefined())).await
        }

        #[wasm_bindgen]
        pub async fn register_with_metadata(&mut self, peer: JsString, origin: PeerOrigin, metadata: &js_sys::Map) {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => {
                    // TODO: ignoring metadata for now
                    // self.add_with_metadata(&p, origin, js_map_to_hash_map(metadata))
                    match poll_fn(|cx| Pin::new(&mut self.change_notifier).poll_ready(cx)).await {
                        Ok(_) => {
                            match self.change_notifier.start_send(NetworkEvent::Register(p, origin)) {
                                Ok(_) => {},
                                Err(e) => error!("Failed to sent network update 'register' to the receiver: {}", e),
                            }
                        }
                        Err(e) => error!("The receiver for network updates was dropped: {}", e)
                    }
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
        pub async fn unregister(&mut self, peer: JsString) {
            let peer: String = peer.into();
            match PeerId::from_str(&peer) {
                Ok(p) => {
                    match poll_fn(|cx| Pin::new(&mut self.change_notifier).poll_ready(cx)).await {
                        Ok(_) => {
                            match self.change_notifier.start_send(NetworkEvent::Unregister(p)) {
                                Ok(_) => {},
                                Err(e) => error!("Failed to sent network update 'unregister' to the receiver: {}", e),
                            }
                        }
                        Err(e) => error!("The receiver for network updates was dropped: {}", e)
                    }
                }
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