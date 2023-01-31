use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("error while parsing or deserializing data")]
    ParseError
}