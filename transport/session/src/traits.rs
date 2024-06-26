use hopr_internal_types::protocol::ApplicationData;
use libp2p_identity::PeerId;

use crate::{errors::TransportSessionError, SendOptions};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SendMsg {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: PeerId,
        options: SendOptions,
    ) -> std::result::Result<(), TransportSessionError>;
}
