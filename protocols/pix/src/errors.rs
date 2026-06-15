use crate::SsaIndex;

/// List of all errors that can occur in the PIX protocol.
#[derive(Debug, thiserror::Error)]
pub enum PixError<P: std::fmt::Display> {
    #[error("invalid input to the function")]
    InvalidInput,
    #[error("acknowledgement from this peer is not paired to any encrypted share")]
    UnexpectedShare,
    #[error("received an ssa share from pseudonym {0} #{1} that could not be verified")]
    InvalidShare(P, SsaIndex),
    #[error("encrypted partial ssa share is empty")]
    ShareIsEmpty,
    #[error("ssa commitment does not match ssa")]
    InvalidSsa,
    #[error("received duplicate commitment")]
    DuplicateCommitment,
    #[error("missing commitment for building ssa")]
    MissingSsaCommitment,
    #[error("missing verifier for partial ssa reconstruction")]
    MissingVerifier,
    #[error("ssa index will overflow")]
    SsaIndexOverflow,
    #[error("crypto error: {0}")]
    CryptoError(#[from] hopr_types::crypto::errors::CryptoError),
    #[error("ecc calculation error: {0}")]
    EccError(#[from] vsss_rs::elliptic_curve::Error),
    #[error("secret sharing error: {0}")]
    VsssError(vsss_rs::Error),
}

impl<P: std::fmt::Display> From<vsss_rs::Error> for PixError<P> {
    fn from(err: vsss_rs::Error) -> Self {
        PixError::VsssError(err)
    }
}

pub type Result<T, P> = std::result::Result<T, PixError<P>>;
