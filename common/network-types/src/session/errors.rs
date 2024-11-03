use crate::prelude::FrameId;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SessionError {
    #[error("error while processing frame or segment: {0}")]
    ProcessingError(String),

    #[error("failed to parse session message")]
    ParseError,

    #[error("invalid protocol version")]
    WrongVersion,

    #[error("message has an incorrect length")]
    IncorrectMessageLength,

    #[error("the message has an unknown tag")]
    UnknownMessageTag,

    #[error("session is closed")]
    SessionClosed,

    #[error("attempt to insert invalid frame id")]
    InvalidFrameId,

    #[error("input data exceeds the maximum allowed size of segment")]
    DataTooLong,

    #[error("cannot reassemble frame {0}, because it is not complete")]
    IncompleteFrame(FrameId),

    #[error("segment could not be parsed correctly")]
    InvalidSegment,

    #[error("received a segment of a frame {0} that was already completed or evicted")]
    OldSegment(FrameId),

    #[error("frame {0} has expired and has been evicted")]
    FrameDiscarded(FrameId),

    #[error("frame reassembler is closed")]
    ReassemblerClosed,

    #[error("invalid size of a segment was specified")]
    InvalidSegmentSize,
}

pub type Result<T> = std::result::Result<T, SessionError>;
