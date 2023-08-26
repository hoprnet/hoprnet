use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;

use core_network::{heartbeat::HeartbeatExternalApi, network::Network, PeerId};

use crate::adaptors::network::ExternalNetworkInteractions;

/// Implementor of the heartbeat external API.
///
/// Heartbeat requires functionality from external components in order to obtain
/// the triggers for its functionality. This class implements the basic API by
/// aggregating all necessary heartbeat resources without leaking them into the
/// `Heartbeat` object and keeping both the adaptor and the heartbeat object
/// OCP and SRP compliant.
pub struct HeartbeatExternalInteractions {
    network: Arc<RwLock<Network<ExternalNetworkInteractions>>>,
}

impl HeartbeatExternalInteractions {
    pub fn new(network: Arc<RwLock<Network<ExternalNetworkInteractions>>>) -> Self {
        Self { network }
    }
}

#[async_trait]
impl HeartbeatExternalApi for HeartbeatExternalInteractions {
    /// Get all peers considered by the `Network` object to be pingable.
    ///
    /// After a duration of non-pinging based specified by the configurable threshold.
    async fn get_peers(&self, from_timestamp: u64) -> Vec<PeerId> {
        let reader = self.network.write().await;
        (*reader).find_peers_to_ping(from_timestamp)
    }
}
