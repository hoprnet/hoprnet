use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkTypeError {
    #[error("attempt to insert invalid frame id")]
    InvalidFrameId,

    #[error("cannot reassemble frame because it is not complete")]
    IncompleteFrame,

    #[error("segment could not be parsed correctly")]
    InvalidSegment,

    #[error("frame reassembler is closed")]
    ReassemblerClosed,
}

pub type Result<T> = std::result::Result<T, NetworkTypeError>;
