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
    /// A typed variant rather than a formatted [`SphinxError::PacketDecodingError`] so a caller can recognize the
    /// opener-cache miss specifically. It says nothing about *why* the opener is gone: a concurrent copy of the same
    /// reply may have consumed it, or it may have expired, been evicted, or been lost with the cache on restart. The
    /// pseudonym is carried so a log line points at the pseudonym whose openers went missing.
    #[error("no reply opener stored for the reply's sender id (pseudonym {0})")]
    MissingReplyOpener(String),

    #[error(transparent)]
    CryptoError(#[from] hopr_types::crypto::errors::CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_types::primitive::errors::GeneralError),
}
