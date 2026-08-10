use crate::SsaIndex;

/// List of all errors that can occur in the PIX protocol.
#[derive(Debug, thiserror::Error)]
pub enum PixError<P: std::fmt::Display> {
    #[error("invalid input to the function")]
    InvalidInput,
    #[error("invalid configuration: {0}")]
    InvalidConfiguration(#[from] validator::ValidationErrors),
    #[error("acknowledgement from this peer is not paired to any encrypted share")]
    UnexpectedShare,
    /// No longer produced by the reconstructor: a share failing verification is expected adversarial
    /// input rather than a fault in the call, and the caller needs the cycle's running fault total
    /// along with it — which an error cannot carry without becoming a telemetry channel. It surfaces
    /// as [`ShareResolution::InvalidShares`](crate::ShareResolution::InvalidShares) instead.
    ///
    /// Retained because it is what makes [`PixError`] generic over the pseudonym; dropping the
    /// parameter touches every signature in the crate and is its own change.
    #[error("received an ssa share from pseudonym {0} #{1} that could not be verified")]
    InvalidShare(P, SsaIndex),
    #[error("encrypted partial ssa share is empty")]
    ShareIsEmpty,
    /// The share was dropped rather than buffered: the reconstructor is already holding
    /// `max_ack_buffer_bytes` worth of shares awaiting acknowledgement.
    ///
    /// Deliberately *not* an expected error. Reaching it means share loss — the Exit has put a
    /// packet on the wire whose acknowledgement will now find nothing — so it should be as loud as
    /// the caller's log level allows.
    #[error("awaiting-acknowledgement buffer is at its configured byte budget; share dropped")]
    AckBufferFull,
    #[error("ssa commitment does not match ssa")]
    InvalidSsa,
    #[error("received duplicate commitment")]
    DuplicateCommitment,
    #[error("missing commitment for building ssa")]
    MissingSsaCommitment,
    #[error(
        "client ssa commitment is not accompanied by a valid proof of knowledge of its discrete logarithm — the \
         sender may be attempting to make the deposit key recoverable by itself alone"
    )]
    UnprovenSsaCommitment,
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
