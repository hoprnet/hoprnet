use generic_array::{ArrayLength, GenericArray};
use hkdf::SimpleHkdf;
use hopr_crypto_types::prelude::*;

// Module-specific constants
const HASH_KEY_HMAC: &str = "HASH_KEY_HMAC";
const HASH_KEY_PACKET_TAG: &str = "HASH_KEY_PACKET_TAG";

/// Helper function to expand an already cryptographically strong key material using the HKDF expand function.
/// The size of the secret must be at least the size of the underlying hash function, which in this
/// case is Blake2s256, meaning the `secret` size must be at least 32 bytes.
fn hkdf_expand_from_prk<L: ArrayLength<u8>>(secret: &SecretKey, tag: &[u8]) -> GenericArray<u8, L> {
    // Create HKDF instance
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret.as_ref()).expect("size of the hkdf secret key is invalid"); // should not happen

    // Expand the key to the required length
    let mut out = GenericArray::default();
    hkdf.expand(tag, &mut out).expect("invalid hkdf output size"); // should not happen

    out
}

/// Derives the packet tag used during packet construction by expanding the given secret.
pub fn derive_packet_tag(secret: &SecretKey) -> PacketTag {
    hkdf_expand_from_prk::<typenum::U16>(secret, HASH_KEY_PACKET_TAG.as_bytes()).into()
}

/// Derives a key for MAC calculation by expanding the given secret.
pub fn derive_mac_key(secret: &SecretKey) -> SecretKey {
    hkdf_expand_from_prk::<typenum::U32>(secret, HASH_KEY_HMAC.as_bytes()).into()
}

/// Internal convenience function to generate key and IV from the given secret.
///
/// The `secret` must be at least 16 bytes long.
/// The function internally uses Blake2s256 based HKDF (see RFC 5869).
///
/// For `extract_with_salt` is given, the HKDF uses `Extract` with the given salt first
/// and then calls `Expand` to derive the key and IV.
/// For otherwise, only `Expand` is used to derive key and IV using the given `info`.
pub(crate) fn generate_key_iv<T: crypto_traits::KeyIvInit, S: AsRef<[u8]>>(
    secret: &S,
    info: &[u8],
    extract_with_salt: Option<&[u8]>,
) -> hopr_crypto_types::errors::Result<T> {
    let key_material = secret.as_ref();
    if key_material.len() < 16 {
        return Err(CryptoError::InvalidInputValue("secret must have at least 128-bits"));
    }

    let hkdf = if extract_with_salt.is_some() {
        SimpleHkdf::<Blake2s256>::new(extract_with_salt, key_material)
    } else {
        SimpleHkdf::<Blake2s256>::from_prk(key_material).map_err(|_| CryptoError::InvalidInputValue("secret"))?
    };

    let mut key = crypto_traits::Key::<T>::default();
    let mut iv = crypto_traits::Iv::<T>::default();

    let mut out = vec![0u8; key.len() + iv.len()];
    hkdf.expand(info, &mut out)
        .map_err(|_| CryptoError::InvalidInputValue("output length"))?;

    let (v_iv, v_key) = out.split_at(iv.len());
    iv.copy_from_slice(v_iv);
    key.copy_from_slice(v_key);

    Ok(T::new(&key, &iv))
}

#[cfg(test)]
mod tests {
    use super::*;
    use elliptic_curve::hash2curve::{ExpandMsgXmd, GroupDigest};
    use elliptic_curve::{sec1::ToEncodedPoint, ProjectivePoint, ScalarPrimitive};
    use hex_literal::hex;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_crypto_types::types::PublicKey;
    use hopr_crypto_types::vrf::derive_vrf_parameters;
    use k256::{Scalar, Secp256k1};

    #[test]
    fn test_derive_packet_tag() {
        let tag = derive_packet_tag(&SecretKey::default());

        let r = hex!("e0cf0fb82ea5a541b0367b376eb36a60");
        assert_eq!(r, tag.as_ref());
    }

    #[test]
    fn test_derive_mac_key() {
        let tag = derive_mac_key(&SecretKey::default());

        let r = hex!("7f656daaf7c2e64bcfc1386f8af273890e863dec63b410967a5652630617b09b");
        assert_eq!(r, tag.as_ref());
    }

    #[test]
    fn test_sample_field_element() {
        let secret = [1u8; SecretKey::LENGTH];
        assert!(sample_secp256k1_field_element(&secret, "TEST_TAG").is_ok());
    }

    #[test]
    fn test_vrf_parameter_generation() -> anyhow::Result<()> {
        let dst = b"some DST tag";
        let priv_key: [u8; 32] = hex!("f13233ff60e1f618525dac5f7d117bef0bad0eb0b0afb2459f9cbc57a3a987ba"); // dummy
        let message = hex!("f13233ff60e1f618525dac5f7d117bef0bad0eb0b0afb2459f9cbc57a3a987ba"); // dummy

        let keypair = ChainKeypair::from_secret(&priv_key)?;
        // vrf verification algorithm
        let pub_key = PublicKey::from_privkey(&priv_key)?;

        let params = derive_vrf_parameters(&message, &keypair, dst)?;

        let cap_b =
            Secp256k1::hash_from_bytes::<ExpandMsgXmd<Keccak256>>(&[&pub_key.to_address().as_ref(), &message], &[dst])?;

        assert_eq!(
            params.get_s_b_witness(&keypair.public().to_address(), &message, dst)?,
            (cap_b * params.s).to_encoded_point(false)
        );

        let a: Scalar = ScalarPrimitive::<Secp256k1>::from_slice(&priv_key)?.into();
        assert_eq!(params.get_h_v_witness(), (cap_b * a * params.h).to_encoded_point(false));

        let r_v: ProjectivePoint<Secp256k1> = cap_b * params.s - params.V.clone().into_projective_point() * params.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<Keccak256>>(
            &[
                &pub_key.to_address().as_ref(),
                &params.V.as_uncompressed().as_bytes()[1..],
                &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
                &message,
            ],
            &[dst],
        )?;

        assert_eq!(h_check, params.h);

        Ok(())
    }
}
