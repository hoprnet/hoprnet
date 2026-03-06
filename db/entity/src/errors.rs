use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbEntityError {
    #[error("conversion error: {0}")]
    ConversionError(String),

    #[error(transparent)]
    TypesError(#[from] hopr_api::types::internal::errors::CoreTypesError),

    #[error(transparent)]
    CryptoError(#[from] hopr_api::types::crypto::errors::CryptoError),

    #[error(transparent)]
    GeneralError(#[from] hopr_api::types::primitive::errors::GeneralError),
}

pub type Result<T> = std::result::Result<T, DbEntityError>;
