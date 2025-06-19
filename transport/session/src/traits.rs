use hopr_network_types::prelude::DestinationRouting;
use hopr_transport_packet::prelude::ApplicationData;

use crate::errors::TransportSessionError;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SendMsg {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: DestinationRouting,
    ) -> std::result::Result<(), TransportSessionError>;
}
