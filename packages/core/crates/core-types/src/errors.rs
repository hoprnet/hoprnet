use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreTypesError {
    #[error("{0}")]
    InvalidInputData(String),

    #[error("failed to parse/deserialize the data of {0}")]
    ParseError(String),
}

pub type Result<T> = core::result::Result<T, CoreTypesError>;
