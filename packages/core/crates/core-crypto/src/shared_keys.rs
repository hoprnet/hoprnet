use std::marker::PhantomData;
use blake2::Blake2s256;
use std::ops::Mul;

use elliptic_curve::rand_core::{CryptoRng, RngCore};
use elliptic_curve::Group;

use k256::{NonZeroScalar, ProjectivePoint};

use crate::errors::CryptoError::{CalculationError, InvalidSecretScalar};
use hkdf::SimpleHkdf;
use libp2p_identity::PeerId;
use rand::rngs::OsRng;
use utils_types::traits::{BinarySerializable, PeerIdLike};

use crate::errors::Result;
use crate::parameters::SECRET_KEY_LENGTH;
use crate::types::{CurvePoint, PublicKey};

pub type SecretKey = [u8; SECRET_KEY_LENGTH];

pub trait Exponent: Mul<Output = Self> + Sized {
    type Repr: From<SecretKey>;
    fn from_repr(repr: Self::Repr) -> Result<Self>;
    fn to_repr(&self) -> Self::Repr;
    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self;
}

pub trait GroupElement<'a, E: Exponent>: Clone {
    type Repr: BinarySerializable<'a> + Clone ;

    fn to_repr(&self) -> Self::Repr;

    fn from_repr(repr: Self::Repr) -> Result<Self>;

    fn from_public_key(public_key: &PublicKey) -> Self;

    fn generate(exponent: &E) -> Self;

    fn mul(&self, exponent: &E) -> Self;

    fn is_valid(&self) -> bool;

    /// Extract a keying material from a group element using HKDF extract
    fn extract_key(&self, salt: &[u8]) -> SecretKey {
        SimpleHkdf::<Blake2s256>::extract(Some(salt), &self.to_repr().to_bytes()).0.into()
    }

    /// Performs KDF expansion from the given EC point using HKDF expand
    fn expand_key(&self, salt: &[u8]) -> SecretKey {
        let mut out = [0u8; SECRET_KEY_LENGTH];
        SimpleHkdf::<Blake2s256>::new(Some(salt), &self.to_repr().to_bytes())
            .expand(b"", &mut out)
            .unwrap(); // Cannot panic, unless the constants are wrong

        out
    }
}

impl Exponent for NonZeroScalar {
    type Repr = [u8; 32];

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        NonZeroScalar::try_from(repr.as_slice()).map_err(|_| InvalidSecretScalar)
    }

    fn to_repr(&self) -> Self::Repr {
        let mut ret = [0u8; 32];
        ret.copy_from_slice(self.to_bytes().as_slice());
        ret
    }

    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self {
        NonZeroScalar::random(rng)
    }
}

impl GroupElement<'_, NonZeroScalar> for ProjectivePoint {
    type Repr = CurvePoint;

    fn to_repr(&self) -> Self::Repr {
        CurvePoint::from_affine(self.to_affine())
    }

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        Ok(repr.to_projective_point())
    }

    fn from_public_key(public_key: &PublicKey) -> Self {
        CurvePoint::from(public_key).to_projective_point()
    }

    fn generate(exponent: &NonZeroScalar) -> Self {
        ProjectivePoint::GENERATOR * exponent.as_ref()
    }

    fn mul(&self, exponent: &NonZeroScalar) -> Self {
        Mul::mul(self, exponent.as_ref())
    }

    fn is_valid(&self) -> bool {
        self.is_identity().unwrap_u8() == 0
    }
}

/// Structure containing shared keys for peers.
/// The members are exposed only using specialized methods.
pub struct SharedKeys<'a, E: Exponent, G: GroupElement<'a, E>> {
    pub alpha: G::Repr,
    pub secrets: Vec<SecretKey>,
    exponent_type: PhantomData<E>
}

