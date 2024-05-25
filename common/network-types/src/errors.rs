use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkTypeError {
    #[error("frame reassembler is closed")]
    ReassemblerClosed,
}

pub type Result<T> = std::result::Result<T, NetworkTypeError>;
