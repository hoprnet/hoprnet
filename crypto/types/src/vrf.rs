use hopr_crypto_random::random_bytes;
use hopr_primitive_types::prelude::*;
use k256::{
    AffinePoint, Scalar, Secp256k1,
    elliptic_curve::{
        ProjectivePoint,
        hash2curve::{ExpandMsgXmd, GroupDigest},
        sec1::ToEncodedPoint,
    },
};

use crate::{
    errors::{CryptoError::CalculationError, Result},
    keypairs::{ChainKeypair, Keypair},
    types::{PublicKey, affine_point_from_bytes},
    utils::k256_scalar_from_bytes,
};

/// Bundles values given to the smart contract to prove that a ticket is a win.
///
/// The VRF is thereby needed because it generates on-demand deterministic
/// entropy that can only be derived by the ticket redeemer.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Default)]
pub struct VrfParameters {
    /// the pseudo-random point
    pub V: AffinePoint,
    pub h: Scalar,
    pub s: Scalar,
}

#[cfg(feature = "serde")]
impl serde::Serialize for VrfParameters {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v: [u8; Self::SIZE] = (*self).into();
        serializer.serialize_bytes(v.as_ref())
    }
}

#[cfg(feature = "serde")]
mod de {
    use serde::de;

    use super::*;

    pub(super) struct VrfParametersVisitor {}

    impl de::Visitor<'_> for VrfParametersVisitor {
        type Value = VrfParameters;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_fmt(format_args!("a byte-array with {} elements", VrfParameters::SIZE))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
        where
            E: de::Error,
        {
            VrfParameters::try_from(v).map_err(|e| de::Error::custom(e.to_string()))
        }
    }
}

// Use compact deserialization for tickets as they are used very often
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for VrfParameters {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(de::VrfParametersVisitor {})
    }
}

impl std::fmt::Debug for VrfParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VrfParameters")
            .field("V", &hex::encode(self.V.to_encoded_point(true)))
            .field("h", &hex::encode(self.h.to_bytes()))
            .field("s", &hex::encode(self.s.to_bytes()))
            .finish()
    }
}

impl From<VrfParameters> for [u8; VRF_PARAMETERS_SIZE] {
    fn from(value: VrfParameters) -> Self {
        let mut ret = [0u8; VRF_PARAMETERS_SIZE];
        ret[0..PublicKey::SIZE_COMPRESSED].copy_from_slice(value.V.to_encoded_point(true).as_bytes());
        ret[PublicKey::SIZE_COMPRESSED..PublicKey::SIZE_COMPRESSED + 32].copy_from_slice(value.h.to_bytes().as_ref());
        ret[PublicKey::SIZE_COMPRESSED + 32..PublicKey::SIZE_COMPRESSED + 64]
            .copy_from_slice(value.s.to_bytes().as_ref());
        ret
    }
}

impl TryFrom<&[u8]> for VrfParameters {
    type Error = GeneralError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            let mut v = [0u8; PublicKey::SIZE_COMPRESSED];
            v.copy_from_slice(&value[..PublicKey::SIZE_COMPRESSED]);
            Ok(VrfParameters {
                V: affine_point_from_bytes(&value[..PublicKey::SIZE_COMPRESSED])
                    .map_err(|_| GeneralError::ParseError("VrfParameters.V".into()))?,
                h: k256_scalar_from_bytes(&value[PublicKey::SIZE_COMPRESSED..PublicKey::SIZE_COMPRESSED + 32])
                    .map_err(|_| GeneralError::ParseError("VrfParameters.h".into()))?,
                s: k256_scalar_from_bytes(
                    &value[PublicKey::SIZE_COMPRESSED + 32..PublicKey::SIZE_COMPRESSED + 32 + 32],
                )
                .map_err(|_| GeneralError::ParseError("VrfParameters.s".into()))?,
            })
        } else {
            Err(GeneralError::ParseError("VrfParameters.size".into()))
        }
    }
}

const VRF_PARAMETERS_SIZE: usize = PublicKey::SIZE_COMPRESSED + 32 + 32;
impl BytesEncodable<VRF_PARAMETERS_SIZE> for VrfParameters {}

