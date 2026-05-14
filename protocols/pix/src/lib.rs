use std::ops::Add;

use hopr_types::crypto::{
    crypto_traits::{BlockSizeUser, FixedOutput, HashMarker, KeyIvInit, OutputSizeUser, StreamCipher},
    prelude::Pseudonym,
};
#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;
use vsss_rs::{
    DefaultShare, IdentifierPrimeField, Share, ShareElement, ShareVerifierGroup, ValueGroup,
    elliptic_curve::{
        Curve, CurveArithmetic, Group, PrimeCurve, PrimeField,
        consts::U256,
        generic_array::{
            ArrayLength,
            typenum::{IsLess, IsLessOrEqual, U4},
        },
        group::{GroupEncoding, cofactor::CofactorGroup},
        hash2curve::{ExpandMsgXmd, FromOkm, GroupDigest},
    },
};

pub mod errors;
mod generator;
mod reconstructor;
mod types;
mod traits;

pub use generator::{SsaGeneratorConfig, SsaShareGenerator, transpose_commitments};
pub use reconstructor::{ReconstructorEvent, SsaReconstructor, SsaReconstructorConfig};
pub use types::{
    CoefficientIndex, EncryptedPartialSsaShare, PartialSsaShare, PolynomialIndex, SsaId, SsaIndex, SsaPolynomialId,
};
pub use traits::ExitAcknowledgementShareProcessor;

/// Number of polynomials per SSA.
pub const DEFAULT_POLYS_PER_SSA: usize = 1000;
/// Minimum number of shares to recover a part of an SSA.
pub const DEFAULT_POLY_THRESHOLD: usize = 100;

/// Specification of the Protocol for Incentivization of eXits (PIX) instantiation.
pub trait PixSpec
where
    PixScalar<Self>: PrimeField + FromOkm + Send,
    PixGroup<Self>: Group<Scalar = PixScalar<Self>> + GroupEncoding + Default + CofactorGroup,
    PixGroupRepr<Self>: std::fmt::Debug + PartialEq + Eq,
    <PixDigest<Self> as OutputSizeUser>::OutputSize: IsLess<U256>,
    <PixDigest<Self> as OutputSizeUser>::OutputSize: IsLessOrEqual<<PixDigest<Self> as BlockSizeUser>::BlockSize>,
    <Self::Curve as Curve>::FieldBytesSize: Add<U4>,
    <<Self::Curve as Curve>::FieldBytesSize as Add<U4>>::Output: ArrayLength<u8>,
{
    /// Prime order elliptic curve use for commitments.
    type Curve: PrimeCurve + CurveArithmetic + GroupDigest;
    /// Digest used for hashing operations.
    type Digest: BlockSizeUser + FixedOutput + std::fmt::Debug + Default + HashMarker;
    /// Pseudonym used to identify groups of SURBs.
    type Pseudonym: Pseudonym + Copy + Send + Sync + 'static;
    /// Stream cipher used to encrypt the SSA shares.
    type Cipher: StreamCipher + KeyIvInit;
    /// Context data used to derive the SSA encryption key.
    const KEY_DERIVATION_CONTEXT: &str = "HASH_SSA_POLY_SHARE";
    /// Domain separator used to derive the X value of a share.
    const HASH_SCALAR_DERIVATION_CONTEXT: &str = "HASH_SSA_POLY_SHARE_SCALAR";
}

/// Finite field used to represent the polynomial coefficients.
pub type PixScalar<S> = <<S as PixSpec>::Curve as CurveArithmetic>::Scalar;
/// Elliptic curve point used to represent the polynomial coefficient commitments.
pub type PixGroup<S> = <<S as PixSpec>::Curve as CurveArithmetic>::ProjectivePoint;
/// Serializable representation of the polynomial coefficient commitments.
pub type PixGroupRepr<S> = <PixGroup<S> as GroupEncoding>::Repr; // This internally converts to affine
/// Digest used for hashing operations.
pub type PixDigest<S> = <S as PixSpec>::Digest;

pub(crate) fn msg_to_scalar<S: PixSpec>(
    spi: &SsaPolynomialId<S>,
    msg: impl AsRef<[u8]>,
) -> errors::Result<PixScalar<S>> {
    Ok(<S::Curve as GroupDigest>::hash_to_scalar::<ExpandMsgXmd<S::Digest>>(
        &[
            msg.as_ref(),
            spi.pseudonym().as_ref(),
            spi.ssa_index().to_be_bytes().as_ref(),
            spi.poly_index().to_be_bytes().as_ref(),
        ],
        &[
            format!("{:?}_XMD:{:?}_SSWU_RO_", S::Curve::default(), S::Digest::default()).as_bytes(),
            S::HASH_SCALAR_DERIVATION_CONTEXT.as_bytes(),
        ],
    )?)
}

pub(crate) type CompletedShare<S> =
    DefaultShare<IdentifierPrimeField<PixScalar<S>>, IdentifierPrimeField<PixScalar<S>>>;

