use std::sync::OnceLock;

use hopr_crypto_random::random_bytes;
use hopr_primitive_types::prelude::*;
use k256::elliptic_curve::hash2curve::{ExpandMsgXmd, GroupDigest};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::elliptic_curve::ProjectivePoint;
use k256::{Scalar, Secp256k1};
use serde::{Deserialize, Serialize};

use crate::errors::CryptoError::CalculationError;
use crate::keypairs::{ChainKeypair, Keypair};
use crate::types::CurvePoint;
use crate::utils::k256_scalar_from_bytes;

/// Bundles values given to the smart contract to prove that a ticket is a win.
///
/// The VRF is thereby needed because it generates on-demand deterministic
/// entropy that can only be derived by the ticket redeemer.
#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct VrfParameters {
    /// the pseudo-random point
    #[serde(with = "serde_bytes")]
    pub v: [u8; CurvePoint::SIZE_COMPRESSED],
    pub h: Scalar,
    pub s: Scalar,
    #[serde(skip)]
    v_decompressed: OnceLock<CurvePoint>,
}

impl PartialEq for VrfParameters {
    fn eq(&self, other: &Self) -> bool {
        // Exclude cached properties
        self.v == other.v && self.h == other.h && self.s == other.s
    }
}

impl Default for VrfParameters {
    fn default() -> Self {
        Self {
            v: [0u8; CurvePoint::SIZE_COMPRESSED],
            h: Scalar::default(),
            s: Scalar::default(),
            v_decompressed: OnceLock::new(),
        }
    }
}

impl std::fmt::Debug for VrfParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VrfParameters")
            .field("V", &format!("({}", hex::encode(self.v),))
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
                v,
                h: k256_scalar_from_bytes(&data[CurvePoint::SIZE_COMPRESSED..CurvePoint::SIZE_COMPRESSED + 32])
                    .unwrap(),
                s: k256_scalar_from_bytes(
                    &data[CurvePoint::SIZE_COMPRESSED + 32..CurvePoint::SIZE_COMPRESSED + 32 + 32],
                )
                .unwrap(),
                v_decompressed: OnceLock::new(),
            })
        } else {
            Err(GeneralError::ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.v);
        ret.extend_from_slice(&self.h.to_bytes());
        ret.extend_from_slice(&self.s.to_bytes());
        ret.into_boxed_slice()
    }
}

impl VrfParameters {
    /// Verifies that VRF values are valid
    pub fn verify<const T: usize>(&self, creator: &Address, msg: &[u8; T], dst: &[u8]) -> crate::errors::Result<()> {
        let cap_b = self.get_encoded_payload(creator, msg, dst);

        let v = self.get_decompressed_v()?; // decompresses the point

        let r_v: ProjectivePoint<Secp256k1> = cap_b * self.s - v.to_projective_point() * self.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
            &[
                &creator.to_bytes(),
                &v.to_bytes()[1..],
                &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
                msg,
            ],
            &[dst],
        )
        .unwrap();

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
        (self.get_decompressed_v().unwrap().affine * self.h)
            .to_affine()
            .to_encoded_point(false)
    }

    /// Performs a scalar point multiplication with the encoded payload
    /// and `self.s`. Expands the payload and applies the hash_to_curve
    /// algorithm.
    ///
    /// Used to create the witness values needed by the smart contract.
    pub fn get_s_b_witness<const T: usize>(&self, creator: &Address, msg: &[u8; T], dst: &[u8]) -> k256::EncodedPoint {
        (self.get_encoded_payload(creator, msg, dst) * self.s)
            .to_affine()
            .to_encoded_point(false)
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
    ) -> k256::ProjectivePoint {
        Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&creator.to_bytes(), msg], &[dst]).unwrap()
    }

    /// Returns decompressed `v`.
    pub fn get_decompressed_v(&self) -> crate::errors::Result<CurvePoint> {
        // OnceLock::get_try_or_init is a perfect fit, but unstable

        if let Some(cached_v) = self.v_decompressed.get() {
            Ok(*cached_v)
        } else {
            let v_decompressed = CurvePoint::from_bytes(&self.v)?;
            Ok(*self.v_decompressed.get_or_init(|| v_decompressed))
        }
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
    let b = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&chain_addr.to_bytes(), msg], &[dst])?;

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
            &chain_addr.to_bytes(),
            &v.to_affine().to_encoded_point(false).as_bytes()[1..],
            &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
            msg,
        ],
        &[dst],
    )?;
    let s = r + h * a;

    // We only store the compressed point
    let mut comp_v = [0u8; CurvePoint::SIZE_COMPRESSED];
    comp_v.copy_from_slice(v.to_encoded_point(true).as_bytes());

    Ok(VrfParameters {
        v: comp_v,
        h,
        s,
        v_decompressed: OnceLock::new(),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::Hash;
    use hex_literal::hex;

    const PRIVATE_KEY: [u8; 32] = hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8");

    #[test]
    fn vrf_values_serialize_deserialize() {
        let keypair = ChainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        let test_msg: [u8; 32] = hex!("8248a966b9215e154c8f673cb154da030916be3fb31af3b1220419a1c98eeaed");

        let vrf_values = derive_vrf_parameters(&test_msg, &keypair, &Hash::default().to_bytes()).unwrap();

        assert_eq!(vrf_values, VrfParameters::from_bytes(&vrf_values.to_bytes()).unwrap());
    }

    #[test]
    fn vrf_values_crypto() {
        let keypair = ChainKeypair::from_secret(&PRIVATE_KEY).unwrap();

        let test_msg: [u8; 32] = hex!("8248a966b9215e154c8f673cb154da030916be3fb31af3b1220419a1c98eeaed");

        let vrf_values = derive_vrf_parameters(&test_msg, &keypair, &Hash::default().to_bytes()).unwrap();

        assert!(vrf_values
            .verify(&keypair.public().to_address(), &test_msg, &Hash::default().to_bytes())
            .is_ok());
    }
}
