use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;

use core_network::{network::Network, ping::PingExternalAPI, ping::PingResult, PeerId};
use core_path::channel_graph::ChannelGraph;
use hopr_internal_types::protocol::PeerAddressResolver;
use tracing::{debug, error};

use crate::adaptors::network::ExternalNetworkInteractions;

/// Implementor of the ping external API.
///
/// Ping requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary ping resources without leaking them into the
/// `Ping` object and keeping both the adaptor and the ping object OCP and SRP
/// compliant.
#[derive(Debug, Clone)]
pub struct PingExternalInteractions<R: PeerAddressResolver + std::fmt::Debug> {
    network: Arc<Network<ExternalNetworkInteractions>>,
    resolver: R,
    channel_graph: Arc<RwLock<ChannelGraph>>,
}

impl<R: PeerAddressResolver + std::fmt::Debug> PingExternalInteractions<R> {
    pub fn new(
        network: Arc<Network<ExternalNetworkInteractions>>,
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
impl<R: PeerAddressResolver + std::marker::Sync + std::fmt::Debug> PingExternalAPI for PingExternalInteractions<R> {
    #[tracing::instrument(level = "debug")]
    async fn on_finished_ping(&self, peer: &PeerId, result: PingResult, version: String) {
        match self
            .network
            .update(peer, result, result.is_ok().then_some(version))
            .await
        {
            Ok(Some(updated)) => {
                debug!(
                    "peer {peer} has q = {}, avg_q = {}",
                    updated.get_quality(),
                    updated.get_average_quality()
                );
                if let Ok(pk) = OffchainPublicKey::try_from(peer) {
                    let maybe_chain_key = self.resolver.resolve_chain_key(&pk).await;
                    if let Some(chain_key) = maybe_chain_key {
                        let mut g = self.channel_graph.write().await;
                        let self_addr = g.my_address();
                        g.update_channel_quality(self_addr, chain_key, updated.get_quality());
                        debug!(
                            "update channel {self_addr} -> {chain_key} with quality {}",
                            updated.get_quality()
                        );
                    } else {
                        error!("could not resolve chain key for '{peer}'");
                    }
                } else {
                    error!("encountered invalid peer id: '{peer}'");
                }
            }
            Ok(None) => debug!("No update necessary"),
            Err(e) => error!("Encountered error on on updating the collected ping data: {e}"),
        }
    }
}
