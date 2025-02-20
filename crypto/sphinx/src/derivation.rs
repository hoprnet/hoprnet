use blake2::Blake2s256;
use elliptic_curve::hash2curve::{ExpandMsgXmd, GroupDigest};
use generic_array::{ArrayLength, GenericArray};
use hkdf::SimpleHkdf;
use hopr_crypto_random::random_fill;
use hopr_crypto_types::errors::CryptoError::{CalculationError, InvalidParameterSize};
use hopr_crypto_types::primitives::{DigestLike, SecretKey, SimpleDigest, SimpleMac};
use hopr_crypto_types::types::{HalfKey, PacketTag};
use k256::Secp256k1;

// Module-specific constants
const HASH_KEY_COMMITMENT_SEED: &str = "HASH_KEY_COMMITMENT_SEED";
const HASH_KEY_HMAC: &str = "HASH_KEY_HMAC";
const HASH_KEY_PACKET_TAG: &str = "HASH_KEY_PACKET_TAG";
const HASH_KEY_OWN_KEY: &str = "HASH_KEY_OWN_KEY";
const HASH_KEY_ACK_KEY: &str = "HASH_KEY_ACK_KEY";

/// Size of the nonce in the Ping sub protocol
pub const PING_PONG_NONCE_SIZE: usize = 16;

/// Calculates a message authentication code with fixed key tag (HASH_KEY_HMAC)
/// The given `secret` is first transformed using HKDF before the MAC calculation is performed.
/// Based on `SimpleMac`
pub fn create_tagged_mac(secret: &SecretKey, data: &[u8]) -> [u8; SimpleMac::SIZE] {
    let mut mac = SimpleMac::new(&derive_mac_key(secret));
    mac.update(data);
    mac.finalize().into()
}

/// Helper function to expand an already cryptographically strong key material using the HKDF expand function
/// The size of the secret must be at least the size of the output of the underlying hash function, which in this
/// case is Blake2s256, meaning the `secret` size must be at least 32 bytes.
fn hkdf_expand_from_prk<L: ArrayLength<u8>>(secret: &SecretKey, tag: &[u8]) -> GenericArray<u8, L> {
    // Create HKDF instance
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret.as_ref()).expect("size of the hkdf secret key is invalid"); // should not happen

    // Expand the key to the required length
    let mut out = GenericArray::default();
    hkdf.expand(tag, &mut out).expect("invalid hkdf output size"); // should not happen

    out
}

/// Derives a ping challenge (if no challenge is given) or a pong response to a ping challenge.
pub fn derive_ping_pong(challenge: Option<&[u8]>) -> Box<[u8]> {
    let mut ret = [0u8; PING_PONG_NONCE_SIZE];
    match challenge {
        None => random_fill(&mut ret),
        Some(chal) => {
            let mut digest = SimpleDigest::default();
            digest.update(chal);
            // Finalize requires enough space for the hash value, so this needs an extra copy
            let hash = digest.finalize();
            ret.copy_from_slice(&hash[0..PING_PONG_NONCE_SIZE]);
        }
    }
    ret.into()
}

