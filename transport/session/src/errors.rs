use thiserror::Error;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum TransportSessionError {
    #[error("connection timed out")]
    Timeout,

    #[error("incorrect data size")]
    PayloadSize,

    #[error("invalid peer id")]
    PeerId,

    #[error("impossible transport path")]
    Path,

    #[error("no surb available for sending reply data")]
    OutOfSurbs,

    #[error("the other party rejected session initiation with error: {0}")]
    Rejected(hopr_protocol_start::StartErrorReason),

    #[error("received data for an unregistered session")]
    UnknownData,

    #[error("packet sending error: {0}")]
    PacketSendingError(String),

    #[error(transparent)]
    StartProtocolError(#[from] hopr_protocol_start::errors::StartProtocolError),

    #[error(transparent)]
    SessionProtocolError(#[from] hopr_protocol_session::errors::SessionError),

    #[error(transparent)]
    Manager(#[from] SessionManagerError),

    #[error(transparent)]
    Network(#[from] hopr_network_types::errors::NetworkTypeError),

    #[error("session is closed")]
    Closed,
}

impl From<TransportSessionError> for std::io::Error {
    fn from(error: TransportSessionError) -> Self {
        std::io::Error::other(error)
    }
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
