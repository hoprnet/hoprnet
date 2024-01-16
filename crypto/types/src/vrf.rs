use hopr_crypto_random::random_bytes;
use hopr_primitive_types::errors::GeneralError::ParseError;
use hopr_primitive_types::primitives::Address;
use hopr_primitive_types::traits::BinarySerializable;
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
#[derive(Copy, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct VrfParameters {
    /// the pseudo-random point
    pub v: CurvePoint,
    pub h: Scalar,
    pub s: Scalar,
    /// helper value for smart contract
    pub h_v: CurvePoint,
    /// helper value for smart contract
    pub s_b: CurvePoint,
}

impl std::fmt::Debug for VrfParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v_encoded = self.v.affine.to_encoded_point(false);
        let h_v_encoded = self.h_v.affine.to_encoded_point(false);
        let s_b_encoded = self.s_b.affine.to_encoded_point(false);
        f.debug_struct("VrfParameters")
            .field(
                "V",
                &format!(
                    "({},{})",
                    hex::encode(v_encoded.x().unwrap()),
                    hex::encode(v_encoded.y().unwrap())
                ),
            )
            .field("h", &hex::encode(self.h.to_bytes()))
            .field("s", &hex::encode(self.s.to_bytes()))
            .field(
                "h_v",
                &format!(
                    "({},{})",
                    hex::encode(h_v_encoded.x().unwrap()),
                    hex::encode(h_v_encoded.y().unwrap())
                ),
            )
            .field(
                "s_b",
                &format!(
                    "({},{})",
                    hex::encode(s_b_encoded.x().unwrap()),
                    hex::encode(s_b_encoded.y().unwrap())
                ),
            )
            .finish()
    }
}

impl BinarySerializable for VrfParameters {
    const SIZE: usize = CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE + CurvePoint::SIZE;

    fn from_bytes(data: &[u8]) -> hopr_primitive_types::errors::Result<Self> {
        if data.len() == Self::SIZE {
            Ok(VrfParameters {
                v: CurvePoint::from_bytes(&data[0..CurvePoint::SIZE]).unwrap(),
                h: k256_scalar_from_bytes(&data[CurvePoint::SIZE..CurvePoint::SIZE + 32]).unwrap(),
                s: k256_scalar_from_bytes(&data[CurvePoint::SIZE + 32..CurvePoint::SIZE + 32 + 32]).unwrap(),
                h_v: CurvePoint::from_bytes(
                    &data[CurvePoint::SIZE + 32 + 32..CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE],
                )
                .unwrap(),
                s_b: CurvePoint::from_bytes(
                    &data[CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE
                        ..CurvePoint::SIZE + 32 + 32 + CurvePoint::SIZE + CurvePoint::SIZE],
                )
                .unwrap(),
            })
        } else {
            Err(ParseError)
        }
    }

    fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(&self.v.to_bytes());
        ret.extend_from_slice(&self.h.to_bytes());
        ret.extend_from_slice(&self.s.to_bytes());
        ret.extend_from_slice(&self.h_v.to_bytes());
        ret.extend_from_slice(&self.s_b.to_bytes());
        ret.into_boxed_slice()
    }
}

impl VrfParameters {
    /// Verifies that VRF values are valid
    pub fn verify<const T: usize>(&self, creator: &Address, msg: &[u8; T], dst: &[u8]) -> crate::errors::Result<()> {
        let cap_b =
            Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[&creator.to_bytes(), msg], &[dst]).unwrap();

        // Check point witness
        if self.s_b.to_projective_point() != cap_b * self.s {
            return Err(CalculationError);
        }

        // Check point witness
        if self.h_v.to_projective_point() != self.v.to_projective_point() * self.h {
            return Err(CalculationError);
        }

        let r_v: ProjectivePoint<Secp256k1> = cap_b * self.s - self.v.to_projective_point() * self.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
            &[
                &creator.to_bytes(),
                &self.v.to_bytes()[1..],
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

    Ok(VrfParameters {
        v: v.to_affine().into(),
        h,
        s,
        h_v: (v * h).to_affine().into(),
        s_b: (b * s).to_affine().into(),
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
