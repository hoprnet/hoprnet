/// List of all errors that can occur in the PIX protocol.
#[derive(Debug, thiserror::Error)]
pub enum PixError {
    #[error("invalid input to the function")]
    InvalidInput,
    #[error("ssa commitment does not match ssa")]
    InvalidSsa,
    #[error("missing commitment for building ssa")]
    MissingSsaCommitment,
    #[error("missing verifier for partial ssa reconstruction")]
    MissingVerifier,
    #[error("crypto error: {0}")]
    CryptoError(#[from] hopr_types::crypto::errors::CryptoError),
    #[error("ecc calculation error: {0}")]
    EccError(#[from] vsss_rs::elliptic_curve::Error),
    #[error("secret sharing error: {0}")]
    VsssError(vsss_rs::Error),
}

impl From<vsss_rs::Error> for PixError {
    fn from(err: vsss_rs::Error) -> Self {
        PixError::VsssError(err)
    }
}

pub type Result<T> = std::result::Result<T, PixError>;
