use thiserror::Error;

#[derive(Error, Debug)]
pub enum SphinxError {
    #[error("failed to decode packet: {0}")]
    PacketDecodingError(String),

    #[error("failed to construct packet: {0}")]
    PacketConstructionError(String),

    #[error("data could not be padded")]
    PaddingError,

    /// No [`ReplyOpener`](super::surb::ReplyOpener) is stored for the reply's sender id.
    ///
    /// A typed variant rather than a formatted [`SphinxError::PacketDecodingError`] because a caller has to tell it
    /// apart: for a reply it means "a concurrent copy of this packet peeked the opener first", which is a different
    /// outcome from a malformed packet.
    #[error("no reply opener stored for the reply's sender id")]
    MissingReplyOpener,

    #[error(transparent)]
    CryptoError(#[from] hopr_types::crypto::errors::CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_types::primitive::errors::GeneralError),
}
