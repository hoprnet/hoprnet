use libp2p_identity::PeerId;

use crate::errors::CryptoError;
use crate::keypairs::OffchainKeypair;
use crate::types::OffchainPublicKey;

/// Performs randomized encryption of the given serializable object, so that
/// only the recipient with the given `peer_id` can [decrypt it](unseal_data).
pub fn seal_data<T: serde::Serialize>(data: T, peer_id: PeerId) -> crate::errors::Result<Box<[u8]>> {
    let recipient_pk: crypto_box::PublicKey =
        curve25519_dalek::MontgomeryPoint::from(&OffchainPublicKey::try_from(peer_id)?).into();

    let plain_text = bincode::serialize(&data).map_err(|_| CryptoError::InvalidInputValue)?;

    recipient_pk
        .seal(&mut hopr_crypto_random::rng(), &plain_text)
        .map_err(|_| CryptoError::SealingError)
        .map(|vec| vec.into_boxed_slice())
}

/// Decrypts a deserializable data object previously encrypted with [`seal_data`].
///
/// The given `keypair` must correspond to the `peer_id` given during encryption.
pub fn unseal_data<T: for<'a> serde::Deserialize<'a>>(
    data: &[u8],
    keypair: &OffchainKeypair,
) -> crate::errors::Result<T> {
    let recipient_sk = crypto_box::SecretKey::from(curve25519_dalek::scalar::Scalar::from(keypair));

    recipient_sk
        .unseal(data)
        .map_err(|_| CryptoError::SealingError)
        .and_then(|vec| bincode::deserialize::<T>(&vec).map_err(|_| CryptoError::SealingError))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypairs::Keypair;
    use std::ops::Not;

    #[test]
    fn seal_unseal_should_be_identity() -> anyhow::Result<()> {
        let data = "some test data".to_string();

        let keypair = OffchainKeypair::random();

        let sealed = seal_data(data.clone(), keypair.public().into())?;

        let unsealed: String = unseal_data(&sealed, &keypair)?;

        assert_eq!(data, unsealed);

        Ok(())
    }

    #[test]
    fn unseal_should_fail_with_different_private_key() -> anyhow::Result<()> {
        let data = "some test data".to_string();

        let keypair_1 = OffchainKeypair::random();
        let keypair_2 = OffchainKeypair::random();

        let sealed = seal_data(data.clone(), keypair_1.public().into())?;

        assert_eq!(
            Err(CryptoError::SealingError),
            unseal_data::<String>(&sealed, &keypair_2)
        );

        Ok(())
    }

    #[test]
    fn unseal_should_fail_when_ciphertext_has_been_tampered_with() -> anyhow::Result<()> {
        let data = "some test data".to_string();

        let keypair = OffchainKeypair::random();

        let mut sealed = seal_data(data.clone(), keypair.public().into())?;
        sealed[1] = sealed[1].not();

        assert_eq!(Err(CryptoError::SealingError), unseal_data::<String>(&sealed, &keypair));

        Ok(())
    }
}
