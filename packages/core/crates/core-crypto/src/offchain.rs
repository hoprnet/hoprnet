use std::ops::Mul;
use curve25519_dalek::edwards::CompressedEdwardsY;
use curve25519_dalek::EdwardsPoint;
use curve25519_dalek::traits::IsIdentity;
use libp2p_identity::{PublicKey as lp2p_PublicKey, ed25519::PublicKey as lp2p_k256_PublicKey};
use libp2p_identity::PeerId;
use rand::{CryptoRng, RngCore};
use utils_log::warn;
use utils_types::errors::GeneralError::ParseError;
use utils_types::traits::{BinarySerializable, PeerIdLike};
use crate::errors::CryptoError;
use crate::errors::CryptoError::{CalculationError, InvalidInputValue, Other};
use crate::errors::Result;
use crate::shared_keys::{GroupElement, GroupEncoding, Scalar};
use crate::types::CurvePoint;

pub type EdScalar = curve25519_dalek::Scalar;

impl Scalar for EdScalar {
    type Repr = [u8; 32];

    fn from_repr(repr: Self::Repr) -> Result<Self> {
        Ok(EdScalar::from_bytes_mod_order(repr))
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
        CompressedEdwardsY::from_slice(repr.key.as_bytes())
            .map_err(|_| InvalidInputValue)
            .and_then(|p| p.decompress().ok_or(CalculationError))
    }

    fn generate(scalar: &curve25519_dalek::Scalar) -> Self {
        EdwardsPoint::mul_base(scalar)
    }

    fn multiply(&self, scalar: &curve25519_dalek::Scalar) -> Self {
        self.mul(scalar)
    }

    fn is_valid(&self) -> bool {
        self.is_torsion_free() && !self.is_identity()
    }
}


#[derive(Clone, Debug)]
pub struct OffchainPublicKey {
    key: ed25519_dalek::PublicKey
}

impl PeerIdLike for OffchainPublicKey {
    fn from_peerid(peer_id: &PeerId) -> utils_types::errors::Result<Self> {
        // Workaround for the missing public key API on PeerIds
        let peer_id_str = peer_id.to_base58();
        if peer_id_str.starts_with("12D") {
            // Here we explicitly assume non-RSA PeerId, so that multihash bytes are the actual public key
            let pid = peer_id.to_bytes();
            let (_, mh) = pid.split_at(6);
            Self::from_bytes(mh)
        } else {
            // RSA-based peer ID might never going to be supported by HOPR
            warn!("peer id type not supported: {peer_id_str}");
            Err(ParseError)
        }
    }

    fn to_peerid(&self) -> PeerId {
        PeerId::from_public_key(&lp2p_PublicKey::Ed25519(libp2p_identity::ed25519::PublicKey::decode(self.key.as_bytes()).unwrap()))
    }
}

impl BinarySerializable<'_> for OffchainPublicKey {
    const SIZE: usize = 0;

    fn from_bytes(data: &[u8]) -> utils_types::errors::Result<Self> {
        Ok(OffchainPublicKey {
            key: ed25519_dalek::PublicKey::from_bytes(data.try_into().map_err(|_| ParseError)?).map_err(|_| ParseError)?
        })
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

}


#[cfg(test)]
pub mod tests {
    use curve25519_dalek::EdwardsPoint;
    use utils_types::traits::PeerIdLike;
    use crate::offchain::OffchainPublicKey;
    use crate::shared_keys::tests::generic_test_shared_keys;

    #[test]
    fn test_offchain_pubkey_peerid() {
        let peer_id = "12D3KooWEt8N8XRwrTfHufsUuGpjcVqTiUjSwrXs7Vfx9Rek17qH";
        let pk1 = OffchainPublicKey::from_peerid_str(peer_id).unwrap();
        assert_eq!(peer_id.to_string(), pk1.to_peerid_str());
    }

    #[test]
    fn test_secp256k1_shared_keys() {
        generic_test_shared_keys::<curve25519_dalek::Scalar, EdwardsPoint>()
    }
}

