use thiserror::Error;

/// Error representing all possible erroneous states of the HOPR config.
#[derive(Error, Debug)]
pub enum HoprConfigError {
    #[error("configuration error: {0}")]
    Configuration(String),
}

/// The default [Result] object translating errors in the [HoprChainError] type
pub type Result<T> = core::result::Result<T, HoprConfigError>;
