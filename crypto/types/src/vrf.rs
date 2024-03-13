use hopr_crypto_random::random_bytes;
use hopr_primitive_types::prelude::*;
use k256::elliptic_curve::hash2curve::{ExpandMsgXmd, GroupDigest};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::elliptic_curve::ProjectivePoint;
use k256::{Scalar, Secp256k1};
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};

use crate::errors::{CryptoError::CalculationError, Result};
use crate::keypairs::{ChainKeypair, Keypair};
use crate::types::CurvePoint;
use crate::utils::k256_scalar_from_bytes;

/// Bundles values given to the smart contract to prove that a ticket is a win.
///
/// The VRF is thereby needed because it generates on-demand deterministic
/// entropy that can only be derived by the ticket redeemer.
#[derive(Clone, Copy, Default, Eq)]
pub struct VrfParameters {
    /// the pseudo-random point
    pub v: CurvePoint,
    pub h: Scalar,
    pub s: Scalar,
}

impl Serialize for VrfParameters {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.to_bytes().as_ref())
    }
}

struct VrfParametersVisitor {}

impl<'de> Visitor<'de> for VrfParametersVisitor {
    type Value = VrfParameters;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!("a byte-array with {} elements", VrfParameters::SIZE))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        VrfParameters::from_bytes(v).map_err(|e| de::Error::custom(e.to_string()))
    }
}

// Use compact deserialization for tickets as they are used very often
impl<'de> Deserialize<'de> for VrfParameters {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(VrfParametersVisitor {})
    }
}

impl PartialEq for VrfParameters {
    fn eq(&self, other: &Self) -> bool {
        // Exclude cached properties
        self.v == other.v && self.h == other.h && self.s == other.s
    }
}

impl std::fmt::Debug for VrfParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VrfParameters")
            .field("V", &hex::encode(self.v.serialize_compressed()))
            .field("h", &hex::encode(self.h.to_bytes()))
            .field("s", &hex::encode(self.s.to_bytes()))
            .finish()
    }
}

impl BinarySerializable for VrfParameters {
    const SIZE: usize = CurvePoint::SIZE_COMPRESSED + 32 + 32;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            let mut v = [0u8; CurvePoint::SIZE_COMPRESSED];
            v.copy_from_slice(&data[..CurvePoint::SIZE_COMPRESSED]);
            Ok(VrfParameters {
                v: CurvePoint::from_bytes(&data[..CurvePoint::SIZE_COMPRESSED])?,
                h: k256_scalar_from_bytes(&data[CurvePoint::SIZE_COMPRESSED..CurvePoint::SIZE_COMPRESSED + 32])
                    .map_err(|_| GeneralError::ParseError)?,
                s: k256_scalar_from_bytes(
                    &data[CurvePoint::SIZE_COMPRESSED + 32..CurvePoint::SIZE_COMPRESSED + 32 + 32],
                )
                .map_err(|_| GeneralError::ParseError)?,
            })
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(self.v.serialize_compressed().as_bytes());
        ret.extend_from_slice(&self.h.to_bytes());
        ret.extend_from_slice(&self.s.to_bytes());
        ret.into_boxed_slice()
    }
}

impl VrfParameters {
    /// Verifies that VRF values are valid
    pub fn verify<const T: usize>(&self, creator: &Address, msg: &[u8; T], dst: &[u8]) -> Result<()> {
        let cap_b = self.get_encoded_payload(creator, msg, dst)?;

        let r_v: ProjectivePoint<Secp256k1> = cap_b * self.s - self.v.to_projective_point() * self.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
            &[
                &creator.as_ref(),
                &self.v.serialize_uncompressed().as_bytes()[1..],
                &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
                msg,
            ],
            &[dst],
        )
        .or(Err(CalculationError))?;

        if h_check != self.h {
            return Err(CalculationError);
        }

        Ok(())
    }

    /// Performs a scalar point multiplication of `self.h` and `self.v`
    /// and returns the point in affine coordinates.
    ///
    /// Used to create the witness values needed by the smart contract.
    pub fn get_h_v_witness(&self) -> k256::EncodedPoint {
        (self.v.affine * self.h).to_affine().to_encoded_point(false)
    }

    /// Performs a scalar point multiplication with the encoded payload
    /// and `self.s`. Expands the payload and applies the hash_to_curve
    /// algorithm.
    ///
    /// Used to create the witness values needed by the smart contract.
    pub fn get_s_b_witness<const T: usize>(
        &self,
        creator: &Address,
        msg: &[u8; T],
        dst: &[u8],
    ) -> crate::errors::Result<k256::EncodedPoint> {
        Ok((self.get_encoded_payload(creator, msg, dst)? * self.s)
            .to_affine()
            .to_encoded_point(false))
    }

    /// Takes the message upon which the VRF gets computed, the domain separation tag
    /// and the Ethereum address of the creator, expand the raw data with the
    /// `ExpandMsgXmd` algorithm (https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-expand_message_xmd)
    /// and applies the hash-to-curve function to it.
    ///
    /// Finally, returns a elliptic curve point on Secp256k1.
    fn get_encoded_payload<const T: usize>(
        &self,
        creator: &Address,
        msg: &[u8; T],
        dst: &[u8],
    ) -> crate::errors::Result<k256::ProjectivePoint> {
        Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&creator.as_ref(), msg], &[dst])
            .or(Err(CalculationError))
    }
}

