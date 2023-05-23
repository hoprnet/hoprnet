use std::ops::Mul;
use curve25519_dalek::edwards::CompressedEdwardsY;
use curve25519_dalek::EdwardsPoint;
use curve25519_dalek::traits::IsIdentity;
use libp2p_identity::{PublicKey as lp2p_PublicKey};
use libp2p_identity::PeerId;
use rand::{CryptoRng, RngCore};
use rand::rngs::OsRng;
use utils_log::warn;
use utils_types::errors::GeneralError::ParseError;
use utils_types::traits::{BinarySerializable, PeerIdLike};
use crate::errors::CryptoError::{CalculationError, InvalidSecretScalar};
use crate::errors::Result;
use crate::shared_keys::{GroupElement, GroupEncoding, Scalar, SharedKeys};
use crate::types::SecretKey;

pub type EdScalar = curve25519_dalek::Scalar;

impl Scalar for EdScalar {
    type Repr = [u8; 32];

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        Ok(EdScalar::from_bits(repr))
    }

    fn to_repr(&self) -> Self::Repr {
        self.to_bytes()
    }

    fn random(rng: &mut (impl CryptoRng + RngCore)) -> Self {
        EdScalar::random(rng)
    }
}

impl GroupElement<EdScalar> for EdwardsPoint {
    type Repr = OffchainPublicKey;

    fn to_repr(&self) -> Self::Repr {
        OffchainPublicKey::from_bytes(self.compress().as_bytes())
                .expect("free-form operation resulted in invalid public key") // must not happen
    }

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        repr.key.decompress().ok_or(CalculationError)
    }

    fn generate(scalar: &curve25519_dalek::Scalar) -> Self {
        EdwardsPoint::mul_base(scalar)
    }

    fn multiply(&self, scalar: &curve25519_dalek::Scalar) -> Self {
        self.mul(scalar)
    }

    fn is_valid(&self) -> bool {
        !self.is_small_order() && !self.is_identity()
    }
}


/// Instantiation of Sphinx shared keys generation using Ed25519 group
pub type Ed25519SharedKeys = SharedKeys<EdScalar, EdwardsPoint>;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct OffchainPublicKey {
    key: CompressedEdwardsY
}

impl PeerIdLike for OffchainPublicKey {
    fn from_peerid(peer_id: &PeerId) -> utils_types::errors::Result<Self> {
        let mh = peer_id.as_ref();
        if mh.code() == 0 {
            Self::from_bytes(&mh.digest()[4..])
        } else {
            warn!("peer id type not supported: {peer_id}");
            Err(ParseError)
        }
    }

    fn to_peerid(&self) -> PeerId {
        let k = libp2p_identity::ed25519::PublicKey::try_from_bytes(self.key.as_bytes()).unwrap();
        PeerId::from_public_key(&lp2p_PublicKey::from(k))
    }
}

impl BinarySerializable<'_> for OffchainPublicKey {
    const SIZE: usize = 32;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        CompressedEdwardsY::from_slice(data)
            .map(|key| Self {key})
            .map_err(|_| ParseError)
    }

    fn to_bytes(&self) -> Box<[u8]> {
        self.key.to_bytes().into()
    }
}

impl GroupEncoding for OffchainPublicKey {
    fn encode(&self) -> Box<[u8]> {
        self.to_bytes()
    }
}

impl OffchainPublicKey {
    pub fn random() -> Self {
        let (pt, _) = EdwardsPoint::random_pair(&mut OsRng);
        Self {
            key: pt.compress()
        }
    }

    pub fn from_privkey(key: SecretKey) -> Result<Self> {
        let scalar = EdScalar::from_bits_clamped(key);
        let point = EdwardsPoint::mul_base(&scalar);
        if !point.is_identity() && !point.is_small_order() {
            Ok(Self{ key: point.compress() })
        } else {
            Err(InvalidSecretScalar)
        }
    }
}


#[cfg(test)]
pub mod tests {
    use curve25519_dalek::EdwardsPoint;
    use crate::shared_keys::tests::generic_test_shared_keys;

    #[test]
    fn test_peerid() {

    }

    #[test]
    fn test_secp256k1_shared_keys() {
        generic_test_shared_keys::<curve25519_dalek::Scalar, EdwardsPoint>()
    }
}

