use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;

use core_network::{PeerId, network::Network, heartbeat::HeartbeatExternalApi};

use crate::adaptors::network::ExternalNetworkInteractions;

pub struct HeartbeatExternalInteractions {
    network: Arc<RwLock<Network<ExternalNetworkInteractions>>>
}

impl HeartbeatExternalInteractions {
    pub fn new(network: Arc<RwLock<Network<ExternalNetworkInteractions>>>) -> Self {
        Self { network }
    }
}

#[async_trait]
impl HeartbeatExternalApi for HeartbeatExternalInteractions {
    async fn get_peers(&self, from_timestamp: u64) -> Vec<PeerId> {
        let reader = self.network.write().await;
        (*reader).find_peers_to_ping(from_timestamp)
    }
}