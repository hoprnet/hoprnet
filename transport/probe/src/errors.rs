use libp2p_identity::PeerId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProbeError {
    #[error("the network operation timed out after {0} seconds")]
    Timeout(u64),

    #[error("error while decoding message data")]
    DecodingError,

    #[error("peer does not exist")]
    NonExistingPeer,

    #[error("error while pinging peer {0}: {1}")]
    PingerError(PeerId, String),

    #[error("error sending probe: {0}")]
    SendError(String),
}

pub type Result<T> = core::result::Result<T, ProbeError>;
