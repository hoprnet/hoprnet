use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_sql::api::resolver::HoprDbResolverOperations;
use hopr_path::channel_graph::ChannelGraph;
use hopr_transport_network::{
    network::{Network, NetworkTriggeredEvent},
    ping::PingExternalAPI,
    HoprDbPeersOperations, PeerId,
};
use tracing::{debug, error};

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
    T: HoprDbPeersOperations + HoprDbResolverOperations + Sync + Send + Clone + std::fmt::Debug,
{
    network: Arc<Network<T>>,
    resolver: T,
    channel_graph: Arc<RwLock<ChannelGraph>>,
    /// Implementation of the network interface allowing emitting events
    /// based on the [hopr_transport_network::network::Network] events into the p2p swarm.
    emitter: futures::channel::mpsc::Sender<NetworkTriggeredEvent>,
}

impl<T> PingExternalInteractions<T>
where
    T: HoprDbPeersOperations + HoprDbResolverOperations + Sync + Send + Clone + std::fmt::Debug,
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
    T: HoprDbPeersOperations + HoprDbResolverOperations + Sync + Send + Clone + std::fmt::Debug,
{
    #[tracing::instrument(level = "info", skip(self))]
    async fn on_finished_ping(
        &self,
        peer: &PeerId,
        result: &hopr_transport_network::errors::Result<std::time::Duration>,
        version: String,
    ) {
        let result = match &result {
            Ok(duration) => Ok(*duration),
            Err(_) => Err(()),
        };

        // Update the channel graph
        if let Ok(pk) = OffchainPublicKey::try_from(peer) {
            let maybe_chain_key = self.resolver.resolve_chain_key(&pk).await;
            if let Ok(Some(chain_key)) = maybe_chain_key {
                let mut g = self.channel_graph.write().await;
                g.update_node_score(&chain_key, result.into());
                debug!(%chain_key, ?result, "update node score for peer");
            } else {
                error!(%peer, "could not resolve chain key ");
            }
        } else {
            error!(%peer, "encountered invalid peer id:");
        }

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
                        error!(error = %e, "Failed to emit a network event 'close connection'")
                    }
                }
                NetworkTriggeredEvent::UpdateQuality(peer, quality) => {
                    debug!("'{peer}' changed quality to '{quality}'");
                }
            },
            Ok(None) => debug!("No update necessary"),
            Err(e) => error!(error = %e, "Encountered error on on updating the collected ping data"),
        }
    }
}
