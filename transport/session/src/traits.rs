use hopr_internal_types::protocol::ApplicationData;
use libp2p_identity::PeerId;
use thiserror::Error;

use crate::SendOptions;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Connection timed out")]
    Timeout,
    #[error("Application tag from unallowed range")]
    Tag,
    #[error("Impossible transport path")]
    Path,
    #[error("Session is closed")]
    Closed,
}

#[async_trait::async_trait]
pub trait SendMsg {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: PeerId,
        options: SendOptions,
    ) -> std::result::Result<(), SessionError>;
}
