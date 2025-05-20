use libp2p_identity::PeerId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProbeError {
    #[error("the network operation timed out after {0} seconds")]
    Timeout(u64),

    #[error("error while decoding message data")]
    DecodingError,

    #[error("error in the messaging sub-protocol: {0}")]
    MessagingError(String),

    #[error("peer does not exist")]
    NonExistingPeer,

    #[error("error while pinging peer {0}: {1}")]
    PingerError(PeerId, String),
}

pub type Result<T> = core::result::Result<T, ProbeError>;
