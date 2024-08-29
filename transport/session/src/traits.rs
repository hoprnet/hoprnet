use crate::errors::TransportSessionError;
use hopr_internal_types::protocol::ApplicationData;
use hopr_network_types::prelude::RoutingOptions;
use libp2p_identity::PeerId;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SendMsg {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: PeerId,
        options: RoutingOptions,
    ) -> std::result::Result<(), TransportSessionError>;

    fn close(&self);
}