/// Derives the commitment seed given the compressed private key representation
/// and the serialized channel information.
pub fn derive_commitment_seed(private_key: &[u8], channel_info: &[u8]) -> [u8; SimpleMac::SIZE] {
    let sk: SecretKey = hkdf_expand_from_prk(
        &private_key.try_into().expect("commitment private key size invalid"),
        HASH_KEY_COMMITMENT_SEED.as_bytes(),
    )
    .into();
    let mut mac = SimpleMac::new(&sk);
    mac.update(channel_info);
    mac.finalize().into()
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
/// WARNING: The `iv_first` distinguishes if the IV should be sampled before or after the key is sampled.
pub(crate) fn generate_key_iv(secret: &SecretKey, info: &[u8], key: &mut [u8], iv: &mut [u8], iv_first: bool) {
    let hkdf = SimpleHkdf::<Blake2s256>::from_prk(secret.as_ref()).expect("secret key length must be correct");

    let mut out = vec![0u8; key.len() + iv.len()];
    hkdf.expand(info, &mut out)
        .expect("key and iv are too big for this kdf");

    if iv_first {
        let (v_iv, v_key) = out.split_at(iv.len());
        iv.copy_from_slice(v_iv);
        key.copy_from_slice(v_key);
    } else {
        let (v_key, v_iv) = out.split_at(key.len());
        key.copy_from_slice(v_key);
        iv.copy_from_slice(v_iv);
    }
}

/// Sample a random secp256k1 field element that can represent a valid secp256k1 point.
/// The implementation uses `hash_to_field` function as defined in
/// `<https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-13.html#name-hashing-to-a-finite-field>`
/// The `secret` must be at least `SecretKey::LENGTH` long.
/// The `tag` parameter will be used as an additional Domain Separation Tag.
pub fn sample_secp256k1_field_element(secret: &[u8], tag: &str) -> hopr_crypto_types::errors::Result<HalfKey> {
    if secret.len() >= SecretKey::LENGTH {
        let scalar = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Sha3_256>>(
            &[secret],
            &[b"secp256k1_XMD:SHA3-256_SSWU_RO_", tag.as_bytes()],
        )
        .map_err(|_| CalculationError)?;
        Ok(HalfKey::try_from(scalar.to_bytes().as_ref())?)
    } else {
        Err(InvalidParameterSize {
            name: "secret".into(),
            expected: SecretKey::LENGTH,
        })
    }
}

/// Used in Proof of Relay to derive own half-key (S0)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
pub fn derive_own_key_share(secret: &SecretKey) -> HalfKey {
    sample_secp256k1_field_element(secret.as_ref(), HASH_KEY_OWN_KEY).expect("failed to sample own key share")
}

/// Used in Proof of Relay to derive the half-key of for the acknowledgement (S1)
/// The function samples a secp256k1 field element using the given `secret` via `sample_field_element`.
pub fn derive_ack_key_share(secret: &SecretKey) -> HalfKey {
    sample_secp256k1_field_element(secret.as_ref(), HASH_KEY_ACK_KEY).expect("failed to sample ack key share")
}

#[cfg(test)]
mod tests {
    use super::*;
    use elliptic_curve::{sec1::ToEncodedPoint, ProjectivePoint, ScalarPrimitive};
    use hex_literal::hex;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use hopr_crypto_types::types::PublicKey;
    use hopr_crypto_types::vrf::derive_vrf_parameters;
    use k256::Scalar;

    #[test]
    fn test_derive_commitment_seed() {
        let priv_key = [0u8; SecretKey::LENGTH];
        let chinfo = [0u8; SecretKey::LENGTH];

        let res = derive_commitment_seed(&priv_key, &chinfo);

        let r = hex!("0abe559a1577e99e16f112bb8a88f7793ff1fb22af46b810995fb754ea319386");
        assert_eq!(r, res.as_ref());
    }

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

        let cap_b = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(
            &[&pub_key.to_address().as_ref(), &message],
            &[dst],
        )?;

        assert_eq!(
            params.get_s_b_witness(&keypair.public().to_address(), &message, dst)?,
            (cap_b * params.s).to_encoded_point(false)
        );

        let a: Scalar = ScalarPrimitive::<Secp256k1>::from_slice(&priv_key)?.into();
        assert_eq!(params.get_h_v_witness(), (cap_b * a * params.h).to_encoded_point(false));

        let r_v: ProjectivePoint<Secp256k1> = cap_b * params.s - params.V.clone().into_projective_point() * params.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
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

    #[test]
    fn test_mac() {
        let key = GenericArray::from([1u8; SecretKey::LENGTH]);
        let data = [2u8; 64];
        let mac = create_tagged_mac(&key.into(), &data);

        let expected = hex!("77264e8ea3052b621dbb8b1904403a64b1064c884cf7629c266edd7e237f2799");
        assert_eq!(SimpleMac::SIZE, mac.len());
        assert_eq!(expected, mac);
    }
}
