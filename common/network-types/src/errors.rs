use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkTypeError {
    #[error("attempt to insert invalid frame id")]
    InvalidFrameId,

    #[error("cannot reassemble frame {0}, because it is not complete")]
    IncompleteFrame(u32),

    #[error("segment could not be parsed correctly")]
    InvalidSegment,

    #[error("received a segment of a frame {0} that was already completed or evicted")]
    OldSegment(u32),

    #[error("frame reassembler is closed")]
    ReassemblerClosed,

    #[error("cannot parse session protocol message")]
    InvalidSessionMessage,
}

pub type Result<T> = std::result::Result<T, NetworkTypeError>;
