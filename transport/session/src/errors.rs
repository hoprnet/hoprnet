use thiserror::Error;

/// Enumeration of errors thrown from this library.
#[derive(Error, Debug)]
pub enum TransportSessionError {
    #[error("session operation timed out")]
    Timeout,

    #[error("unparseable session id")]
    InvalidSessionId,

    #[error("the other party rejected session initiation with error: {0}")]
    Rejected(hopr_protocol_start::StartErrorReason),

    #[error("received data for an unregistered session")]
    UnknownData,

    #[error("packet sending error: {0}")]
    PacketSendingError(anyhow::Error),

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

impl TransportSessionError {
    pub fn packet_sending<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::PacketSendingError(e.into())
    }
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
    #[error(transparent)]
    Other(anyhow::Error),
}

impl SessionManagerError {
    pub fn other<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::Other(e.into())
    }
}

pub type Result<T> = std::result::Result<T, TransportSessionError>;
