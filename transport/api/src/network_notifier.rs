use std::sync::Arc;

use async_trait::async_trait;
use hopr_api::db::HoprDbPeersOperations;
use hopr_transport_network::network::{Network, UpdateFailure};
use hopr_transport_probe::traits::{PeerDiscoveryFetch, ProbeStatusUpdate};
use tracing::error;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_TIME_TO_PING:  hopr_metrics::SimpleHistogram =
         hopr_metrics::SimpleHistogram::new(
            "hopr_ping_time_sec",
            "Measures total time it takes to ping a single node (seconds)",
            vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0],
        ).unwrap();
    static ref METRIC_PROBE_COUNT:  hopr_metrics::MultiCounter =  hopr_metrics::MultiCounter::new(
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
pub struct ProbeNetworkInteractions<Db> {
    network: Arc<Network<Db>>,
}

impl<Db> ProbeNetworkInteractions<Db>
where
    Db: HoprDbPeersOperations + Sync + Send + Clone,
{
    pub fn new(network: Arc<Network<Db>>) -> Self {
        Self { network }
    }
}

#[async_trait]
impl<Db> PeerDiscoveryFetch for ProbeNetworkInteractions<Db>
where
    Db: HoprDbPeersOperations + Sync + Send + Clone,
{
    /// Get all peers considered by the `Network` to be pingable.
    ///
    /// After a duration of non-pinging based specified by the configurable threshold.
    #[tracing::instrument(level = "trace", skip(self))]
    async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<hopr_transport_network::PeerId> {
        self.network
            .find_peers_to_ping(from_timestamp)
            .await
            .unwrap_or_else(|error| {
                tracing::error!(%error, "failed to generate peers for the heartbeat procedure");
                vec![]
            })
    }
}

#[async_trait]
impl<Db> ProbeStatusUpdate for ProbeNetworkInteractions<Db>
where
    Db: HoprDbPeersOperations + Sync + Send + Clone,
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
            Err(error) => {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_PROBE_COUNT.increment(&["false"]);

                tracing::trace!(%error, "Encountered timeout on peer ping");
                Err(UpdateFailure::Timeout)
            }
        };

        if let Err(error) = self.network.update(peer, result).await {
            error!(%error, "Encountered error on on updating the collected ping data")
        }
    }
}
