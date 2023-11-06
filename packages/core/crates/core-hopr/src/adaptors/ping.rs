use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;
use core_crypto::types::OffchainPublicKey;

use core_network::{
    network::Network,
    ping::{Ping, PingExternalAPI},
    types::Result,
    PeerId,
};
use core_path::channel_graph::ChannelGraph;
use core_types::protocol::PeerAddressResolver;
use utils_log::{debug, error};
use utils_types::traits::PeerIdLike;

use crate::{adaptors::network::ExternalNetworkInteractions, constants::PEER_METADATA_PROTOCOL_VERSION};

/// Implementor of the ping external API.
///
/// Ping requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary ping resources without leaking them into the
/// `Ping` object and keeping both the adaptor and the ping object OCP and SRP compliant.
#[derive(Clone)]
pub struct PingExternalInteractions<R: PeerAddressResolver> {
    network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
    resolver: R,
    channel_graph: Arc<RwLock<ChannelGraph>>,
}

impl<R: PeerAddressResolver> PingExternalInteractions<R> {
    pub fn new(
        network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
        resolver: R,
        channel_graph: Arc<RwLock<ChannelGraph>>,
    ) -> Self {
        Self {
            network,
            resolver,
            channel_graph,
        }
    }
}

#[async_trait(? Send)]
impl<R: PeerAddressResolver> PingExternalAPI for PingExternalInteractions<R> {
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

        let updated = self.network.write().await.update_with_metadata(peer, result, metadata);

        if let Some(status) = updated {
            debug!(
                "peer {peer} has q = {}, avg_q = {}",
                status.get_quality(),
                status.get_average_quality()
            );
            if let Ok(pk) = OffchainPublicKey::from_peerid(&peer) {
                let maybe_chain_key = self.resolver.resolve_chain_key(&pk).await;
                if let Some(chain_key) = maybe_chain_key {
                    let mut g = self.channel_graph.write().await;
                    let self_addr = g.my_address();
                    g.update_channel_quality(self_addr, chain_key, status.get_quality())
                } else {
                    error!("could not resolve chain key for peer {peer}");
                }
            } else {
                error!("encountered invalid peer id: {peer}");
            }
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use super::*;
    use core_network::ping::Pinging;
    use core_path::DbPeerAddressResolver;
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
        ping: Arc<RwLock<Ping<PingExternalInteractions<DbPeerAddressResolver>>>>,
    }

    impl WasmPing {
        pub(crate) fn new(ping: Arc<RwLock<Ping<PingExternalInteractions<DbPeerAddressResolver>>>>) -> Self {
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
