use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;

use core_network::{
    PeerId,
    network::Network,
    ping::{Ping, PingExternalAPI},
    types::Result
};

use crate::adaptors::network::ExternalNetworkInteractions;


#[derive(Clone)]
pub struct PingExternalInteractions {
    network: Arc<RwLock<Network<ExternalNetworkInteractions>>>
}

impl PingExternalInteractions {
    pub fn new(network: Arc<RwLock<Network<ExternalNetworkInteractions>>>) -> Self {
        Self { network }
    }
}

#[async_trait]
impl PingExternalAPI for PingExternalInteractions {
    async fn on_finished_ping(&self, peer: &PeerId, result: Result) {
        let mut writer = self.network.write().await;
        (*writer).update_with_metadata(peer, result, None)
    }
}


#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use std::str::FromStr;
    use core_network::ping::Pinging;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct WasmPing {
        ping: Arc<RwLock<Ping<PingExternalInteractions>>>,
    }

    impl WasmPing {
        pub(crate) fn new(ping: Arc<RwLock<Ping<PingExternalInteractions>>>) -> Self {
            Self { ping }
        }
    }

    #[wasm_bindgen] 
    impl WasmPing {
        /// Ping the peers represented as a Vec<JsString> values that are converted into usable
        /// PeerIds.
        ///
        /// # Arguments
        /// * `peers` - Vector of String representations of the PeerIds to be pinged.
        #[wasm_bindgen]
        pub async fn ping(&mut self, peer: js_sys::JsString) {
            let x: String = peer.into();
            if let Some(converted) = core_network::PeerId::from_str(&x).ok() {
                (*self.ping.write().await).ping(vec![converted]).await;
            }
        }
    }
}