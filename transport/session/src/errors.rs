use thiserror::Error;

use crate::initiation::StartErrorReason;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum TransportSessionError {
    #[error("connection timed out")]
    Timeout,

    #[error("incorrect data size")]
    PayloadSize,

    #[cfg(feature = "serde")]
    #[error("serializer encoding error: {0}")]
    SerializerEncoding(#[from] bincode::error::EncodeError),

    #[cfg(feature = "serde")]
    #[error("serializer decoding error: {0}")]
    SerializerDecoding(#[from] bincode::error::DecodeError),

    #[error("invalid peer id")]
    PeerId,

    #[error("impossible transport path")]
    Path,

    #[error("no surb available for sending reply data")]
    OutOfSurbs,

    #[error("the other party rejected session initiation with error: {0}")]
    Rejected(StartErrorReason),

    #[error("received data for an unregistered session")]
    UnknownData,

    #[error("session establishment protocol error: {0}")]
    StartProtocolError(String),

    #[error(transparent)]
    Manager(#[from] SessionManagerError),

    #[error(transparent)]
    Network(#[from] hopr_network_types::errors::NetworkTypeError),

    #[error("session is closed")]
    Closed,
}

#[derive(Error, Debug)]
pub enum SessionManagerError {
    #[error("manager is not started")]
    NotStarted,
    #[error("manager is already started")]
    AlreadyStarted,
    #[error("all challenge slots are occupied")]
    NoChallengeSlots,
    #[error("session with the given id does not exist")]
    NonExistingSession,
    #[error("number of sessions exceeds the maximum allowed")]
    TooManySessions,
    #[error("loopback sessions are not allowed")]
    Loopback,
    #[error("non-specific session manager error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, TransportSessionError>;
