use std::ops::{Add, Mul};

use hopr_types::{
    crypto::{
        crypto_traits::{
            BlockSizeUser, FixedOutput, HashMarker, KeyIvInit, OutputSizeUser, StreamCipher,
            elliptic_curve::ops::Reduce,
            hash2curve::{ExpandMsgXmd, GroupDigest, MapToCurve, hash_to_scalar},
        },
        prelude::Pseudonym,
    },
    primitive::hybrid_array::{
        Array, ArraySize,
        typenum::{IsGreaterOrEqual, IsLess, IsLessOrEqual, NonZero, Prod, True, U2},
    },
};
#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Share, ShareElement, ShareVerifierGroup, ValueGroup,
    elliptic_curve::{Curve, CurveArithmetic, PrimeCurve, PrimeField, consts::U256},
};

pub mod ack_verify;
pub mod errors;
mod generator;
mod reconstructor;
mod traits;
mod types;

pub use generator::{SsaGeneratorConfig, SsaShareGenerator};
pub use reconstructor::{SsaReconstructor, SsaReconstructorConfig};
pub use traits::{EntryShareGenerator, ExitAcknowledgementShareProcessor, ShareResolution};
pub use types::{
    CoefficientIndex, EncryptedPartialSsaShare, GeneratedShare, PartialSsaShare, PolynomialIndex, RawSsaIndex,
    RecoveredSsa, SsaCommitment, SsaCommitmentState, SsaId, SsaIndex, SsaPolyIndexPrefixSize, SsaPolynomialId,
    SsaRecoveryProgress, TaggedEncryptedPartialSsaShare,
};
pub use vsss_rs::elliptic_curve::{
    Group,
    group::{GroupEncoding, cofactor::CofactorGroup},
};

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
}

/// Number of polynomials per SSA.
pub const DEFAULT_POLYS_PER_SSA: u16 = 8192;
/// Minimum number of shares to recover a part of an SSA.
pub const DEFAULT_POLY_THRESHOLD: u16 = 128;

/// Maximum number of polynomials per SSA supported by the [`SsaReconstructor`].
pub const MAX_POLYS_PER_SSA: u16 = 16192;
/// Maximum SSA polynomial threshold supported by the [`SsaReconstructor`].
pub const MAX_POLY_THRESHOLD: u16 = 4096;

