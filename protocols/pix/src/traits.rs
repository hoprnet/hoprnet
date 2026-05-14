use hopr_types::crypto::prelude::{HalfKeyChallenge, OffchainPublicKey};
use hopr_types::internal::prelude::Acknowledgement;
use crate::{EncryptedPartialSsaShare, PixScalar, PixSpec};

pub trait ExitAcknowledgementShareProcessor<S: PixSpec> {
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn insert_encrypted_share(
        &self,
        peer: OffchainPublicKey,
        challenge: HalfKeyChallenge,
        pseudonym: &S::Pseudonym,
        msg: &impl AsRef<[u8]>,
        enc: EncryptedPartialSsaShare<S>,
    ) -> Result<(), Self::Error>;
    
    fn acknowledge_shares(
        &self,
        peer: OffchainPublicKey,
        acks: Vec<Acknowledgement>
    ) -> Result<Vec<PixScalar<S>>, Self::Error>;
}