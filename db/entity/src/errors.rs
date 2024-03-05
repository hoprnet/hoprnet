use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbEntityError {
    #[error("conversion error: {0}")]
    ConversionError(String),

    #[error(transparent)]
    TypesError(#[from] hopr_internal_types::errors::CoreTypesError),

    #[error(transparent)]
    CryptoError(#[from] hopr_crypto_types::errors::CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError)
}

pub type Result<T> = std::result::Result<T, DbEntityError>;