/// Specification of the Protocol for Incentivization of eXits (PIX) instantiation.
pub trait PixSpec: Send + Sync + 'static
where
    PixScalar<Self>: PrimeField,
    PixGroup<Self>: Group<Scalar = PixScalar<Self>> + GroupEncoding + Default + CofactorGroup,
    PixGroupRepr<Self>: std::fmt::Debug + PartialEq + Eq,
    <PixDigest<Self> as OutputSizeUser>::OutputSize: IsLess<U256>,
    <PixDigest<Self> as OutputSizeUser>::OutputSize:
        IsLessOrEqual<<PixDigest<Self> as BlockSizeUser>::BlockSize, Output = True>,
    <Self::Curve as Curve>::FieldBytesSize: Add<SsaPolyIndexPrefixSize>,
    <<Self::Curve as Curve>::FieldBytesSize as Add<SsaPolyIndexPrefixSize>>::Output: ArraySize,
    // hash2curve `hash_to_scalar` bounds for `msg_to_scalar`
    <<Self::Curve as MapToCurve>::SecurityLevel as Mul<U2>>::Output: Sized,
    <Self::Curve as MapToCurve>::SecurityLevel: Mul<U2>,
    <PixDigest<Self> as OutputSizeUser>::OutputSize:
        IsGreaterOrEqual<Prod<<Self::Curve as MapToCurve>::SecurityLevel, U2>, Output = True>,
    <Self::Curve as Curve>::FieldBytesSize: NonZero,
    PixScalar<Self>: Reduce<Array<u8, <Self::Curve as Curve>::FieldBytesSize>>,
{
    /// Prime order elliptic curve use for commitments.
    type Curve: PrimeCurve + CurveArithmetic + GroupDigest;
    /// Digest used for hashing operations.
    type Digest: BlockSizeUser + FixedOutput + std::fmt::Debug + Default + HashMarker;
    /// Pseudonym used to identify groups of SURBs.
    type Pseudonym: Pseudonym + std::fmt::Debug + Copy + Send + Sync + 'static;
    /// Stream cipher used to encrypt the SSA shares.
    type Cipher: StreamCipher + KeyIvInit;
    /// Deposit address type.
    type DepositAddress: Copy + for<'a> From<&'a Self::AddressPrivateKey> + Send + Sync + 'static;
    /// Private key type.
    type AddressPrivateKey: Clone + Send + Sync + 'static;

    /// Context data used to derive the SSA encryption key.
    const KEY_DERIVATION_CONTEXT: &'static str = "HASH_SSA_POLY_SHARE";
    /// Domain separator used to derive the X value of a share.
    const HASH_SCALAR_DERIVATION_CONTEXT: &'static str = "HASH_SSA_POLY_SHARE_SCALAR";

    /// Stable, protocol-versioned hash-to-scalar suite identifier used for
    /// domain separation. This must be a fixed string — deriving it dynamically
    /// from Debug output would break wire compatibility when dependency versions
    /// change formatting.
    const HASH_TO_SCALAR_SUITE_ID: &'static [u8];

    /// Performs conversion of the given `spi` and `msg` into [`PixScalar`] of this spec.
    fn msg_to_scalar(
        spi: &SsaPolynomialId<Self::Pseudonym>,
        msg: impl AsRef<[u8]>,
    ) -> errors::Result<PixScalar<Self>, Self::Pseudonym>
    where
        Self: Sized,
    {
        hash_to_scalar::<Self::Curve, ExpandMsgXmd<Self::Digest>, <Self::Curve as Curve>::FieldBytesSize>(
            &[
                msg.as_ref(),
                spi.pseudonym().as_ref(),
                spi.ssa_index().get().to_be_bytes().as_ref(),
                spi.poly_index().to_be_bytes().as_ref(),
            ],
            &[
                Self::HASH_TO_SCALAR_SUITE_ID,
                Self::HASH_SCALAR_DERIVATION_CONTEXT.as_bytes(),
            ],
        )
        .map_err(|_| errors::PixError::InvalidInput)
    }

    /// Converts `PixGroup` to an address that can be deposited to.
    ///
    /// Returns `None` if the conversion is not possible.
    fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress>;
    /// Convert `PixScalar` to a private key of a deposit address.
    ///
    /// Returns `None` if the conversion is not possible.
    fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey>;
}

/// Finite field used to represent the polynomial coefficients.
pub type PixScalar<S> = <<S as PixSpec>::Curve as CurveArithmetic>::Scalar;
/// Elliptic curve point used to represent the polynomial coefficient commitments.
pub type PixGroup<S> = <<S as PixSpec>::Curve as CurveArithmetic>::ProjectivePoint;
/// Serializable representation of the polynomial coefficient commitments.
pub type PixGroupRepr<S> = <PixGroup<S> as GroupEncoding>::Repr; // This internally converts to affine
/// Digest used for hashing operations.
pub type PixDigest<S> = <S as PixSpec>::Digest;

pub(crate) type CompletedShare<S> =
    DefaultShare<IdentifierPrimeField<PixScalar<S>>, IdentifierPrimeField<PixScalar<S>>>;

#[inline]
pub(crate) fn into_completed_share<S: PixSpec>(
    identifier: PixScalar<S>,
    share: &PartialSsaShare<S>,
) -> errors::Result<CompletedShare<S>, S::Pseudonym> {
    Ok(DefaultShare {
        identifier: identifier.into(),
        value: Option::from(PixScalar::<S>::from_repr(share.0))
            .map(|s: PixScalar<S>| s.into())
            .ok_or(vsss_rs::Error::InvalidShare)?,
    })
}

