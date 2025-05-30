use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_sql::api::resolver::HoprDbResolverOperations;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, SimpleHistogram};
use hopr_path::channel_graph::ChannelGraph;
use hopr_transport_network::{
    HoprDbPeersOperations,
    network::{Network, NetworkTriggeredEvent},
};
use hopr_transport_probe::traits::{PeerDiscoveryFetch, ProbeStatusUpdate};
use tracing::{debug, error, trace};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_PING: SimpleHistogram =
        SimpleHistogram::new(
            "hopr_ping_time_sec",
            "Measures total time it takes to ping a single node (seconds)",
            vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0],
        ).unwrap();
    static ref METRIC_PROBE_COUNT: MultiCounter = MultiCounter::new(
            "hopr_probe_count",
            "Total number of pings by result",
            &["success"]
        ).unwrap();
}

/// Implementor of the ping external API.
///
/// Ping requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary ping resources without leaking them into the
/// `Ping` object and keeping both the adaptor and the ping object OCP and SRP
/// compliant.
#[derive(Debug, Clone)]
pub struct ProbeNetworkInteractions<T>
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

impl<T> ProbeNetworkInteractions<T>
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
impl<T> PeerDiscoveryFetch for ProbeNetworkInteractions<T>
where
    T: HoprDbPeersOperations + HoprDbResolverOperations + Sync + Send + Clone + std::fmt::Debug,
{
    /// Get all peers considered by the `Network` to be pingable.
    ///
    /// After a duration of non-pinging based specified by the configurable threshold.
    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<hopr_transport_network::PeerId> {
        self.network
            .find_peers_to_ping(from_timestamp)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to generate peers for the heartbeat procedure");
                vec![]
            })
    }
}

#[async_trait]
impl<T> ProbeStatusUpdate for ProbeNetworkInteractions<T>
where
    T: HoprDbPeersOperations + HoprDbResolverOperations + Sync + Send + Clone + std::fmt::Debug,
{
    #[tracing::instrument(level = "debug", skip(self))]
    async fn on_finished(
        &self,
        peer: &hopr_transport_network::PeerId,
        result: &hopr_transport_probe::errors::Result<std::time::Duration>,
    ) {
        let result = match &result {
            Ok(duration) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    METRIC_TIME_TO_PING.observe((duration.as_millis() as f64) / 1000.0); // precision for seconds
                    METRIC_PROBE_COUNT.increment(&["true"]);
                }

                Ok(*duration)
            }
            Err(_) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_PROBE_COUNT.increment(&["false"]);

                Err(())
            }
        };

        // Update the channel graph
        if let Ok(pk) = OffchainPublicKey::try_from(peer) {
            let maybe_chain_key = self.resolver.resolve_chain_key(&pk).await;
            if let Ok(Some(chain_key)) = maybe_chain_key {
                let mut g = self.channel_graph.write().await;
                g.update_node_score(&chain_key, result.into());
                debug!(%chain_key, ?result, "update node score for peer");
            } else {
                error!(%peer, "could not resolve chain key");
            }
        } else {
            error!(%peer, "encountered invalid peer id");
        }

        match self.network.update(peer, result, None).await {
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
                    tracing::trace!(%peer, %quality, "peer changed quality");
                }
            },
            Ok(None) => trace!("No update necessary"),
            Err(e) => error!(error = %e, "Encountered error on on updating the collected ping data"),
        }
    }
}
