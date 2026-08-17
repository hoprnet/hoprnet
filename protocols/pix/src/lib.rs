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
use vsss_rs::{
    DefaultShare, IdentifierPrimeField,
    elliptic_curve::{Curve, CurveArithmetic, PrimeCurve, PrimeField, consts::U256},
};

pub mod ack_verify;
pub mod errors;
mod generator;
mod params;
mod reconstructor;
mod traits;
mod types;

pub use generator::{SHARE_EMISSION_WINDOW, SsaGeneratorConfig, SsaShareGenerator};
pub use params::{InvalidPixParams, PixParams, PixSuite};
pub use reconstructor::{
    AWAITING_ACK_ENTRY_BYTES, MAX_DEFERRED_ACKS_PER_CYCLE, MAX_DEFERRED_ACKS_PER_POLYNOMIAL, SsaCommitmentGuard,
    SsaReconstructor, SsaReconstructorConfig,
};
pub use traits::{EntryShareGenerator, ExitAcknowledgementShareProcessor, ShareResolution};
pub use types::{
    CoefficientIndex, EncryptedPartialSsaShare, GeneratedShare, PartialSsaShare, PolynomialIndex, RawSsaIndex,
    RecoveredSsa, SsaCommitment, SsaCommitmentProof, SsaCommitmentState, SsaId, SsaIndex, SsaPolyIndexPrefixSize,
    SsaPolynomialId, SsaRecoveryProgress, TaggedEncryptedPartialSsaShare,
};
pub use vsss_rs::elliptic_curve::{
    Field, Group,
    group::{GroupEncoding, cofactor::CofactorGroup},
};

#[doc(hidden)]
pub mod prelude {
    pub use super::*;
}

/// Coefficient index of the polynomial constant term.
///
/// The only coefficient PIX commits to: see [`SsaPartCommitment`] for why the rest are not sent.
/// The wire format still carries a `coefficient_index`, so a peer *may* send others — the Exit
/// ignores them.
pub const CONSTANT_TERM_COEFFICIENT: CoefficientIndex = 0;

/// Number of polynomials per SSA.
///
/// This and [`DEFAULT_POLY_THRESHOLD`] are the **deployed** split, not merely a library
/// convenience: `hopr-transport-session` aliases them as `DEFAULT_PIX_POLYS_PER_SSA` and
/// `DEFAULT_PIX_SHARES_PER_POLY`, and the Exit's accepted quota range is derived from their
/// product. Changing either changes what nodes negotiate, and a product that no longer matches
/// the Exit's range makes every PIX Session fail to establish — so move them together.
pub const DEFAULT_POLYS_PER_SSA: u16 = 8192;

/// Minimum number of shares needed to recover one polynomial of an SSA.
///
/// 64 rather than the 128 this was before the split was re-tuned. Dropping the non-constant
/// coefficient commitments (see [`SsaPartCommitment`]) put every commitment-side cost — wire
/// volume, ingest, reconstructor memory, share verification — on `polys` alone. What still grows
/// with the threshold is Exit interpolation (`O(threshold²)` per polynomial), fault-detection
/// latency (`threshold` return packets) and, on the Entry, the per-packet
/// [`SsaShareGenerator::next_share`], which evaluates a `threshold`-wide polynomial by Horner for
/// every share it emits. The full cost model is documented on `DEFAULT_PIX_POLYS_PER_SSA` in
/// `hopr-transport-session`.
///
/// # What 64 optimises
///
/// **Exit reconstruction capacity, deliberately, and not total network CPU.** The two sides do not
/// deserve equal weight: an Exit is a shared resource serving 10–30 concurrent clients while each
/// Entry generates only its own shares, so the Exit is the side that saturates first and the one
/// whose per-share cost sets what the network can carry. Both were measured (48 cores,
/// `acknowledge_shares/interpolation` and `SsaShareGenerator::next_share`, µs/share):
///
/// | threshold | Exit  | Entry | total |
/// | --------- | ----- | ----- | ----- |
/// | 16        | 13.15 | 0.90  | 14.05 |
/// | 32        | 11.12 | 1.20  | 12.32 |
/// | 48        | 10.62 | 1.51  | 12.13 |
/// | 64        | 10.68 | 1.82  | 12.50 |
///
/// The Exit curve *falls* and then flattens: fitting µs/polynomial = `A + B·t + C·t²` gives
/// `A ≈ 81 µs`, `B ≈ 7.61`, `C ≈ 0.028`, so per share it is `A/t + B + C·t` — the fixed
/// per-polynomial commitment opening is amortised over `threshold` shares and punishes *low*
/// thresholds harder than interpolation punishes high ones. The Exit-only optimum is
/// `t = √(A/C) ≈ 54`, flat within 0.5 % from 48 to 64, and 64 sits within 0.4 % of it.
///
/// Under the total column 48 is about 3 % cheaper. That reading is recorded rather than acted on:
/// it weights one Entry's share generation equally with the Exit work of the 10–30 clients that
/// Exit serves, which is not where the capacity limit is. Moving the split also moves the
/// negotiated per-SSA quota and everything derived from it, which is not worth 3 % of a model that
/// does not describe the bottleneck.
pub const DEFAULT_POLY_THRESHOLD: u8 = 64;

