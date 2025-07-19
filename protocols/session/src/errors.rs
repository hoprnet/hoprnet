use hopr_primitive_types::prelude::GeneralError;
use thiserror::Error;

use crate::frames::FrameId;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("error while processing frame or segment: {0}")]
    ProcessingError(String),

    #[error("socket is in invalid state: {0}")]
    InvalidState(String),

    #[error("socket state is not running")]
    StateNotRunning,

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

    #[error("input data exceeds the maximum allowed size of the message")]
    DataTooLong,

    #[error("cannot reassemble frame {0}, because it is not complete")]
    IncompleteFrame(FrameId),

    #[error("there are too many incomplete frames in the reassembler")]
    TooManyIncompleteFrames,

    #[error("segment could not be parsed correctly")]
    InvalidSegment,

    #[error("received a segment of a frame {0} that was already completed or evicted")]
    OldSegment(FrameId),

    #[error("frame {0} has expired or has been discarded")]
    FrameDiscarded(FrameId),

    #[error("frame reassembler is closed")]
    ReassemblerClosed,

    #[error("invalid size of a segment was specified")]
    InvalidSegmentSize,

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    GeneralError(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, SessionError>;
