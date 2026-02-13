use libp2p_identity::PeerId;

use crate::{errors::CryptoError, keypairs::OffchainKeypair, types::OffchainPublicKey};

/// Performs randomized encryption of the given data, so that
/// only the recipient with the given `peer_id` can [decrypt it](unseal_data).
pub fn seal_data(data: &[u8], peer_id: PeerId) -> crate::errors::Result<Box<[u8]>> {
    let recipient_pk: crypto_box::PublicKey =
        curve25519_dalek::MontgomeryPoint::from(&OffchainPublicKey::from_peerid(&peer_id)?).into();

    recipient_pk
        .seal(&mut hopr_crypto_random::rng(), data)
        .map_err(|_| CryptoError::SealingError)
        .map(|vec| vec.into_boxed_slice())
}

/// Decrypts a data previously encrypted with [`seal_data`].
///
/// The given `keypair` must correspond to the `peer_id` given during encryption.
pub fn unseal_data(data: &[u8], keypair: &OffchainKeypair) -> crate::errors::Result<Box<[u8]>> {
    let recipient_sk = crypto_box::SecretKey::from(curve25519_dalek::scalar::Scalar::from(keypair));

    recipient_sk
        .unseal(data)
        .map(|vec| vec.into_boxed_slice())
        .map_err(|_| CryptoError::SealingError)
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use hex_literal::hex;

    use super::*;
    use crate::keypairs::Keypair;

    #[test]
    fn seal_unseal_should_be_identity() -> anyhow::Result<()> {
        let data = "some test data".to_string();

        let keypair = OffchainKeypair::random();

        let sealed = seal_data(data.as_bytes(), keypair.public().into())?;

        let unsealed = String::from_utf8(unseal_data(&sealed, &keypair)?.into_vec())?;

        assert_eq!(data, unsealed);

        Ok(())
    }

    #[test]
    fn unseal_should_fail_with_different_private_key() -> anyhow::Result<()> {
        let data = "some test data".to_string();

        let keypair_1 = OffchainKeypair::random();
        let keypair_2 = OffchainKeypair::random();

        let sealed = seal_data(data.as_bytes(), keypair_1.public().into())?;

        assert_eq!(Err(CryptoError::SealingError), unseal_data(&sealed, &keypair_2));

        Ok(())
    }

    #[test]
    fn unseal_should_fail_when_ciphertext_has_been_tampered_with() -> anyhow::Result<()> {
        let data = "some test data".to_string();

        let keypair = OffchainKeypair::random();

        let mut sealed = seal_data(data.as_bytes(), keypair.public().into())?;
        sealed[1] = sealed[1].not();

        assert_eq!(Err(CryptoError::SealingError), unseal_data(&sealed, &keypair));

        Ok(())
    }

    #[test]
    fn unseal_fixed_test() -> anyhow::Result<()> {
        let data = hex!(
            "d7538951e728a28c6381a481f9f33111b6d78211bd1d6a286bdf1b16ee1ad35837b5b0ffcd3b308a4fa9939af0a208150418629c7af31ad457d3fe51602dc9b5f0da253fb44ec0fb75cac9e0bcb9a3ef"
        );
        let peer_id: PeerId = "12D3KooWHcCWDKzMkypyLWvri5ioSVivCazU8jgbWzyerM5aMuf8".parse()?;

        let keypair = OffchainKeypair::from_secret(&hex!(
            "1142b6483e171aa577baea2290797023cd14e034d36f9febb975772ac2924c00"
        ))?;
        assert_eq!(PeerId::from(keypair.public()), peer_id);

        let pt = String::from_utf8(unseal_data(&data, &keypair)?.into_vec())?;

        assert_eq!("Hello, this is a secret message!", pt);

        Ok(())
    }
}