/// Share loss a polynomial's surplus is sized to absorb, as a reciprocal: `1/5` is 20 %.
///
/// The surplus is insurance against *lost* shares, and this is the loss rate it covers. A
/// polynomial reconstructs from the first `threshold` distinct shares to arrive out of
/// `threshold + surplus` emitted, so surviving a per-packet loss rate of `p` needs
/// `surplus >= threshold · p/(1−p)` — which makes the surplus inherently a **ratio** of the
/// threshold, and `surplus/(threshold + surplus)` the loss rate it tolerates.
///
/// At `threshold/4` that is 20 %, and [`default_surplus_for`] rounds *up* so it is a floor rather
/// than an approximation — see there for why the direction matters.
const SURPLUS_LOSS_TOLERANCE_DIVISOR: u8 = 4;

/// Shares to emit per polynomial beyond `threshold`, to absorb losses.
///
/// **A ratio, evaluated at the threshold actually configured** — see
// `SURPLUS_LOSS_TOLERANCE_DIVISOR` is deliberately unlinked here and below: it is crate-private, so
// an intra-doc link from this public function trips `rustdoc::private_intra_doc_links`, which the
// `nix build .#docs` job builds as an error.
/// `SURPLUS_LOSS_TOLERANCE_DIVISOR` for why a ratio is the physically meaningful shape.
///
/// This used to be a bare constant, `DEFAULT_POLY_THRESHOLD / 2`, evaluated once at the *default*
/// threshold and then applied whatever the configured one was. Deployments run thresholds from 16
/// to 64, so a single absolute surplus means wildly different insurance across that range: a flat
/// 20 covers 24 % loss at threshold 64 but 56 % at threshold 16, where it exceeds the shares it
/// insures. Since H5 the surplus is billed on purchase rather than on claim, so that over-insurance
/// is quota an Entry pays for in every deposit.
///
/// **Rounds up.** The result is a guarantee of *at least* the documented tolerance, not an
/// approximation of it: shares are indivisible, so a threshold that is not a multiple of
/// `SURPLUS_LOSS_TOLERANCE_DIVISOR` has to land on one side of 20 % or the other, and covering
/// less than the documented rate is the failure this function exists to prevent. Rounding down
/// undershot for every such threshold — 33 received 8 surplus shares and covered 19.51 % — and gave
/// thresholds 2 and 3 a surplus of **zero**, i.e. no loss tolerance at all, at and just above
/// [`MIN_POLY_THRESHOLD`]. Rounding up over-covers by less than one share, which is most visible at
/// the smallest thresholds (2 → 1 surplus → 33 %) and vanishes as the threshold grows (63 → 16 →
/// 20.3 %).
///
/// The deployed value is unaffected: [`DEFAULT_POLY_THRESHOLD`] is 64, a multiple of the divisor, so
/// this returns 16 either way and no negotiated quota moves.
pub const fn default_surplus_for(threshold: u8) -> u8 {
    // Not `(threshold + DIVISOR - 1) / DIVISOR`: that addition overflows a `u8` for thresholds above
    // 252, which `MAX_POLY_THRESHOLD` (255) admits, and would fail const evaluation rather than
    // merely misbehave.
    threshold.div_ceil(SURPLUS_LOSS_TOLERANCE_DIVISOR)
}