impl VrfParameters {
    /// Verifies that VRF values are valid.
    /// The SC performs the verification. This method is here just to test correctness.
    #[allow(non_snake_case)]
    pub fn verify<const T: usize>(&self, creator: &Address, msg: &[u8; T], dst: &[u8]) -> Result<()> {
        let cap_B = self.get_encoded_payload(creator, msg, dst)?;
        let v_proj = ProjectivePoint::<Secp256k1>::from(self.V);

        let R_v: ProjectivePoint<Secp256k1> = cap_B * self.s - v_proj * self.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
            &[
                creator.as_ref(),
                &self.V.to_encoded_point(false).as_bytes()[1..],
                &R_v.to_affine().to_encoded_point(false).as_bytes()[1..],
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

    /// Returns the encoded VRF `V` value as an uncompressed point in affine coordinates.
    pub fn get_v_encoded_point(&self) -> k256::EncodedPoint {
        self.V.to_encoded_point(false)
    }

    /// Performs a scalar point multiplication of `self.h` and `self.v`
    /// and returns the point in affine coordinates.
    ///
    /// Used to create the witness values needed by the smart contract.
    pub fn get_h_v_witness(&self) -> k256::EncodedPoint {
        (ProjectivePoint::<Secp256k1>::from(self.V) * self.h)
            .to_affine()
            .to_encoded_point(false)
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
    ) -> Result<k256::EncodedPoint> {
        Ok((self.get_encoded_payload(creator, msg, dst)? * self.s)
            .to_affine()
            .to_encoded_point(false))
    }

    /// Takes the message upon which the VRF gets computed, the domain separation tag
    /// and the Ethereum address of the creator, expand the raw data with the
    /// `ExpandMsgXmd` algorithm (https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-expand_message_xmd)
    /// and applies the hash-to-curve function to it.
    ///
    /// Finally, returns an elliptic curve point on Secp256k1.
    fn get_encoded_payload<const T: usize>(
        &self,
        creator: &Address,
        msg: &[u8; T],
        dst: &[u8],
    ) -> Result<k256::ProjectivePoint> {
        Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[creator.as_ref(), msg], &[dst])
            .or(Err(CalculationError))
    }
}

/// Takes a private key, the corresponding Ethereum address and a payload
/// and creates all parameters that are required by the smart contract
/// to prove that a ticket is a win.
#[cfg(feature = "rust-ecdsa")]
#[allow(non_snake_case)]
pub fn derive_vrf_parameters<T: AsRef<[u8]>>(
    msg: T,
    chain_keypair: &ChainKeypair,
    dst: &[u8],
) -> crate::errors::Result<VrfParameters> {
    let chain_addr = chain_keypair.public().to_address();
    let B = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[chain_addr.as_ref(), msg.as_ref()], &[dst])?;

    let a: Scalar = chain_keypair.into();

    let V = B * a;

    let r = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            &a.to_bytes(),
            &V.to_affine().to_encoded_point(false).as_bytes()[1..],
            &random_bytes::<64>(),
        ],
        &[dst],
    )?;

    let R_v = B * r;

    let h = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            chain_addr.as_ref(),
            &V.to_affine().to_encoded_point(false).as_bytes()[1..],
            &R_v.to_affine().to_encoded_point(false).as_bytes()[1..],
            msg.as_ref(),
        ],
        &[dst],
    )?;
    let s = r + h * a;

    Ok(VrfParameters { V: V.to_affine(), h, s })
}