#[inline]
pub(crate) fn into_completed_share<S: PixSpec>(
    spi: SsaPolynomialId<S>,
    msg: impl AsRef<[u8]>,
    share: &PartialSsaShare<S>,
) -> errors::Result<CompletedShare<S>> {
    Ok(DefaultShare {
        identifier: msg_to_scalar::<S>(&spi, msg)?.into(),
        value: Option::from(PixScalar::<S>::from_repr(share.0.clone()))
            .map(|s: PixScalar<S>| s.into())
            .ok_or(vsss_rs::Error::InvalidShare)?,
    })
}

/// Verifier for shares of a polynomial with the given [`SsaPolynomialId`].
#[derive(Debug, Clone, PartialEq, Eq)]
//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PartialSsaShareVerifier<S: PixSpec> {
    pub(crate) spi: SsaPolynomialId<S>,
    pub(crate) poly_commitment: Vec<ShareVerifierGroup<PixGroup<S>>>,
}

impl<S: PixSpec> PartialSsaShareVerifier<S> {
    /// Returns the [`SsaPolynomialId`] of the polynomial corresponding to this verifier.
    #[inline]
    pub fn spi(&self) -> &SsaPolynomialId<S> {
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
    pub fn into_serializable_commitments(self) -> (SsaPolynomialId<S>, Vec<PixGroupRepr<S>>) {
        (
            self.spi,
            self.poly_commitment
                .into_iter()
                .filter(|s| s.0 != PixGroup::<S>::generator()) // Generator is typically the first entry
                .map(|c| c.to_bytes())
                .collect(),
        )
    }

    /// Tries to create a new verifier from [`SsaPolynomialId`] and serialized polynomial coefficient commitments.
    ///
    /// The `poly_commitments` do not need to contain the generator, because it is added automatically.
    pub fn from_serializable_commitments(
        spi: SsaPolynomialId<S>,
        poly_commitments: Vec<PixGroupRepr<S>>,
    ) -> errors::Result<Self> {
        if poly_commitments.is_empty() {
            return Err(errors::PixError::InvalidInput);
        }

        let recv_commitments = poly_commitments
            .into_iter()
            .map(|c| {
                Option::<PixGroup<S>>::from(PixGroup::<S>::from_bytes(&c))
                    .map(ShareVerifierGroup::<PixGroup<S>>::from)
                    .ok_or(errors::PixError::InvalidInput)
            })
            .filter(|res: &errors::Result<ShareVerifierGroup<PixGroup<S>>>| {
                // Explicitly filter out the generator, because we're adding it later.
                // It is therefore allowed for the generator not to be present in the commitments.
                res.is_err() || res.as_ref().is_ok_and(|e| e.0 != PixGroup::<S>::generator())
            });

        // Re-add the generator as the first entry
        let poly_commitment = std::iter::once(Ok(ShareVerifierGroup::<PixGroup<S>>::generator()))
            .chain(recv_commitments)
            .collect::<errors::Result<Vec<_>>>()?;
        Ok(Self { spi, poly_commitment })
    }

    pub(crate) fn verify_completed_share(&self, share: &CompletedShare<S>) -> errors::Result<()> {
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
    pub fn verify(&self, share: &PartialSsaShare<S>, msg: impl AsRef<[u8]>) -> errors::Result<()> {
        self.verify_completed_share(&into_completed_share(self.spi, msg, share)?)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use anyhow::Context;
    use hopr_types::crypto::prelude::SimplePseudonym;
    use vsss_rs::{
        ParticipantIdGeneratorType,
        elliptic_curve::{
            Field,
            rand_core::{CryptoRng, RngCore},
        },
        feldman,
    };

    use super::*;
    use crate::types::SsaId;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
    pub struct TestSpec;

    impl PixSpec for TestSpec {
        type Cipher = hopr_types::crypto::primitives::ChaCha20;
        type Curve = k256::Secp256k1;
        type Digest = hopr_types::crypto::primitives::Sha3_256;
        type Pseudonym = SimplePseudonym;
    }

    type Share<S> = DefaultShare<IdentifierPrimeField<PixScalar<S>>, IdentifierPrimeField<PixScalar<S>>>;

    fn standard_shamir_generate<S: PixSpec>(
        secret: PixScalar<S>,
        t: usize,
        x: &[PixScalar<S>],
        mut rng: impl RngCore + CryptoRng,
    ) -> anyhow::Result<(Vec<Share<S>>, Vec<ShareVerifierGroup<PixGroup<S>>>)> {
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
        let mut rng = vsss_rs::elliptic_curve::rand_core::OsRng;
        let secret = k256::Scalar::random(&mut rng);

        let spi = SsaPolynomialId::new(SsaId::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1), 1);
        let x = (0..=20_u32)
            .map(|i| msg_to_scalar::<TestSpec>(&spi, i.to_be_bytes()).unwrap())
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
        let mut rng = vsss_rs::elliptic_curve::rand_core::OsRng;
        let secret = k256::Scalar::random(&mut rng);

        let spi = SsaPolynomialId::new(SsaId::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1), 1);
        let x = (0..=20_u32)
            .map(|i| msg_to_scalar::<TestSpec>(&spi, i.to_be_bytes()).unwrap())
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
}
