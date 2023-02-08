use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("error while parsing or deserializing data")]
    ParseError,
    #[error("error computing the result, possibly cause by invalid input")]
    MathError
}

pub type Result<T> = core::result::Result<T, GeneralError>;