/// Takes a private key, the corresponding Ethereum address and a payload
/// and creates all parameters that are required by the smart contract
/// to prove that a ticket is a win.
#[cfg(not(feature = "rust-ecdsa"))]
#[allow(non_snake_case)]
pub fn derive_vrf_parameters<T: AsRef<[u8]>>(
    msg: T,
    chain_keypair: &ChainKeypair,
    dst: &[u8],
) -> Result<VrfParameters> {
    let chain_addr = chain_keypair.public().to_address();
    let B = Secp256k1::hash_from_bytes::<ExpandMsgXmd<sha3::Keccak256>>(&[chain_addr.as_ref(), msg.as_ref()], &[dst])?
        .to_affine();

    let a = secp256k1::Scalar::from_be_bytes(chain_keypair.secret().clone().into())
        .map_err(|_| crate::errors::CryptoError::InvalidSecretScalar)?;

    let B_pk = secp256k1::PublicKey::from_byte_array_uncompressed(
        B.to_encoded_point(false)
            .as_bytes()
            .try_into()
            .map_err(|_| crate::errors::CryptoError::InvalidPublicKey)?,
    )
    .map_err(|_| crate::errors::CryptoError::InvalidPublicKey)?;

    let V = B_pk
        .mul_tweak(secp256k1::global::SECP256K1, &a)
        .map_err(|_| CalculationError)?;

    let r = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            &a.to_be_bytes(),
            &V.serialize_uncompressed()[1..],
            &random_bytes::<64>(),
        ],
        &[dst],
    )?;

    let r_scalar = secp256k1::Scalar::from_be_bytes(r.to_bytes().into())
        .map_err(|_| crate::errors::CryptoError::InvalidSecretScalar)?;

    let R_v = B_pk
        .mul_tweak(secp256k1::global::SECP256K1, &r_scalar)
        .map_err(|_| CalculationError)?;

    let h = Secp256k1::hash_to_scalar::<ExpandMsgXmd<sha3::Keccak256>>(
        &[
            chain_addr.as_ref(),
            &V.serialize_uncompressed()[1..],
            &R_v.serialize_uncompressed()[1..],
            msg.as_ref(),
        ],
        &[dst],
    )?;
    let s = r + h * Scalar::from(chain_keypair);

    let V = affine_point_from_bytes(&V.serialize_uncompressed()).map_err(|_| CalculationError)?;

    Ok(VrfParameters { V, h, s })
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use k256::elliptic_curve::ScalarPrimitive;
    use sha3::Keccak256;

    use super::*;
    use crate::types::Hash;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8")).expect("lazy static keypair should be valid");
        static ref ALICE_ADDR: Address = ALICE.public().to_address();

        static ref TEST_MSG: [u8; 32] = hex!("8248a966b9215e154c8f673cb154da030916be3fb31af3b1220419a1c98eeaed");
        static ref ALICE_VRF_OUTPUT: [u8; 97] = hex!("02a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff0652ee2b835e7c475fde5adfedeb7cc31ecdd690f13ac6bb59ed046ca4c189c9996fe60abaad8c93e771c19acfe697e15c1e5ed6a182b2960bf8c7bd687e77a9975");

        static ref WRONG_V_POINT_PREFIX: [u8; 97] = hex!("01a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff0652ee2b835e7c475fde5adfedeb7cc31ecdd690f13ac6bb59ed046ca4c189c9996fe60abaad8c93e771c19acfe697e15c1e5ed6a182b2960bf8c7bd687e77a9975");
        static ref H_NOT_IN_FIELD: [u8; 97] = hex!("02a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff065fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe60abaad8c93e771c19acfe697e15c1e5ed6a182b2960bf8c7bd687e77a9975");
        static ref S_NOT_IN_FIELD: [u8; 97] = hex!("02a4e1fa28e8a40348baf79b576a6e040b370b74893d355cd48fc382d5235ff0652ee2b835e7c475fde5adfedeb7cc31ecdd690f13ac6bb59ed046ca4c189c9996ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn vrf_values_serialize_deserialize() -> anyhow::Result<()> {
        let vrf_values = derive_vrf_parameters(*TEST_MSG, &ALICE, Hash::default().as_ref())?;

        let deserialized = VrfParameters::try_from(ALICE_VRF_OUTPUT.as_ref())?;

        // check for regressions
        assert_eq!(vrf_values.V, deserialized.V);
        assert!(
            deserialized
                .verify(&ALICE_ADDR, &TEST_MSG, Hash::default().as_ref())
                .is_ok()
        );

        // PartialEq is intentionally not implemented for VrfParameters
        let vrf: [u8; VrfParameters::SIZE] = vrf_values.clone().into();
        let other = VrfParameters::try_from(vrf.as_ref())?;
        assert!(vrf_values.s == other.s && vrf_values.V == other.V && vrf_values.h == other.h);

        Ok(())
    }

    #[test]
    fn vrf_values_serialize_deserialize_bad_examples() {
        assert!(VrfParameters::try_from(WRONG_V_POINT_PREFIX.as_ref()).is_err());

        assert!(VrfParameters::try_from(H_NOT_IN_FIELD.as_ref()).is_err());

        assert!(VrfParameters::try_from(S_NOT_IN_FIELD.as_ref()).is_err());
    }

    #[test]
    fn vrf_values_crypto() -> anyhow::Result<()> {
        let vrf_values = derive_vrf_parameters(*TEST_MSG, &ALICE, Hash::default().as_ref())?;

        assert!(
            vrf_values
                .verify(&ALICE_ADDR, &TEST_MSG, Hash::default().as_ref())
                .is_ok()
        );

        Ok(())
    }

    #[test]
    fn test_vrf_parameter_generation() -> anyhow::Result<()> {
        let dst = b"some DST tag";
        let priv_key: [u8; 32] = hex!("f13233ff60e1f618525dac5f7d117bef0bad0eb0b0afb2459f9cbc57a3a987ba"); // dummy
        let message = hex!("f13233ff60e1f618525dac5f7d117bef0bad0eb0b0afb2459f9cbc57a3a987ba"); // dummy

        let keypair = ChainKeypair::from_secret(&priv_key)?;
        // vrf verification algorithm
        let pub_key = PublicKey::from_privkey(&priv_key)?;

        let params = derive_vrf_parameters(message, &keypair, dst)?;

        let cap_b =
            Secp256k1::hash_from_bytes::<ExpandMsgXmd<Keccak256>>(&[pub_key.to_address().as_ref(), &message], &[dst])?;

        assert_eq!(
            params.get_s_b_witness(&keypair.public().to_address(), &message, dst)?,
            (cap_b * params.s).to_encoded_point(false)
        );

        let a: Scalar = ScalarPrimitive::<Secp256k1>::from_slice(&priv_key)?.into();
        assert_eq!(params.get_h_v_witness(), (cap_b * a * params.h).to_encoded_point(false));

        let r_v: ProjectivePoint<Secp256k1> =
            cap_b * params.s - ProjectivePoint::<Secp256k1>::from(params.V) * params.h;

        let h_check = Secp256k1::hash_to_scalar::<ExpandMsgXmd<Keccak256>>(
            &[
                pub_key.to_address().as_ref(),
                &params.V.to_encoded_point(false).as_bytes()[1..],
                &r_v.to_affine().to_encoded_point(false).as_bytes()[1..],
                &message,
            ],
            &[dst],
        )?;

        assert_eq!(h_check, params.h);

        Ok(())
    }
}
