use crate::session::FrameId;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkTypeError {
    #[error("attempt to insert invalid frame id")]
    InvalidFrameId,

    #[error("frame cannot be segmented because it is too long")]
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

    #[error(transparent)]
    SessionProtocolError(#[from] crate::session::errors::SessionError),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, NetworkTypeError>;
