//! Deprecated: glue layer for the code that connects probing to the new network API
//! defined by the `hopr_api`.

use async_trait::async_trait;
use hopr_transport_network::traits::{NetworkObservations, NetworkView};
use hopr_transport_p2p::HoprNetwork;
use hopr_transport_probe::traits::{PeerDiscoveryFetch, ProbeStatusUpdate};
use rand::seq::SliceRandom;

/// Implementor of the 0-hop probing external API.
///
/// Probing requires functionality from external components in order to obtain
/// the triggers for its functionality. This struct implements the basic API by
/// aggregating all necessary network resources without leaking them into the
/// `Probe` object, keeping both the adaptor and the probe object OCP and SRP
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
    /// Get all peers considered by the `Network` to be probeable.
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
                tracing::trace!(%error, "Encountered timeout on peer probe");
                Err(())
            }
        };

        self.network.update(peer, result);
    }
}