impl <'a, E: Exponent, G: GroupElement<'a, E>> SharedKeys<'a, E, G> {
    /// Generates shared secrets for the given path of peers.
    pub fn new(path: &[&PeerId]) -> Result<Self> {
        Self::generate(
            &mut OsRng,
            &path
                .iter()
                .map(|peer_id| PublicKey::from_peerid(peer_id))
                .collect::<utils_types::errors::Result<Vec<_>>>()?,
        )
    }

    /// Generates shared secrets given the peer public keys array.
    /// The order of the peer public keys is preserved for resulting shared keys.
    /// The specified random number generator will be used.
    pub fn generate(rng: &mut (impl CryptoRng + RngCore), peer_public_keys: &[PublicKey]) -> Result<SharedKeys<'a, E, G>> {
        let mut shared_keys = Vec::new();

        // This becomes: x * b_0 * b_1 * b_2 * ...
        let mut coeff_prev = E::random(rng);

        // This becomes: x * b_0 * b_1 * b_2 * ... * G
        // We remain in projective coordinates to save some cycles
        let mut alpha_prev = G::generate(&coeff_prev);
        let alpha = alpha_prev.to_repr();

        // Iterate through all the given peer public keys
        for (i, group_elem) in peer_public_keys.iter().map(G::from_public_key).enumerate() {
            // Try to decode the given public key point & multiply by the current coefficient
            let shared_secret = group_elem.mul(&coeff_prev);

            // Extract the shared secret from the computed EC point and copy it into the shared keys structure
            shared_keys.push(shared_secret.extract_key(&group_elem.to_repr().to_bytes()));

            // Stop here, we don't need to compute anything more
            if i == peer_public_keys.len() - 1 {
                break;
            }

            // Compute the new blinding factor b_k (alpha needs compressing first)
            let b_k = shared_secret.expand_key(&alpha_prev.to_repr().to_bytes());
            let b_k_checked = E::from_repr(b_k.into())?;

            // Update coeff prev and alpha
            alpha_prev = alpha_prev.mul(&b_k_checked);
            coeff_prev = coeff_prev.mul(b_k_checked);

            if !alpha_prev.is_valid() {
                return Err(CalculationError);
            }
        }

        Ok(SharedKeys {
            alpha,
            secrets: shared_keys,
            exponent_type: PhantomData,
        })
    }

    /// Calculates the forward transformation for the given peer public key.
    pub fn forward_transform(alpha: G::Repr, private_key: E::Repr) -> Result<(G::Repr, SecretKey)> {
        let private_scalar = E::from_repr(private_key)?;
        let public_key = G::generate(&private_scalar);
        let alpha_point = G::from_repr(alpha)?;

        let s_k = alpha_point.mul(&private_scalar);
        let secret = s_k.extract_key(&public_key.to_repr().to_bytes());

        let b_k = s_k.expand_key(&alpha_point.to_repr().to_bytes());
        let b_k_checked = E::from_repr(b_k.into())?;

        let alpha_new = alpha_point.mul(&b_k_checked);

        Ok((alpha_new.to_repr(), secret))
    }

}

pub type Secp256k1SharedKeys<'a> = SharedKeys<'a, NonZeroScalar, ProjectivePoint>;

/// Unit tests of the Rust code
#[cfg(test)]
pub mod tests {
    use super::*;
    use elliptic_curve::rand_core::OsRng;
    use hex_literal::hex;

    #[test]
    fn test_extract_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = ProjectivePoint::GENERATOR;

        let key = pt.extract_key(&salt);
        assert_eq!(SECRET_KEY_LENGTH, key.len());

        let res = hex!("9e286c3d45fbbe22e570d96ffe83987960a75b4dbe3d3a74b52c96125323ee9a");
        let rs = hex::encode(key.as_slice());
        assert_eq!(res, key.as_ref());
    }

    #[test]
    fn test_expand_key_from_group_element() {
        let salt = [0xde, 0xad, 0xbe, 0xef];
        let pt = ProjectivePoint::GENERATOR;

        let key = pt.expand_key(&salt);
        assert_eq!(SECRET_KEY_LENGTH, key.len());

        let res = hex!("cc3caa95b5caa141bbdecfbcc18e7f9ae9492ba02b4a5b5d3261f14157b125ec");
        let rs = hex::encode(key.as_slice());
        assert_eq!(res, key.as_ref());
    }

    pub fn generate_random_keypairs<'a, E: Exponent, G: GroupElement<'a, E>>(count: usize) -> (Vec<E::Repr>, Vec<PublicKey>) {
        (0..count)
            .map(|_| E::random(&mut OsRng))
            .map(|s|
                (
                    s.to_repr(),
                    PublicKey::from_bytes(&G::generate(&s).to_repr().to_bytes()).unwrap()
                )
            )
            .unzip()
    }

    fn generic_test_shared_keys<'a, E: Exponent, G: GroupElement<'a, E>>() {
        // DummyRng is useful for deterministic debugging
        //let mut used_rng = crate::dummy_rng::DummyFixedRng::new();

        const COUNT_KEYPAIRS: usize = 3;
        let mut used_rng = OsRng;

        // Generate some random key pairs
        let (priv_keys, pub_keys) = generate_random_keypairs::<'a, E, G>(COUNT_KEYPAIRS);

        // Now generate the key shares for the public keys
        let generated_shares = SharedKeys::<'a, E, G>::generate(&mut used_rng, &pub_keys).unwrap();

        let mut alpha_cpy = generated_shares.alpha.clone();
        for (i, priv_key) in priv_keys.into_iter().enumerate() {
            let (alpha, secret) = SharedKeys::<'a, E, G>::forward_transform(alpha_cpy, priv_key).unwrap();

            assert_eq!(secret.as_ref(), generated_shares.secrets[i]);

            alpha_cpy = alpha;
        }
    }

    #[test]
    fn test_secp256k1_shared_keys() {
        generic_test_shared_keys::<'_, NonZeroScalar, ProjectivePoint>()
    }

    #[test]
    fn test_key_shares() {
        let pub_keys = [
            hex!("0253f6e72ad23de294466b830619448d6d9059a42050141cd83bac4e3ee82c3f1e"),
            hex!("035fc5660f59059c263d3946d7abaf33fa88181e27bf298fcc5a9fa493bec9110b"),
            hex!("038d2b50a77fd43eeae9b37856358c7f1aee773b3e3c9d26f30b8706c02cbbfbb6"),
        ]
        .into_iter()
        .map(|p| PublicKey::from_bytes(&p))
        .collect::<utils_types::errors::Result<Vec<_>>>()
        .unwrap();

        let keyshares = Secp256k1SharedKeys::generate(&mut OsRng, &pub_keys).unwrap();
        assert_eq!(3, keyshares.secrets.len());
    }
}


