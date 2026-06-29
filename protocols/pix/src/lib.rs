use std::ops::Add;

use elliptic_curve::{
    Curve, CurveArithmetic, Field, PrimeCurve, PrimeField,
    consts::U256,
    generic_array::{
        ArrayLength,
        typenum::{IsLess, IsLessOrEqual},
    },
    group::cofactor::CofactorGroup,
    hash2curve::{ExpandMsgXmd, FromOkm, GroupDigest},
    ops::MulByGenerator,
};
use hopr_types::crypto::{
    crypto_traits::{BlockSizeUser, FixedOutput, HashMarker, KeyIvInit, OutputSizeUser, StreamCipher},
    prelude::Pseudonym,
};
#[cfg(feature = "rayon")]
use hopr_utils::parallelize::cpu::rayon::prelude::*;

pub mod ack_verify;
pub mod combine;
mod errors;
mod generator;
mod reconstructor;
mod traits;
mod types;

pub use combine::{CombineError, RawPolynomial, ReadableShareSet, Share};
pub use elliptic_curve::{Group, group::GroupEncoding};
pub use generator::{SsaGeneratorConfig, SsaShareGenerator};
pub use reconstructor::{SsaReconstructor, SsaReconstructorConfig};
pub use traits::{EntryShareGenerator, ExitAcknowledgementShareProcessor, ShareResolution};
pub use types::{
    CoefficientIndex, EncryptedPartialSsaShare, GeneratedShare, PartialSsaShare, PolynomialIndex, RawSsaIndex,
    RecoveredSsa, SsaCommitment, SsaCommitmentState, SsaId, SsaIndex, SsaPolyIndexPrefixSize, SsaPolynomialId,
    TaggedEncryptedPartialSsaShare,
};

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
}

/// Number of polynomials per SSA.
pub const DEFAULT_POLYS_PER_SSA: usize = 8192;
/// Minimum number of shares to recover a part of an SSA.
pub const DEFAULT_POLY_THRESHOLD: usize = 128;

/// Maximum number of polynomials per SSA supported by the [`SsaReconstructor`].
pub const MAX_POLYS_PER_SSA: usize = 16192;
/// Maximum SSA polynomial threshold supported by the [`SsaReconstructor`].
pub const MAX_POLY_THRESHOLD: usize = 4096;