/// Takes a private key, the corresponding Ethereum address and a payload
/// and creates all parameters that are required by the smart contract
/// to prove that a ticket is a win.
pub fn derive_vrf_parameters<const T: usize>(
    msg: &[u8; T],
    chain_keypair: &ChainKeypair,
    dst: &[u8],
) -> crate::errors::Result<VrfParameters> {
    let chain_addr = chain_keypair.public().to_address();
    let b = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&chain_addr.as_ref(), msg], &[dst])?;

    let a: Scalar = chain_keypair.into();

    if a.is_zero().into() {
        return Err(crate::errors::CryptoError::InvalidSecretScalar);
    }

    let v = b * a;

    let r = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            &a.to_bytes(),
            &v.to_affine().to_encoded_point(false).as_bytes()[1..],
            &random_bytes::<64>(),
        ],
        &[dst],
    )?;

    let r_v = b * r;

    let h = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            &chain_addr.as_ref(),
            &v.to_affine().to_encoded_point(false).as_bytes()[1..],
            &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
            msg,
        ],
        &[dst],
    )?;
    let s = r + h * a;

    Ok(VrfParameters {
        v: v.to_affine().into(),
        h,
        s,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::Hash;
    use hex_literal::hex;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8")).unwrap();
        static ref ALICE_ADDR: Address = ALICE.public().to_address();

        static ref TEST_MSG: [u8; 32] = hex!("8248a966b9215e154c8f673cb154da030916be3fb31af3b1220419a1c98eeaed");
        static ref ALICE_VRF_OUTPUT: [u8; 97] = hex!("02a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff0652ee2b835e7c475fde5adfedeb7cc31ecdd690f13ac6bb59ed046ca4c189c9996fe60abaad8c93e771c19acfe697e15c1e5ed6a182b2960bf8c7bd687e77a9975");

        static ref WRONG_V_POINT_PREFIX: [u8; 97] = hex!("01a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff0652ee2b835e7c475fde5adfedeb7cc31ecdd690f13ac6bb59ed046ca4c189c9996fe60abaad8c93e771c19acfe697e15c1e5ed6a182b2960bf8c7bd687e77a9975");
        static ref H_NOT_IN_FIELD: [u8; 97] = hex!("02a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff065fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe60abaad8c93e771c19acfe697e15c1e5ed6a182b2960bf8c7bd687e77a9975");
        static ref S_NOT_IN_FIELD: [u8; 97] = hex!("02a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff0652ee2b835e7c475fde5adfedeb7cc31ecdd690f13ac6bb59ed046ca4c189c9996ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn vrf_values_serialize_deserialize() {
        let vrf_values = derive_vrf_parameters(&*TEST_MSG, &*ALICE, &Hash::default().to_bytes()).unwrap();

        let deserialized = VrfParameters::from_bytes(&*ALICE_VRF_OUTPUT).unwrap();

        // check for regressions
        assert_eq!(vrf_values.v, deserialized.v);
        assert!(deserialized
            .verify(&*ALICE_ADDR, &*TEST_MSG, &Hash::default().to_bytes())
            .is_ok());

        assert_eq!(vrf_values, VrfParameters::from_bytes(&vrf_values.to_bytes()).unwrap());
    }

    #[test]
    fn vrf_values_serialize_deserialize_bad_examples() {
        assert!(VrfParameters::from_bytes(&*WRONG_V_POINT_PREFIX).is_err());

        assert!(VrfParameters::from_bytes(&*H_NOT_IN_FIELD).is_err());

        assert!(VrfParameters::from_bytes(&*S_NOT_IN_FIELD).is_err());
    }

    #[test]
    fn vrf_values_crypto() {
        let vrf_values = derive_vrf_parameters(&*TEST_MSG, &*ALICE, &Hash::default().to_bytes()).unwrap();

        assert!(vrf_values
            .verify(&ALICE_ADDR, &*TEST_MSG, &Hash::default().to_bytes())
            .is_ok());
    }
}