/// Verifier for shares of a polynomial with the given [`SsaPolynomialId`].
///
/// This contains commitments to all coefficients of the polynomial with the given [`SsaPolynomialId`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PartialSsaShareVerifier<S: PixSpec, P = <S as PixSpec>::Pseudonym> {
    pub(crate) spi: SsaPolynomialId<P>,
    pub(crate) poly_commitment: Vec<ShareVerifierGroup<PixGroup<S>>>,
}

impl<S: PixSpec> PartialSsaShareVerifier<S, S::Pseudonym> {
    /// Returns the [`SsaPolynomialId`] of the polynomial corresponding to this verifier.
    #[inline]
    pub fn spi(&self) -> &SsaPolynomialId<S::Pseudonym> {
        &self.spi
    }

    /// Returns the commitment to the constant term of the polynomial.
    #[inline]
    pub fn constant_term(&self) -> &PixGroup<S> {
        // Constant term is the second entry, first is always the generator.
        &self.poly_commitment[1].0
    }

    /// Minimum number of shares required to reconstruct the polynomial corresponding to this verifier.
    #[inline]
    pub fn min_shares(&self) -> usize {
        // For a polynomial of degree t, there has to be t+1 shares to reconstruct it.
        // However, there are t+2 commitments for a polynomial of degree t (t+1 coefficient commitments + 1 generator),
        // so the minimum number of shares required to reconstruct the polynomial is equal
        // to the number of commitments minus 1.
        self.poly_commitment.len() - 1
    }

    /// Converts this verifier into a tuple containing the [`SsaPolynomialId`] and the serialized polynomial
    /// coefficient commitments.
    ///
    /// The generator (first entry) is omitted by position. The remaining entries are all coefficient
    /// commitments regardless of their value, because a coefficient commitment equal to the generator
    /// validly represents scalar coefficient 1.
    pub fn into_serializable_commitments(self) -> (SsaPolynomialId<S::Pseudonym>, Vec<PixGroupRepr<S>>) {
        (
            self.spi,
            self.poly_commitment
                .into_iter()
                .skip(1) // Omit the structural generator at index 0 by position
                .map(|c| c.to_bytes())
                .collect(),
        )
    }

    /// Tries to create a new verifier from [`SsaPolynomialId`] and serialized polynomial coefficient commitments.
    ///
    /// The `poly_commitments` do not need to contain the generator, because it is added automatically.
    /// All received commitments are deserialized without value-based filtering — a coefficient commitment
    /// equal to the generator validly represents scalar coefficient 1 and must be preserved.
    pub fn from_serializable_commitments(
        spi: SsaPolynomialId<S::Pseudonym>,
        poly_commitments: Vec<PixGroupRepr<S>>,
    ) -> errors::Result<Self, S::Pseudonym> {
        let recv_commitments = poly_commitments.into_iter().map(|c| {
            Option::<PixGroup<S>>::from(PixGroup::<S>::from_bytes(&c))
                .filter(|pt| {
                    // Reject points outside the prime-order subgroup. Baby JubJub has
                    // cofactor 8, so small-order points can pass the on-curve check.
                    bool::from(pt.is_torsion_free())
                })
                .map(ShareVerifierGroup::<PixGroup<S>>::from)
                .ok_or(errors::PixError::InvalidInput)
        });

        // Re-add the generator as the first entry
        let poly_commitment = std::iter::once(Ok(ShareVerifierGroup::<PixGroup<S>>::generator()))
            .chain(recv_commitments)
            .collect::<errors::Result<Vec<_>, S::Pseudonym>>()?;
        if poly_commitment.len() < 2 {
            return Err(errors::PixError::InvalidInput);
        }
        Ok(Self { spi, poly_commitment })
    }