/// Shares emitted per polynomial beyond [`DEFAULT_POLY_THRESHOLD`], to absorb losses.
///
/// The third leg of the deployed split, and like the other two a single value rather than one per
/// crate: it used to be a literal `20` here and a separately-derived `32` in
/// `PixGlobalConfig::additional_shares`, which meant [`SsaGeneratorConfig::default`] modelled a
/// cycle no deployed node ever runs. That mattered little while the surplus was unpriced; it stopped
/// being harmless once the per-SSA quota started counting it, because the quota is a `const` and had
/// to pick one of the two.
///
/// Only the value of [`default_surplus_for`] at the default threshold. Anything that knows its own
/// threshold should call that instead — this constant cannot, which is exactly the defect above.
pub const DEFAULT_SURPLUS_SHARES: u8 = default_surplus_for(DEFAULT_POLY_THRESHOLD);

/// Maximum number of polynomials per SSA supported by the [`SsaReconstructor`].
pub const MAX_POLYS_PER_SSA: u16 = 16192;

/// Minimum SSA polynomial threshold.
///
/// A threshold of 1 would make every single share reconstruct its polynomial on its own, so the
/// secret sharing would hide nothing.
pub const MIN_POLY_THRESHOLD: u8 = 2;

/// Maximum SSA polynomial threshold supported by the [`SsaReconstructor`].
///
/// A byte, because the threshold shares the lower half of the negotiated [`PixParams`] word with
/// [`SsaGeneratorConfig::surplus_shares`] — see [`PixParams::to_u32`]. The bound is therefore
/// structural rather than merely checked; the constant exists to name it.
pub const MAX_POLY_THRESHOLD: u8 = u8::MAX;

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
    /// Domain separator used to derive the Fiat–Shamir challenge of an [`SsaCommitmentProof`].
    const HASH_COMMITMENT_PROOF_CONTEXT: &'static str = "HASH_SSA_COMMITMENT_PROOF";

    /// Stable, protocol-versioned hash-to-scalar suite identifier used for
    /// domain separation. This must be a fixed string — deriving it dynamically
    /// from Debug output would break wire compatibility when dependency versions
    /// change formatting.
    const HASH_TO_SCALAR_SUITE_ID: &'static [u8];

    /// Which curve this spec instantiates, as announced to the peer in [`PixParams`].
    ///
    /// Deliberately has no default. It must name the same curve as [`Curve`](Self::Curve), and a
    /// default would let a new spec inherit a wrong answer silently — the failure it exists to
    /// prevent is precisely two peers disagreeing about a curve neither of them states.
    const PIX_SUITE: PixSuite;

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

    /// Derives the Fiat–Shamir challenge of an [`SsaCommitmentProof`] over the client's
    /// `ssa_commitment` and the prover's `nonce_commitment`.
    ///
    /// `ssa_id` is bound in so that a proof cannot be replayed onto a different SSA index or a
    /// different Session's pseudonym.
    ///
    /// The Exit's own commitment is deliberately **not** bound in. The statement being proven is
    /// knowledge of `dlog(ssa_commitment)` alone, and the deposit is protected because the Exit's
    /// secret is what separates that from `dlog(ssa_commitment + exit_commitment)` — which holds
    /// regardless of what the challenge hashes. Binding it would only prevent reusing one proof for
    /// the same `ssa_commitment` against two different Exits, and an Entry that reuses its
    /// commitment does know its discrete log, so that case is honest (reuse is a linkability
    /// concern, not an exploit).
    fn commitment_proof_challenge(
        ssa_id: &SsaId<Self::Pseudonym>,
        ssa_commitment: &PixGroup<Self>,
        nonce_commitment: &PixGroup<Self>,
    ) -> errors::Result<PixScalar<Self>, Self::Pseudonym>
    where
        Self: Sized,
    {
        let ssa_index = ssa_id.ssa_index().get().to_be_bytes();
        let ssa_commitment = ssa_commitment.to_bytes();
        let nonce_commitment = nonce_commitment.to_bytes();

        hash_to_scalar::<Self::Curve, ExpandMsgXmd<Self::Digest>, <Self::Curve as Curve>::FieldBytesSize>(
            &[
                ssa_id.pseudonym().as_ref(),
                ssa_index.as_ref(),
                ssa_commitment.as_ref(),
                nonce_commitment.as_ref(),
            ],
            &[
                Self::HASH_TO_SCALAR_SUITE_ID,
                Self::HASH_COMMITMENT_PROOF_CONTEXT.as_bytes(),
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

/// Commitment to the constant term of the polynomial with the given [`SsaPolynomialId`].
///
/// ## Why only the constant term
///
/// This used to be a full Feldman verifier — a commitment to *every* coefficient — so that each
/// individual share could be checked the moment it arrived. Classic VSS needs that, because its
/// shares sit with mutually distrusting parties that reconstruct later. PIX has exactly **one**
/// shareholder: the Exit holds every share, reconstructs locally, is the whole quorum, and
/// consumes only the recovered constant term. Checking `a₀·G == C₀` once, on the reconstructed
/// part, is therefore deterministic and exact for the property actually relied upon — and costs
/// one scalar multiplication per polynomial instead of `threshold` per share.
///
/// What the per-coefficient commitments did buy was fault *isolation*: one bad share could be
/// rejected on arrival and its slot refilled from the surplus. That is given up. A share that
/// fails to reconstruct implies a dishonest or broken Entry — it travels inside a
/// Sphinx-authenticated SURB, and its decryption key is fixed by the very acknowledgement
/// challenge it is filed under, so there is no benign path to a corrupt one — and such an Entry
/// has already funded the deposit it thereby forfeits. The price paid is detection latency:
/// a dishonest Entry is caught on the `threshold`-th share of a polynomial rather than the first.
///
/// [`surplus_shares`](SsaGeneratorConfig::surplus_shares) still absorbs *lost* shares, since
/// reconstruction starts at the first `threshold` distinct shares that arrive.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SsaPartCommitment<S: PixSpec, P = <S as PixSpec>::Pseudonym> {
    pub(crate) spi: SsaPolynomialId<P>,
    #[cfg_attr(feature = "serde", serde(with = "elliptic_curve_tools::group"))]
    pub(crate) constant_term: PixGroup<S>,
}

impl<S: PixSpec> SsaPartCommitment<S, S::Pseudonym> {
    /// Creates a commitment from an **already decoded and subgroup-checked** group element.
    ///
    /// The only decode is [`decode_commitment`](Self::decode_commitment), performed once when the
    /// commitment arrives on the wire. Decompression requires a modular square root and is the
    /// dominant per-commitment cost, so nothing here decodes a second time.
    #[inline]
    pub fn from_decoded_commitment(spi: SsaPolynomialId<S::Pseudonym>, constant_term: PixGroup<S>) -> Self {
        Self { spi, constant_term }
    }

    /// Returns the [`SsaPolynomialId`] of the polynomial this commitment belongs to.
    #[inline]
    pub fn spi(&self) -> &SsaPolynomialId<S::Pseudonym> {
        &self.spi
    }

    /// Returns the commitment to the constant term of the polynomial.
    #[inline]
    pub fn constant_term(&self) -> &PixGroup<S> {
        &self.constant_term
    }

    /// Checks a reconstructed constant term against this commitment.
    ///
    /// This is the *entire* verification the Exit performs on a polynomial, and it happens once,
    /// after `threshold` shares have been interpolated. A mismatch means at least one of those
    /// shares did not come from the committed polynomial; it does not say which.
    #[inline]
    pub fn verify_reconstructed(&self, secret: &PixScalar<S>) -> bool {
        PixGroup::<S>::mul_by_generator(secret) == self.constant_term
    }

    /// Decodes a single serialized coefficient commitment into a group element.
    ///
    /// Rejects bytes that do not decode, and points outside the prime-order subgroup: Baby JubJub
    /// has cofactor 8, so small-order points can pass the plain on-curve check.
    ///
    /// No value-based filtering is applied — a coefficient commitment equal to the generator
    /// validly represents scalar coefficient 1 and must be preserved.
    pub fn decode_commitment(commitment: &PixGroupRepr<S>) -> errors::Result<PixGroup<S>, S::Pseudonym> {
        Option::<PixGroup<S>>::from(PixGroup::<S>::from_bytes(commitment))
            .filter(|pt| bool::from(pt.is_torsion_free()))
            .ok_or(errors::PixError::InvalidInput)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use hopr_types::{
        crypto::{
            crypto_traits,
            prelude::{ChainKeypair, Keypair, PublicKey, Secp256k1, SimplePseudonym},
        },
        primitive::prelude::Address,
    };
    use vsss_rs::{
        ParticipantIdGeneratorType, ReadableShareSet, ShareVerifierGroup,
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
        type Digest = hopr_types::crypto::primitives::Blake3;
        type Pseudonym = SimplePseudonym;

        const HASH_TO_SCALAR_SUITE_ID: &'static [u8] = b"Secp256k1_XMD:BLAKE3_SSWU_RO_";
        const PIX_SUITE: PixSuite = PixSuite::Secp256k1;

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

    fn test_spi() -> anyhow::Result<SsaPolynomialId<SimplePseudonym>> {
        Ok(SsaPolynomialId::new(
            SsaId::new(SimplePseudonym::try_from([0u8; 10].as_ref())?, 1.try_into()?),
            1,
        ))
    }

    /// The commitment PIX keeps must be exactly the constant-term entry of a standard Feldman
    /// verifier set, and interpolating `threshold` of the standard shares must open it.
    ///
    /// This is the replacement for the old per-share verification test: the property the Exit now
    /// relies on is not "every share lies on the committed polynomial" but "the reconstructed
    /// constant term is the one that was committed to".
    #[test]
    fn ssa_part_commitment_must_correspond_to_standard() -> anyhow::Result<()> {
        const THRESHOLD: usize = 10;

        let mut rng = rand::rng();
        let secret = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::random(&mut rng);
        let spi = test_spi()?;

        let x = (0..=20_u32)
            .map(|i| TestSpec::msg_to_scalar(&spi, i.to_be_bytes()).unwrap())
            .collect::<Vec<_>>();

        let (shares, verifier) = standard_shamir_generate::<TestSpec>(secret, THRESHOLD, &x, &mut rng)?;
        assert_eq!(shares.len(), x.len());
        // [generator, C₀, C₁ … C_{t-1}] — PIX now keeps only the second entry.
        assert_eq!(verifier.len(), THRESHOLD + 1);

        let commitment = SsaPartCommitment::<TestSpec>::from_decoded_commitment(spi, verifier[1].0);
        assert_eq!(&verifier[1].0, commitment.constant_term());
        assert_eq!(&spi, commitment.spi());

        // Exactly `threshold` shares suffice, and they open the commitment.
        let reconstructed = shares[..THRESHOLD].to_vec().combine().map_err(anyhow::Error::msg)?.0;
        assert_eq!(secret, reconstructed, "threshold shares must recover the secret");
        assert!(commitment.verify_reconstructed(&reconstructed));

        // Any other scalar must not.
        assert!(!commitment.verify_reconstructed(&(reconstructed + PixScalar::<TestSpec>::ONE)));

        Ok(())
    }

    /// A single corrupted share is not detected on arrival — that is the cost of dropping the
    /// per-coefficient commitments — but it does surface at reconstruction.
    #[test]
    fn ssa_part_commitment_must_reject_a_corrupted_share_set() -> anyhow::Result<()> {
        const THRESHOLD: usize = 10;

        let mut rng = rand::rng();
        let secret = crypto_traits::elliptic_curve::Scalar::<Secp256k1>::random(&mut rng);
        let spi = test_spi()?;

        let x = (0..=20_u32)
            .map(|i| TestSpec::msg_to_scalar(&spi, i.to_be_bytes()).unwrap())
            .collect::<Vec<_>>();

        let (mut shares, verifier) = standard_shamir_generate::<TestSpec>(secret, THRESHOLD, &x, &mut rng)?;
        let commitment = SsaPartCommitment::<TestSpec>::from_decoded_commitment(spi, verifier[1].0);

        *shares[3].value.as_mut() += PixScalar::<TestSpec>::ONE;

        let reconstructed = shares[..THRESHOLD].to_vec().combine().map_err(anyhow::Error::msg)?.0;
        assert_ne!(secret, reconstructed);
        assert!(
            !commitment.verify_reconstructed(&reconstructed),
            "a corrupted share must make the reconstructed part fail its commitment"
        );

        Ok(())
    }

    /// A commitment equal to the generator represents constant term 1 and must survive decoding —
    /// no value-based filtering is applied.
    #[test]
    fn decode_commitment_accepts_a_generator_valued_commitment() -> errors::Result<(), SimplePseudonym> {
        let generator = PixGroup::<TestSpec>::generator();
        let decoded = SsaPartCommitment::<TestSpec>::decode_commitment(&generator.to_bytes())?;
        assert_eq!(generator, decoded);
        Ok(())
    }
}
