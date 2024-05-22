use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;
use tracing::{debug, error};

use core_network::{
    network::{Network, NetworkTriggeredEvent},
    ping::PingExternalAPI,
    ping::PingResult,
    PeerId,
};
use core_path::channel_graph::ChannelGraph;
use hopr_crypto_types::types::OffchainPublicKey;

/// Implementor of the ping external API.
///
/// Ping requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary ping resources without leaking them into the
/// `Ping` object and keeping both the adaptor and the ping object OCP and SRP
/// compliant.
#[derive(Debug, Clone)]
pub struct PingExternalInteractions<T>
where
    T: hopr_db_api::peers::HoprDbPeersOperations
        + hopr_db_api::resolver::HoprDbResolverOperations
        + Sync
        + Send
        + Clone
        + std::fmt::Debug,
{
    network: Arc<Network<T>>,
    resolver: T,
    channel_graph: Arc<RwLock<ChannelGraph>>,
    /// Implementation of the network interface allowing emitting events
    /// based on the [core_network::network::Network] events into the p2p swarm.
    emitter: futures::channel::mpsc::Sender<NetworkTriggeredEvent>,
}

impl<T> PingExternalInteractions<T>
where
    T: hopr_db_api::peers::HoprDbPeersOperations
        + hopr_db_api::resolver::HoprDbResolverOperations
        + Sync
        + Send
        + Clone
        + std::fmt::Debug,
{
    pub fn new(
        network: Arc<Network<T>>,
        resolver: T,
        channel_graph: Arc<RwLock<ChannelGraph>>,
        emitter: futures::channel::mpsc::Sender<NetworkTriggeredEvent>,
    ) -> Self {
        Self {
            network,
            resolver,
            channel_graph,
            emitter,
        }
    }
}

#[async_trait]
impl<T> PingExternalAPI for PingExternalInteractions<T>
where
    T: hopr_db_api::peers::HoprDbPeersOperations
        + hopr_db_api::resolver::HoprDbResolverOperations
        + Sync
        + Send
        + Clone
        + std::fmt::Debug,
{
    #[tracing::instrument(level = "info", skip(self))]
    async fn on_finished_ping(&self, peer: &PeerId, result: PingResult, version: String) {
        match self
            .network
            .update(peer, result, result.is_ok().then_some(version))
            .await
        {
            Ok(Some(updated)) => match updated {
                NetworkTriggeredEvent::CloseConnection(peer) => {
                    if let Err(e) = self
                        .emitter
                        .clone()
                        .try_send(NetworkTriggeredEvent::CloseConnection(peer))
                    {
                        error!("Failed to emit a network event 'close connection': {}", e)
                    }
                }
                NetworkTriggeredEvent::UpdateQuality(peer, quality) => {
                    debug!("'{peer}' changed quality to '{quality}'");
                    if let Ok(pk) = OffchainPublicKey::try_from(peer) {
                        let maybe_chain_key = self.resolver.resolve_chain_key(&pk).await;
                        if let Ok(Some(chain_key)) = maybe_chain_key {
                            let mut g = self.channel_graph.write().await;
                            let self_addr = g.my_address();
                            g.update_channel_quality(self_addr, chain_key, quality);
                            debug!("update channel {self_addr} -> {chain_key} with quality {quality}");
                        } else {
                            error!("could not resolve chain key for '{peer}'");
                        }
                    } else {
                        error!("encountered invalid peer id: '{peer}'");
                    }
                }
            },
            Ok(None) => debug!("No update necessary"),
            Err(e) => error!("Encountered error on on updating the collected ping data: {e}"),
        }
    }
}