    pub(crate) fn verify_completed_share(&self, share: &CompletedShare<S>) -> errors::Result<(), S::Pseudonym> {
        if (share.value().is_zero() | share.identifier().is_zero()).into() {
            return Err(vsss_rs::Error::InvalidShare.into());
        }
        if self.poly_commitment[0].is_zero().into() {
            return Err(vsss_rs::Error::InvalidGenerator("generator is identity").into());
        }

        let mut i = IdentifierPrimeField::<PixScalar<S>>::one();
        let mut scalars = Vec::with_capacity(self.poly_commitment.len() - 2);

        // The below multi-scalar multiplication method (MSM) is more efficient
        // for large polynomial degrees than Horner's method because it can be parallelized.

        // Computes x^1, x^2, x^3, ... x^t
        for _ in 0..self.poly_commitment.len() - 2 {
            *i.as_mut() *= share.identifier().as_ref();
            scalars.push(i);
        }

        #[cfg(feature = "rayon")]
        let scalars_iter = scalars.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let scalars_iter = scalars.into_iter();

        // v[1] + v[2]*x + v[3]*x^2 + ... + v[t]*x^t
        let rhs = self.poly_commitment[1].0
            + scalars_iter
                .enumerate()
                .map(|(i, c)| (self.poly_commitment[i + 2] * c).0)
                .sum::<PixGroup<S>>();

        let rhs = ValueGroup::from(rhs);
        let lhs = self.poly_commitment[0] * share.value();

        let res = rhs - lhs;

        if res.is_zero().into() {
            Ok(())
        } else {
            Err(vsss_rs::Error::InvalidShare.into())
        }
    }

    /// Verifies that the given `share` corresponding to `msg` belongs to the polynomial associated with this verifier.
    #[inline]
    pub fn verify(&self, share: &PartialSsaShare<S>, msg: impl AsRef<[u8]>) -> errors::Result<(), S::Pseudonym> {
        let msg = S::msg_to_scalar(&self.spi, msg)?;
        self.verify_completed_share(&into_completed_share(msg, share)?)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use anyhow::Context;
    use hopr_types::{
        crypto::{
            crypto_traits,
            prelude::{ChainKeypair, Keypair, PublicKey, Secp256k1, SimplePseudonym},
        },
        primitive::prelude::Address,
    };
    use vsss_rs::{
        ParticipantIdGeneratorType,
        elliptic_curve::{Field, rand_core::CryptoRng},
        feldman,
    };

    use super::*;
    use crate::types::SsaId;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, Ord, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct TestSpec;

    impl PixSpec for TestSpec {
        type AddressPrivateKey = ChainKeypair;
        type Cipher = hopr_types::crypto::primitives::ChaCha20;
        type Curve = hopr_types::crypto::primitives::Secp256k1;
        type DepositAddress = Address;
        type Digest = hopr_types::crypto::primitives::Sha3_256;
        type Pseudonym = SimplePseudonym;

        const HASH_TO_SCALAR_SUITE_ID: &'static [u8] = b"Secp256k1_XMD:SHA3-256_SSWU_RO_";

        fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress> {
            PublicKey::try_from(group.to_affine()).ok().map(|pk| pk.to_address())
        }

        fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey> {
            ChainKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
        }
    }

    type Share<S> = DefaultShare<IdentifierPrimeField<PixScalar<S>>, IdentifierPrimeField<PixScalar<S>>>;
    type StandardShamirResult<S> = (Vec<Share<S>>, Vec<ShareVerifierGroup<PixGroup<S>>>);

    fn standard_shamir_generate<S: PixSpec>(
        secret: PixScalar<S>,
        t: usize,
        x: &[PixScalar<S>],
        mut rng: impl CryptoRng,
    ) -> anyhow::Result<StandardShamirResult<S>> {
        anyhow::ensure!(t > 0, "t must be greater than 0");
        anyhow::ensure!(x.len() >= t, "x must have at least t elements");

        let (shares, verifier_set) =
            feldman::split_secret_with_participant_generator::<Share<S>, ShareVerifierGroup<PixGroup<S>>>(
                t,
                x.len(),
                &secret.into(),
                None,
                &mut rng,
                &[ParticipantIdGeneratorType::list(
                    &x.iter().map(|x| (*x).into()).collect::<Vec<_>>(),
                )],
            )
            .map_err(anyhow::Error::msg)?;

        Ok((shares, verifier_set))
    }

    #[test]
    fn ssa_share_verifier_must_correspond_to_standard() -> anyhow::Result<()> {
        let mut rng = rand::rng();
        let secret = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::random(&mut rng);

        let spi = SsaPolynomialId::new(
            SsaId::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1.try_into()?),
            1,
        );
        let x = (0..=20_u32)
            .map(|i| TestSpec::msg_to_scalar(&spi, i.to_be_bytes()).unwrap())
            .collect::<Vec<_>>();

        let (shares, verifier) = standard_shamir_generate::<TestSpec>(secret, 10, &x, &mut rng)?;

        assert_eq!(verifier.len(), 11);

        let verifier: PartialSsaShareVerifier<TestSpec> = PartialSsaShareVerifier {
            spi,
            poly_commitment: verifier,
        };

        assert_eq!(shares.len(), x.len());
        assert_eq!(verifier.min_shares(), 10);
        assert_eq!(verifier.poly_commitment.len() - 1, verifier.min_shares());

        for (i, s) in shares.into_iter().enumerate() {
            let share: PartialSsaShare<TestSpec> = PartialSsaShare(s.value.0.to_repr());
            verifier
                .verify(&share, (i as u32).to_be_bytes())
                .context(format!("Verification failed for share index {i}"))?;
        }

        Ok(())
    }

