use async_trait::async_trait;
use hopr_transport_network::traits::{NetworkObservations, NetworkView};
use hopr_transport_p2p::HoprNetwork;
use hopr_transport_probe::traits::{PeerDiscoveryFetch, ProbeStatusUpdate};
use rand::seq::SliceRandom;

// TODO: replace with telemetry:
// hopr_metrics::SimpleHistogram::new(
// "hopr_ping_time_sec",
// "Measures total time it takes to ping a single node (seconds)",
// vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 15.0, 30.0],

// TODO: replace with telemetry:
// static ref METRIC_PROBE_COUNT:  hopr_metrics::MultiCounter =  hopr_metrics::MultiCounter::new(
// "hopr_probe_count",
//             "Total number of pings by result",
//             &["success"]

/// Implementor of the ping external API.
///
/// Ping requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary ping resources without leaking them into the
/// `Ping` object and keeping both the adaptor and the ping object OCP and SRP
/// compliant.
#[derive(Debug, Clone)]
pub struct ProbeNetworkInteractions {
    network: HoprNetwork,
}

impl ProbeNetworkInteractions {
    pub fn new(network: HoprNetwork) -> Self {
        Self { network }
    }
}

#[async_trait]
impl PeerDiscoveryFetch for ProbeNetworkInteractions {
    /// Get all peers considered by the `Network` to be pingable.
    ///
    /// After a duration of non-pinging based specified by the configurable threshold.
    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<hopr_transport_network::PeerId> {
        tracing::trace!(?from_timestamp, "fetching peers for probing, ignoring timestamp");
        let mut vec = self.network.discovered_peers().into_iter().collect::<Vec<_>>();
        vec.shuffle(&mut rand::thread_rng());
        vec
    }
}

#[async_trait]
impl ProbeStatusUpdate for ProbeNetworkInteractions {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn on_finished(
        &self,
        peer: &hopr_transport_network::PeerId,
        result: &hopr_transport_probe::errors::Result<std::time::Duration>,
    ) {
        let result = match &result {
            Ok(duration) => Ok(*duration),
            Err(error) => {
                tracing::trace!(%error, "Encountered timeout on peer ping");
                Err(())
            }
        };

        self.network.update(peer, result);
    }
}