/// Specification of the Protocol for Incentivization of eXits (PIX) instantiation.
pub trait PixSpec: Send + Sync + 'static
where
    PixScalar<Self>: PrimeField + FromOkm,
    PixGroup<Self>: Group<Scalar = PixScalar<Self>> + GroupEncoding + Default + CofactorGroup,
    PixGroupRepr<Self>: std::fmt::Debug + PartialEq + Eq,
    <PixDigest<Self> as OutputSizeUser>::OutputSize: IsLess<U256>,
    <PixDigest<Self> as OutputSizeUser>::OutputSize: IsLessOrEqual<<PixDigest<Self> as BlockSizeUser>::BlockSize>,
    <Self::Curve as Curve>::FieldBytesSize: Add<SsaPolyIndexPrefixSize>,
    <<Self::Curve as Curve>::FieldBytesSize as Add<SsaPolyIndexPrefixSize>>::Output: ArrayLength<u8>,
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
    const KEY_DERIVATION_CONTEXT: &str = "HASH_SSA_POLY_SHARE";
    /// Domain separator used to derive the X value of a share.
    const HASH_SCALAR_DERIVATION_CONTEXT: &str = "HASH_SSA_POLY_SHARE_SCALAR";

    /// Performs conversion of the given `spi` and `msg` into [`PixScalar`] of this spec.
    fn msg_to_scalar(
        spi: &SsaPolynomialId<Self::Pseudonym>,
        msg: impl AsRef<[u8]>,
    ) -> errors::Result<PixScalar<Self>, Self::Pseudonym>
    where
        Self: Sized,
    {
        Ok(<Self::Curve as GroupDigest>::hash_to_scalar::<
            ExpandMsgXmd<Self::Digest>,
        >(
            &[
                msg.as_ref(),
                spi.pseudonym().as_ref(),
                spi.ssa_index().get().to_be_bytes().as_ref(),
                spi.poly_index().to_be_bytes().as_ref(),
            ],
            &[
                format!(
                    "{:?}_XMD:{:?}_SSWU_RO_",
                    Self::Curve::default(),
                    Self::Digest::default()
                )
                .as_bytes(),
                Self::HASH_SCALAR_DERIVATION_CONTEXT.as_bytes(),
            ],
        )?)
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

/// A completed share with identifier and value.
pub(crate) type CompletedShare<S> = Share<PixScalar<S>>;

/// Wrapper for group elements that provides arithmetic operations.
///
/// This is a compatibility wrapper that mirrors the functionality of
/// ShareVerifierGroup from vsss-rs, implemented directly using elliptic-curve types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ShareVerifierGroup<G>(pub G);

impl<G> ShareVerifierGroup<G> {
    /// Creates a new ShareVerifierGroup from a group element.
    pub fn from(group: G) -> Self {
        Self(group)
    }

    /// Returns true if the group element is the identity.
    pub fn is_zero(&self) -> bool
    where
        G: Group,
    {
        bool::from(self.0.is_identity())
    }

    /// Converts the group element to its byte representation.
    pub fn to_bytes(&self) -> <G as GroupEncoding>::Repr
    where
        G: GroupEncoding,
    {
        self.0.to_bytes()
    }
}

impl<G> Default for ShareVerifierGroup<G>
where
    G: Default,
{
    fn default() -> Self {
        Self(G::default())
    }
}

impl<G> From<G> for ShareVerifierGroup<G> {
    fn from(group: G) -> Self {
        Self(group)
    }
}

impl<G> ShareVerifierGroup<G>
where
    G: MulByGenerator,
{
    /// Creates a ShareVerifierGroup with the generator element.
    pub fn generator() -> Self {
        Self(G::generator())
    }
}

impl<G> std::ops::Add<G> for ShareVerifierGroup<G>
where
    G: Group,
{
    type Output = Self;
    fn add(self, rhs: G) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl<G> std::ops::Sub<G> for ShareVerifierGroup<G>
where
    G: Group,
{
    type Output = Self;
    fn sub(self, rhs: G) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl<G> std::ops::Mul<&G::Scalar> for ShareVerifierGroup<G>
where
    G: Group,
{
    type Output = Self;
    fn mul(self, rhs: &G::Scalar) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[inline]
pub(crate) fn into_completed_share<S: PixSpec>(
    identifier: PixScalar<S>,
    share: &PartialSsaShare<S>,
) -> errors::Result<CompletedShare<S>, S::Pseudonym> {
    let value =
        Option::from(PixScalar::<S>::from_repr(share.0.clone())).ok_or(errors::PixError::InvalidShareNoContext)?;
    Share::new(identifier, value).ok_or(errors::PixError::InvalidShareNoContext)
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
    pub fn into_serializable_commitments(self) -> (SsaPolynomialId<S::Pseudonym>, Vec<PixGroupRepr<S>>) {
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
        spi: SsaPolynomialId<S::Pseudonym>,
        poly_commitments: Vec<PixGroupRepr<S>>,
    ) -> errors::Result<Self, S::Pseudonym> {
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
            .filter(|res: &errors::Result<ShareVerifierGroup<PixGroup<S>>, S::Pseudonym>| {
                // Explicitly filter out the generator, because we're adding it later.
                // It is therefore allowed for the generator not to be present in the commitments.
                res.is_err() || res.as_ref().is_ok_and(|e| e.0 != PixGroup::<S>::generator())
            });

        // Re-add the generator as the first entry
        let poly_commitment = std::iter::once(Ok(ShareVerifierGroup::<PixGroup<S>>::generator()))
            .chain(recv_commitments)
            .collect::<errors::Result<Vec<_>, S::Pseudonym>>()?;
        Ok(Self { spi, poly_commitment })
    }

    pub(crate) fn verify_completed_share(&self, share: &CompletedShare<S>) -> errors::Result<(), S::Pseudonym> {
        if bool::from(share.value().is_zero()) || bool::from(share.identifier().is_zero()) {
            return Err(errors::PixError::InvalidShareNoContext);
        }
        if self.poly_commitment[0].is_zero() {
            return Err(errors::PixError::InvalidGenerator);
        }

        let mut power = PixScalar::<S>::ONE;
        let mut scalars = Vec::with_capacity(self.poly_commitment.len() - 2);

        // The below multi-scalar multiplication method (MSM) is more efficient
        // for large polynomial degrees than Horner's method because it can be parallelized.

        // Computes x^1, x^2, x^3, ... x^t
        for _ in 0..self.poly_commitment.len() - 2 {
            power *= share.identifier();
            scalars.push(power);
        }

        #[cfg(feature = "rayon")]
        let scalars_iter = scalars.into_par_iter();

        #[cfg(not(feature = "rayon"))]
        let scalars_iter = scalars.into_iter();

        // v[1] + v[2]*x + v[3]*x^2 + ... + v[t]*x^t
        let rhs: PixGroup<S> = self.poly_commitment[1].0
            + scalars_iter
                .enumerate()
                .map(|(idx, c)| (self.poly_commitment[idx + 2] * &c).0)
                .sum::<PixGroup<S>>();

        let lhs = self.poly_commitment[0].0 * share.value();

        let res = rhs - lhs;

        if bool::from(res.is_identity()) {
            Ok(())
        } else {
            Err(errors::PixError::InvalidShareNoContext)
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
    use elliptic_curve::rand_core::OsRng;
    use hopr_types::{
        crypto::prelude::{ChainKeypair, Keypair, PublicKey, SimplePseudonym},
        primitive::prelude::Address,
    };

    use super::*;
    use crate::types::SsaId;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Hash, Ord, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct TestSpec;

    impl PixSpec for TestSpec {
        type AddressPrivateKey = ChainKeypair;
        type Cipher = hopr_types::crypto::primitives::ChaCha20;
        type Curve = k256::Secp256k1;
        type DepositAddress = Address;
        type Digest = hopr_types::crypto::primitives::Sha3_256;
        type Pseudonym = SimplePseudonym;

        fn group_to_deposit_address(group: PixGroup<Self>) -> Option<Self::DepositAddress> {
            PublicKey::try_from(group.to_affine()).ok().map(|pk| pk.to_address())
        }

        fn scalar_to_private_key(scalar: PixScalar<Self>) -> Option<Self::AddressPrivateKey> {
            ChainKeypair::from_secret(scalar.to_bytes().as_ref()).ok()
        }
    }

    #[test]
    fn ssa_share_verifier_must_correspond_to_standard() -> anyhow::Result<()> {
        use vsss_rs::{
            DefaultShare, IdentifierPrimeField, ParticipantIdGeneratorType, Share, ValueGroup, ValuePrimeField, feldman,
        };

        let mut rng = OsRng;
        let secret = k256::Scalar::random(&mut rng);
        let t = 10;

        let spi = SsaPolynomialId::new(
            SsaId::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1.try_into()?),
            1,
        );
        let x: Vec<k256::Scalar> = (0..=20u32)
            .map(|i| TestSpec::msg_to_scalar(&spi, i.to_be_bytes()).unwrap())
            .collect();

        // Generate shares using vsss-rs
        type VsssShare = DefaultShare<IdentifierPrimeField<k256::Scalar>, ValuePrimeField<k256::Scalar>>;
        type VsssVerifier = ValueGroup<k256::ProjectivePoint>;
        let (vsss_shares, vsss_verifier) = feldman::split_secret_with_participant_generator::<VsssShare, VsssVerifier>(
            t,
            x.len(),
            &ValuePrimeField::from(secret),
            None,
            &mut rng,
            &[ParticipantIdGeneratorType::list(
                &x.iter().map(|x| ValuePrimeField::from(*x)).collect::<Vec<_>>(),
            )],
        )
        .map_err(anyhow::Error::msg)?;

        assert_eq!(vsss_verifier.len(), t + 1);

        // Create our verifier from vsss-rs verifier
        let verifier: PartialSsaShareVerifier<TestSpec> = PartialSsaShareVerifier {
            spi,
            poly_commitment: vsss_verifier.iter().map(|v| ShareVerifierGroup::from(v.0)).collect(),
        };

        assert_eq!(vsss_shares.len(), x.len());
        assert_eq!(verifier.min_shares(), t);
        assert_eq!(verifier.poly_commitment.len() - 1, verifier.min_shares());

        for (i, s) in vsss_shares.into_iter().enumerate() {
            let share: PartialSsaShare<TestSpec> = PartialSsaShare(s.value().to_repr());
            verifier
                .verify(&share, (i as u32).to_be_bytes())
                .context(format!("Verification failed for share index {i}"))?;
        }

        Ok(())
    }

    #[test]
    fn ssa_share_verifier_must_be_convertible_to_and_from_serializable_commitments() -> anyhow::Result<()> {
        use vsss_rs::{ParticipantIdGeneratorType, ValueGroup, ValuePrimeField, feldman};

        let mut rng = OsRng;
        let secret = k256::Scalar::random(&mut rng);
        let t = 10;
        let n = 21;

        let spi = SsaPolynomialId::new(
            SsaId::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1.try_into()?),
            1,
        );

        // Generate identifiers for the test
        let identifiers: Vec<k256::Scalar> = (1..=n as u32).map(|i| k256::Scalar::from(i)).collect();

        // Generate verifier using vsss-rs
        type VsssVerifier = ValueGroup<k256::ProjectivePoint>;
        let (_, vsss_verifier) = feldman::split_secret_with_participant_generator::<
            vsss_rs::DefaultShare<vsss_rs::IdentifierPrimeField<k256::Scalar>, vsss_rs::ValuePrimeField<k256::Scalar>>,
            VsssVerifier,
        >(
            t,
            n,
            &ValuePrimeField::from(secret),
            None,
            &mut rng,
            &[ParticipantIdGeneratorType::list(
                &identifiers
                    .iter()
                    .map(|x| ValuePrimeField::from(*x))
                    .collect::<Vec<_>>(),
            )],
        )
        .map_err(anyhow::Error::msg)?;

        let verifier_1: PartialSsaShareVerifier<TestSpec> = PartialSsaShareVerifier {
            spi,
            poly_commitment: vsss_verifier.iter().map(|v| ShareVerifierGroup::from(v.0)).collect(),
        };

        assert!(PartialSsaShareVerifier::<TestSpec>::from_serializable_commitments(spi, vec![]).is_err());

        let (spi, poly_commitments) = verifier_1.clone().into_serializable_commitments();
        let verifier_2: PartialSsaShareVerifier<TestSpec> =
            PartialSsaShareVerifier::from_serializable_commitments(spi, poly_commitments)?;
        assert_eq!(verifier_1, verifier_2);

        Ok(())
    }
}