    #[test]
    fn ssa_share_verifier_must_be_convertible_to_and_from_serializable_commitments() -> anyhow::Result<()> {
        let mut rng = rand::rng();
        let secret = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::random(&mut rng);

        let spi = SsaPolynomialId::new(
            SsaId::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1.try_into()?),
            1,
        );
        let x = (0..=20_u32)
            .map(|i| TestSpec::msg_to_scalar(&spi, i.to_be_bytes()).unwrap())
            .collect::<Vec<_>>();

        let (_, verifier) = standard_shamir_generate::<TestSpec>(secret, 10, &x, &mut rng)?;

        let verifier_1: PartialSsaShareVerifier<TestSpec> = PartialSsaShareVerifier {
            spi,
            poly_commitment: verifier,
        };

        assert!(PartialSsaShareVerifier::<TestSpec>::from_serializable_commitments(spi, vec![]).is_err());

        let (spi, poly_commitments) = verifier_1.clone().into_serializable_commitments();
        let verifier_2: PartialSsaShareVerifier<TestSpec> =
            PartialSsaShareVerifier::from_serializable_commitments(spi, poly_commitments)?;
        assert_eq!(verifier_1, verifier_2);

        Ok(())
    }

    #[test]
    fn from_serializable_commitments_roundtrip_generator_valued_coefficients()
    -> errors::Result<(), <TestSpec as PixSpec>::Pseudonym> {
        let spi = SsaPolynomialId::new(
            SsaId::new(
                SimplePseudonym::try_from([0u8; 10].as_ref()).unwrap(),
                1.try_into().unwrap(),
            ),
            1,
        );
        // A polynomial whose coefficient commitments are all the generator point:
        // these must round-trip correctly since generator-valued coefficients are
        // legitimate (they represent scalar coefficient 1).
        let all_generator = vec![PixGroup::<TestSpec>::generator().to_bytes(); 10];
        let verifier =
            PartialSsaShareVerifier::<TestSpec>::from_serializable_commitments(spi.clone(), all_generator.clone())?;
        let (_, serialized) = verifier.into_serializable_commitments();
        assert_eq!(
            serialized, all_generator,
            "all generator-valued coefficients must round-trip exactly"
        );
        Ok(())
    }
}
