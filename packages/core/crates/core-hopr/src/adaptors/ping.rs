use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;

use core_network::{
    network::Network,
    ping::{Ping, PingExternalAPI},
    types::Result,
    PeerId,
};

use crate::{adaptors::network::ExternalNetworkInteractions, constants::PEER_METADATA_PROTOCOL_VERSION};

/// Implementor of the ping external API.
///
/// Ping requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary ping resources without leaking them into the
/// `Ping` object and keeping both the adaptor and the ping object OCP and SRP compliant.
#[derive(Clone)]
pub struct PingExternalInteractions {
    network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
}

impl PingExternalInteractions {
    pub fn new(network: Arc<RwLock<Network<ExternalNetworkInteractions>>>) -> Self {
        Self { network }
    }
}

#[async_trait]
impl PingExternalAPI for PingExternalInteractions {
    async fn on_finished_ping(&self, peer: &PeerId, result: Result, version: String) {
        // This logic deserves a larger refactor of the entire heartbeat mechanism, but
        // for now it is suffcient to fill out metadata only on successful pongs.
        let metadata = if result.is_ok() {
            let mut map = std::collections::HashMap::new();
            map.insert(PEER_METADATA_PROTOCOL_VERSION.to_owned(), version);
            Some(map)
        } else {
            None
        };

        self.network.write().await.update_with_metadata(peer, result, metadata)
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use core_network::ping::Pinging;
    use futures::{
        future::{select, Either},
        pin_mut, FutureExt,
    };
    use gloo_timers::future::sleep;
    use std::str::FromStr;
    use utils_log::info;
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
        pub async fn ping(&self, peer: js_sys::JsString) {
            let x: String = peer.into();
            if let Some(converted) = core_network::PeerId::from_str(&x).ok() {
                let mut pinger = self.ping.write().await;

                let timeout = sleep(std::time::Duration::from_millis(30_000)).fuse();
                let ping = pinger.ping(vec![converted]).fuse();

                pin_mut!(timeout, ping);

                match select(timeout, ping).await {
                    Either::Left(_) => info!("Manual ping to peer '{}' timed out", converted),
                    Either::Right(_) => info!("Manual ping succeeded"),
                };
            }
        }
    }
}
