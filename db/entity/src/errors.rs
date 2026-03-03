use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbEntityError {
    #[error("conversion error: {0}")]
    ConversionError(String),

    #[error(transparent)]
    TypesError(#[from] hopr_types::internal::errors::CoreTypesError),

    #[error(transparent)]
    CryptoError(#[from] hopr_types::crypto::errors::CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_types::primitive::errors::GeneralError),
}

pub type Result<T> = std::result::Result<T, DbEntityError>;
