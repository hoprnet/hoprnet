use std::sync::Arc;

use async_lock::RwLock;
use async_trait::async_trait;

use core_network::{PeerId, network::Network, ping::PingExternalAPI, types::Result};

use crate::adaptors::network::ExternalNetworkInteractions;


#[derive(Clone)]
pub struct PingExternalInteractions {
    network: Arc<RwLock<Network<ExternalNetworkInteractions>>>
}

impl PingExternalInteractions {
    pub fn new(network: Arc<RwLock<Network<ExternalNetworkInteractions>>>) -> Self {
        Self { network }
    }
}

#[async_trait]
impl PingExternalAPI for PingExternalInteractions {
    async fn on_finished_ping(&self, peer: &PeerId, result: Result) {
        let mut writer = self.network.write().await;
        (*writer).update_with_metadata(peer, result, None)
    }
}