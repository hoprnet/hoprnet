use core_network::{
    network::{NetworkEvent, NetworkExternalActions},
    PeerId,
};
use futures::channel::mpsc::Sender;
use log::error;

use hopr_platform::time::native::current_timestamp;

#[derive(Debug, Clone)]
pub struct ExternalNetworkInteractions {
    emitter: Sender<NetworkEvent>,
}

impl ExternalNetworkInteractions {
    pub fn new(emitter: Sender<NetworkEvent>) -> Self {
        Self { emitter }
    }
}

impl NetworkExternalActions for ExternalNetworkInteractions {
    fn is_public(&self, _: &PeerId) -> bool {
        // NOTE: In the Providence release all nodes are public
        true
    }

    fn emit(&self, event: NetworkEvent) {
        if let Err(e) = self.emitter.clone().start_send(event.clone()) {
            error!("Failed to emit a network status: {}: {}", event, e)
        }
    }
    fn create_timestamp(&self) -> u64 {
        current_timestamp().as_millis() as u64
    }
}
