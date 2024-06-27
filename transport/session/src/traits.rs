use hopr_internal_types::protocol::ApplicationData;
use libp2p_identity::PeerId;

use crate::{errors::TransportSessionError, PathOptions};

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SendMsg {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: PeerId,
        options: PathOptions,
    ) -> std::result::Result<(), TransportSessionError>;
}
