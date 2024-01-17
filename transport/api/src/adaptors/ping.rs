use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;

use core_network::{network::Network, ping::PingExternalAPI, types::Result, PeerId};
use core_path::channel_graph::ChannelGraph;
use hopr_internal_types::protocol::PeerAddressResolver;
use log::{debug, error};

use crate::{adaptors::network::ExternalNetworkInteractions, constants::PEER_METADATA_PROTOCOL_VERSION};

/// Implementor of the ping external API.
///
/// Ping requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary ping resources without leaking them into the
/// `Ping` object and keeping both the adaptor and the ping object OCP and SRP compliant.
#[derive(Debug, Clone)]
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

#[async_trait]
impl<R: PeerAddressResolver + std::marker::Sync> PingExternalAPI for PingExternalInteractions<R> {
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
            if let Ok(pk) = OffchainPublicKey::try_from(peer) {
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
