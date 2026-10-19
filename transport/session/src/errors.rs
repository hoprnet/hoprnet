use hopr_api::types::crypto::prelude::SimplePseudonym;
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
    Network(#[from] hopr_utils::network_types::errors::NetworkTypeError),

    #[error("session is closed")]
    Closed,

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
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
    #[error("received a Start protocol message that is not supported")]
    UnsupportedMessage,
    #[error("one of the parties attempted to negotiate unacceptable Session parameters: {0}")]
    Unacceptable(String),
    /// A Start protocol request was dropped without being served, and nothing about the Session was
    /// found wrong.
    ///
    /// The counterpart of [`Unacceptable`](Self::Unacceptable) for refusals that are *not* terminal. It
    /// is returned for exactly five causes: the request lost a race for the per-Session request lock;
    /// it came too early, before emission reached the successor boundary; it is under-served, since
    /// the returned packets have not yet earned it; it is stale, naming indices at or below the
    /// committed watermark that are not the batch last committed; or the transport cut short the
    /// publication of a commitment — the Entry's own, or the resumption of one a repeat or a scoped
    /// re-request prompted. The Session keeps running and the peer is told nothing.
    ///
    /// Whether a later request succeeds depends on the cause. The first three can succeed on a later
    /// re-send of the same request, once the lock is free or the Entry's state has caught up, and the
    /// Exit does re-send a successor while it is unanswered and its batch is behind the front. A stale
    /// request is never admitted. A cut-short publication is finished only if the Exit asks again: by
    /// re-requesting parts of a commitment that began to arrive, which works wherever the batch stands,
    /// or by repeating the request, which it does while some member of the batch has reported nothing
    /// but only while the batch is behind the front. Nothing asks for a member with no message of it
    /// out whose batch holds the front (the opening batch always does, so any member after a boundary
    /// cut there), nor when the whole burst went out and only the announcement failed, and then the
    /// Exit's own deadline ends the cycle. Nothing is stranded for that member, because nothing was
    /// announced for it.
    ///
    /// A repeat of the batch last committed otherwise returns `Ok(())` — as a no-op if it was published
    /// in full, and after finishing the publication if it was not. Callers that report errors should
    /// treat this as an ordinary outcome rather than a fault.
    #[error("the request was not admitted: {0}")]
    NotAdmitted(String),
    #[error("deposit pool did not supply deposit data for the SSA batch: {0}")]
    MissingDepositData(String),
    #[error(transparent)]
    PixError(#[from] hopr_protocol_pix::errors::PixError<SimplePseudonym>),
    #[error(transparent)]
    Other(anyhow::Error),
}

impl SessionManagerError {
    pub fn other<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::Other(e.into())
    }
}

pub type Result<T> = std::result::Result<T, TransportSessionError>;
