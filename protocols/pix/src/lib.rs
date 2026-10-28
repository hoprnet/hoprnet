// PIX (Protocol for Incentivization of eXits): SSA commitment generation and reconstruction.
#![deny(clippy::debug_assert_with_mut_call)]

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

pub use generator::{
    EmissionProgress, SHARE_EMISSION_WINDOW, SsaGeneratorConfig, SsaShareGenerator, min_emission_for_early_recovery,
};
pub use params::{InvalidPixParams, PixParams, PixSuite};
pub use reconstructor::{
    AWAITING_ACK_ENTRY_BYTES, MAX_DEFERRED_ACKS_PER_CYCLE, MAX_DEFERRED_ACKS_PER_POLYNOMIAL,
    PART_BUILDER_OVERHEAD_BYTES, SsaCommitmentGuard, SsaReconstructor, SsaReconstructorConfig, SsaRetirementScope,
    peak_cycle_bytes, peak_share_buffer_bytes,
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
pub const DEFAULT_POLYS_PER_SSA: u16 = 4096;

/// Minimum number of shares needed to recover one polynomial of an SSA.
///
/// 32, down from the 128 this was before the split was first re-tuned and the 64 it was after —
/// see below for why it now sits under the measured optimum. Dropping the non-constant
/// coefficient commitments (see [`SsaPartCommitment`]) put every commitment-side cost — wire
/// volume, ingest, reconstructor memory, share verification — on `polys` alone. What still grows
/// with the threshold is Exit interpolation (`O(threshold²)` per polynomial), fault-detection
/// latency (`threshold` return packets) and, on the Entry, the per-packet
/// [`SsaShareGenerator::next_share`], which evaluates a `threshold`-wide polynomial by Horner for
/// every share it emits. The full cost model is documented on `DEFAULT_PIX_POLYS_PER_SSA` in
/// `hopr-transport-session`.
///
/// # What 64 optimised, and why 32 is deployed instead
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
/// `t = √(A/C) ≈ 54`, flat within 0.5 % from 48 to 64, and 64 — the value deployed before the
/// current split — sits within 0.4 % of it.
///
/// Under the total column 48 is about 3 % cheaper. That reading is recorded rather than acted on:
/// it weights one Entry's share generation equally with the Exit work of the 10–30 clients that
/// Exit serves, which is not where the capacity limit is.
///
/// **The deployed value is 32, below the Exit-only optimum of ~54, and that is a deliberate
/// trade.** This table prices one *share*, but the quota prices `polys × (threshold + surplus)`, and
/// the surplus is a ratio of the threshold that grows as the threshold falls. A configuration that
/// tolerates 50 % share loss therefore needs a *lower* threshold to be cheap: at `polys = 4096`,
/// `t = 32, s = 79` is 629.6 MiB per cycle where `t = 64, s = 126` is 1078 MiB. The Exit pays ~5 %
/// more per share at 32 than at 54, and the Entry pays ~40 % less quota per cycle — which is the
/// side that pays in money rather than in CPU. See [`DEFAULT_SURPLUS_SHARES`] for the loss tolerance
/// that fixes the surplus, and note that `s/(t+s)` is what the tolerance depends on, not `s` alone.
pub const DEFAULT_POLY_THRESHOLD: u8 = 32;

/// Fraction of cycles [`default_surplus_for`] sizes the surplus to carry through the loss of half of
/// all shares.
///
/// The surplus is insurance against *lost* shares — a share is lost when the first relayer of its
/// return path never acknowledges it, so a dead return relayer takes its whole share of the path's
/// traffic with it. Half is the design point because it is what losing one of two return relayers
/// costs, and a cycle that survives it at this rate is one the operator does not have to tune.
///
/// [`tolerated_share_loss`] reads the same target in the other direction — the loss a given surplus
/// carries rather than the surplus a given loss needs — so that the two agree on what "tolerates"
/// means.
const SURPLUS_TARGET_CYCLE_RECOVERY: f64 = 0.99;

/// Shares to emit per polynomial beyond `threshold`, to absorb losses: the fewest with which a
/// cycle of `polys` polynomials still recovers at 99 % when every share is lost independently with
/// probability one half.
///
/// A polynomial reconstructs from the first `threshold` of the `threshold + surplus` shares emitted
/// for it, so it survives with probability `P(Binom(threshold + surplus, 1/2) >= threshold)`. A
/// *cycle* recovers only if **every** one of its `polys` polynomials does — the SSA is the sum of
/// their constant terms — so that probability is raised to the power `polys`, and the surplus
/// depends on the polynomial count as well as on the threshold.
///
/// This replaced a fixed ratio, `threshold / 4`, documented as covering 20 % share loss. It covered
/// 20 % per *polynomial*; with the exponent applied, a cycle of 4096 tolerated about 3 %, and the
/// former `8192 × (64 + 16)` about 5 %. Deriving from the target also gives every split the same
/// tolerance, where any fixed ratio gives each a different one.
///
/// The required ratio *falls* as the threshold rises — at 4096 polynomials it is 52 surplus shares
/// at threshold 16 (3.3×), 79 at 32 (2.5×), 126 at 64 (2.0×) and 211 at 128 (1.6×) — because a
/// larger sample concentrates around its mean.
///
/// **Saturates at 255**, the byte the surplus travels in. From a threshold of around 160 the target
/// needs more than that, and the result is then the most the wire can carry rather than a surplus
/// meeting the target.
///
/// `const` so that [`DEFAULT_SURPLUS_SHARES`] can be defined by it, and deterministic for the same
/// reason: it is evaluated with IEEE `f64` addition, multiplication and division only, whose results
/// are fixed.
pub const fn default_surplus_for(polys: u16, threshold: u8) -> u8 {
    let mut surplus = 0u8;
    while surplus < u8::MAX
        && cycle_recovery_at_half_share_loss(polys, threshold, surplus) < SURPLUS_TARGET_CYCLE_RECOVERY
    {
        surplus += 1;
    }
    surplus
}

/// Probability that a cycle of `polys` polynomials, each emitting `threshold + surplus` shares,
/// recovers when every share is lost independently with probability one half.
const fn cycle_recovery_at_half_share_loss(polys: u16, threshold: u8, surplus: u8) -> f64 {
    let emitted = threshold as u32 + surplus as u32;

    // The per-polynomial failure tail `P(Binom(emitted, 1/2) < threshold)`, summed term by term
    // rather than taken as one minus the success: the tail is ~1e-6 at the deployed split, and the
    // subtraction would cancel most of its digits. `C(emitted, 0) / 2^emitted` underflows nothing,
    // since `emitted` is at most 510.
    let mut term = 1.0f64;
    let mut i = 0;
    while i < emitted {
        term *= 0.5;
        i += 1;
    }
    let mut failure = 0.0f64;
    let mut k = 0u32;
    while k < threshold as u32 {
        failure += term;
        term = term * (emitted - k) as f64 / (k + 1) as f64;
        k += 1;
    }

    // `(1 - failure)^polys` by squaring, since `powi` is not `const`.
    let mut base = 1.0 - failure;
    let mut exponent = polys;
    let mut recovery = 1.0f64;
    while exponent > 0 {
        if exponent & 1 == 1 {
            recovery *= base;
        }
        base *= base;
        exponent >>= 1;
    }
    recovery
}

/// The largest per-share loss rate at which a cycle of `polys` polynomials, each emitting
/// `threshold + surplus` shares, still recovers at 99 % — the target [`default_surplus_for`] sizes
/// the surplus to, so the deployed split tolerates just over the 50 % it was derived for.
///
/// Shares are taken to be lost independently. A polynomial survives with probability
/// `P(Binom(threshold + surplus, 1 − loss) >= threshold)`, the cycle with that raised to the power
/// `polys`, and the result falls monotonically as the loss rises; the rate where it crosses the target
/// is found by bisection, to within 2⁻⁶⁰.
///
/// **Not `surplus/(threshold + surplus)`.** That is the break-even ratio: the loss at which a
/// polynomial receives exactly `threshold` shares *on average*. Around half of all polynomials fall
/// short of their average, and a cycle needs every one of them, so a cycle at break-even fails. At
/// the deployed `4096 × (32 + 79)` the ratio is 71.2 % and this is 50.3 %; a cycle losing 71.2 % of its
/// shares recovers with probability indistinguishable from zero.
///
/// Evaluated in log space, which is what lets one implementation serve the whole accepted range. The
/// recurrence [`default_surplus_for`] uses starts from the all-lost term `loss^(threshold + surplus)`
/// and multiplies its way up from there; at 510 shares that term underflows to zero for any loss
/// below about a quarter and takes every later term with it, while the tolerances the bisection has
/// to reach in that corner run down to 10⁻⁹. Here each term is a logarithm, the tail is a
/// log-sum-exp, and the cycle probability is `exp(polys · ln(1 − tail))` rather than a power of
/// `1 − tail`, whose subtraction keeps only ten or so of the tail's digits at the deployed split.
///
/// Not `const`, because `ln` and `exp` are not, and nothing needs it to be: this prices service
/// at runtime rather than defining a constant.
///
/// A cycle with no polynomials, or polynomials with a zero threshold, recovers whatever is lost, and
/// tolerates a loss of one.
pub fn tolerated_share_loss(polys: u16, threshold: u8, surplus: u8) -> f64 {
    // Each step halves the bracket, so 60 of them leave it at 2⁻⁶⁰ — absolute, and still under a
    // relative 10⁻⁹ at the smallest tolerance in the accepted range, ~2.4 × 10⁻⁹.
    const BISECTION_STEPS: u32 = 60;

    if cycle_recovery_at_share_loss(polys, threshold, surplus, 1.0) >= SURPLUS_TARGET_CYCLE_RECOVERY {
        return 1.0;
    }

    // Recovery is one at no loss and below the target at total loss, so the crossing lies between.
    let (mut tolerated, mut fatal) = (0.0f64, 1.0f64);
    for _ in 0..BISECTION_STEPS {
        let loss = 0.5 * (tolerated + fatal);
        if cycle_recovery_at_share_loss(polys, threshold, surplus, loss) >= SURPLUS_TARGET_CYCLE_RECOVERY {
            tolerated = loss;
        } else {
            fatal = loss;
        }
    }
    tolerated
}

/// Probability that a cycle of `polys` polynomials, each emitting `threshold + surplus` shares,
/// recovers when every share is lost independently with probability `loss`.
///
/// The general form of [`cycle_recovery_at_half_share_loss`], in log space — see
/// [`tolerated_share_loss`] for why.
fn cycle_recovery_at_share_loss(polys: u16, threshold: u8, surplus: u8, loss: f64) -> f64 {
    if polys == 0 || threshold == 0 || loss <= 0.0 {
        return 1.0;
    }
    if loss >= 1.0 {
        return 0.0;
    }

    let emitted = threshold as u32 + surplus as u32;
    let ln_delivered = (-loss).ln_1p();
    let ln_lost = loss.ln();

    // `ln` of the terms of the per-polynomial failure tail, `C(emitted, k) (1 − loss)^k
    // loss^(emitted − k)` for `k < threshold`, with the binomial coefficient built up alongside.
    // Recomputed rather than collected, so that the two passes of the log-sum-exp allocate nothing.
    let ln_terms = move || {
        let mut ln_choose = 0.0f64;
        (0..threshold as u32).map(move |k| {
            let ln_term = ln_choose + k as f64 * ln_delivered + (emitted - k) as f64 * ln_lost;
            ln_choose += ((emitted - k) as f64).ln() - ((k + 1) as f64).ln();
            ln_term
        })
    };
    let peak = ln_terms().fold(f64::NEG_INFINITY, f64::max);
    let ln_failure = peak + ln_terms().map(|ln_term| (ln_term - peak).exp()).sum::<f64>().ln();

    // Clamped because rounding can carry a tail of one a hair past it, and `ln_1p` of anything below
    // minus one is NaN rather than the zero recovery it means.
    let failure = ln_failure.exp().min(1.0);
    (polys as f64 * (-failure).ln_1p()).exp()
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
/// [`default_surplus_for`] at the deployed split, which is 79: `surplus/(threshold + surplus) =
/// 0.712`, carrying a 50 % per-share loss at 99 % of cycles. That is the configuration the
/// localcluster soak exercises with a relay killed mid-run.
///
/// **A negotiated wire parameter**, so a change to the derivation that moves it is a protocol
/// change: the Exit's default `quota_range` is computed from it. A test pins the value for exactly
/// that reason.
///
/// It remains a *default*, not a bound. Any explicit value within the wire byte is accepted — see
/// [`SsaGeneratorConfig`] for why.
pub const DEFAULT_SURPLUS_SHARES: u8 = default_surplus_for(DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD);

/// Maximum number of polynomials per SSA supported by the [`SsaReconstructor`].
pub const MAX_POLYS_PER_SSA: u16 = 16192;

/// Retransmission requests the Entry answers for one SSA cycle.
///
/// A cycle's commitment ships as hundreds of packets, and a single lost one strands the whole cycle
/// (see [`SsaShareGenerator::recommit`]), so the Exit must be able to ask again for the pieces it
/// never received. This bounds how often it may: one request can legally name every polynomial of
/// the cycle, and an answer is the entire burst again, so an unbounded count is a packet
/// amplification lever available to a peer that has already been served.
///
/// Eight, because the Exit re-asks on an idle timer rather than once. At the shipped Exit settings
/// — a 3 s re-request interval inside a 20 s commitment deadline — a cycle can produce about six
/// attempts, and a peer batching several SSAs per request produces proportionally more against a
/// scaled deadline. The excess is simply refused: an unanswered request costs the Exit one packet
/// and this node nothing, which is why the two sides are not required to agree on the figure.
pub const MAX_COMMITMENT_RETRANSMISSIONS: u8 = 8;

/// Cycles per pseudonym whose commitment material the [`SsaShareGenerator`] retains for
/// retransmission.
///
/// The newest this many are kept and the oldest is evicted, and within a live cache entry that is
/// the only thing that releases them — emission deliberately does not enter into it. (The entry as a
/// whole goes on idle eviction, which is a separate lifetime; see below.) Whether a re-sent commitment is
/// still worth anything is a question about what the peer holds, and none of that is visible from
/// this side: it completes a cycle out of the shares it has already buffered plus the commitment it
/// is still missing, on its own delivery deadline. Releasing on drain instead would give up the
/// repair exactly where it is worth most, a cycle whose every share has gone out and whose only
/// missing piece is the one a lost packet took.
///
/// Eighteen is `MAX_OVERLAPPING_BATCHES` (2) generations of `MAX_SSA_BATCH_SIZE` (9) cycles, the
/// figures `hopr-transport-session` is built to; they are not visible from here, so the derivation is
/// written down rather than imported. It leaves a cycle covered for the whole of its own batch plus
/// one further generation, and the successor gate makes that second generation expensive — a peer
/// cannot reach it without draining nearly every cycle of the first, by which point it has been
/// asking for far longer than [`MAX_COMMITMENT_RETRANSMISSIONS`] attempts on any sane re-request
/// interval allow.
///
/// Costs `polynomials_per_ssa × 33 B` per cycle — 135 kB at the deployed dimensions, 2.4 MB per
/// pseudonym with the cap full. Entry-side state, which is why it is not charged against an Exit's
/// incoming-Session memory budget: that reserves reconstructor state for Sessions this node *serves*,
/// and nothing else the generator holds is charged there either.
///
/// It lives as long as the pseudonym's entry in the generator's cache, which idle eviction alone
/// releases — closing a Session clears nothing here. So a node holds this for every pseudonym it has
/// committed a cycle for within that window, Sessions long closed included, and churn accumulates it
/// rather than it tracking the live Session count.
/// What bounds the consequence is that the same entry keeps its polynomial state under exactly the
/// same rule, and that term is larger by a factor of about sixty: 135 kB here against ~8 MB there,
/// per cycle. A Session whose cycles are still queued when it closes — the ordinary case, since
/// draining one takes `polynomials_per_ssa × (threshold + surplus)` shares, 455 k at the deployed
/// dimensions — leaves both, and this is under 2% of it. Reaching the cap instead means having
/// drained eighteen cycles, some eight million shares, by which point the polynomials are gone and
/// 2.4 MB is the entire residue.
///
/// Clearing the entry outright at Session teardown would release both sooner, and is worth doing for
/// the polynomial term; [`SsaShareGenerator::forget`] exists for it, but no production path calls it
/// today.
pub const MAX_RETAINED_COMMITMENT_CYCLES: usize = 18;

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

/// Protocol-wide floor on [`SsaReconstructorConfig::early_recovery_threshold`].
///
/// The Entry refuses a successor `SsaRequest` that arrives before the current batch can plausibly
/// have been recovered — see [`min_emission_for_early_recovery`]. That boundary is a function of the
/// *Exit's* `early_recovery_threshold`, and the two sides negotiate [`PixParams`] but not this: it is
/// a local reconstructor setting on the Exit, and the Entry computing the gate has no way to read it.
///
/// So the Entry computes the gate at this floor instead of at its own value, and every Exit is
/// required to sit at or above it. That turns an unshared value into a shared one without a wire
/// change, and it is sound in the only direction that matters — an Exit at or above the floor asks at
/// or after the point the Entry admits, so no conforming pair can fail to communicate. Deriving the
/// gate from the Entry's own setting instead left two individually valid configurations that could
/// not talk: an Exit configured lower sent its one-shot request early, the Entry silently dropped it,
/// and the Session died waiting for a commitment that was deliberately refused.
///
/// Equal to the shipped default today, but a separate constant on purpose. The default is a policy
/// choice an operator may raise; this is a compatibility bound, and *lowering* it is a protocol
/// break — every Entry still running the old value would refuse the earlier request. Raising it is
/// equally breaking in the other direction, since Exits at the old floor become unservable.
///
/// Enforced where the two halves meet rather than in [`SsaReconstructorConfig`]'s own validator: a
/// reconstructor exercised on its own has no peer to be incompatible with, and pinning the
/// early-recovery logic in a unit test needs thresholds well below anything deployable.
pub const MIN_EARLY_RECOVERY_THRESHOLD: f64 = 0.85;

// The shipped default must itself be admissible under the floor it is checked against. The two are
// defined independently and only compared at configuration time, so a default lowered below this
// would ship, validate every hand-written config, and then be rejected the moment an operator wrote
// the default out explicitly — or worse, be accepted locally and refused by every conforming Entry.
// Asserted here rather than only in a test because it costs nothing and cannot be skipped.
const _: () = assert!(
    SsaReconstructorConfig::DEFAULT_EARLY_RECOVERY_THRESHOLD >= MIN_EARLY_RECOVERY_THRESHOLD,
    "SsaReconstructorConfig::DEFAULT_EARLY_RECOVERY_THRESHOLD must not sit below MIN_EARLY_RECOVERY_THRESHOLD — a \
     stock Exit would ask for its next SSA batch before any conforming Entry admits the request"
);

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
/// fails to reconstruct implies a dishonest or broken Entry. The forward payload carrying its SURB
/// has no MAC, but every hop applies a wide-block PRP to it, so tampering in transit randomises the
/// whole SURB, acknowledgement challenge included. The share is decrypted only with an
/// acknowledgement that opens the challenge it is filed under, so a garbled SURB is a *lost* share,
/// never a decryptable corrupt one, and there is no benign path to the latter. Such an Entry
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
    use anyhow::Context;
    use hopr_types::{
        crypto::{
            crypto_traits,
            prelude::{ChainKeypair, Keypair, PublicKey, Secp256k1, SimplePseudonym},
        },
        primitive::prelude::Address,
    };
    use vsss_rs::{
        ParticipantIdGenerator, ReadableShareSet, ShareVerifierGroup,
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
            feldman::split_secret_with_participant_generators::<Share<S>, ShareVerifierGroup<PixGroup<S>>>(
                t,
                x.len(),
                &secret.into(),
                None,
                &mut rng,
                &[ParticipantIdGenerator::list(
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

    /// The tolerated loss must match values computed independently at 80 significant digits, by
    /// bisection to 2⁻⁶⁰, rather than the implementation's own `f64` arithmetic.
    ///
    /// The rows cover the deployed split, the one it replaced, the corners of the accepted range and
    /// the surplus-free splits, whose tolerances run down to 10⁻⁹ and are where a recurrence in linear
    /// space underflows. The relative bound is therefore also an absolute one of 10⁻¹² wherever the
    /// tolerance is below 10⁻⁶.
    #[test]
    fn tolerated_share_loss_matches_an_independent_high_precision_computation() -> anyhow::Result<()> {
        for (polys, threshold, surplus, expected) in [
            (4096u16, 32u8, 79u8, 0.5028832574f64),
            (8192, 64, 16, 0.05450787123),
            (8192, 64, 128, 0.5008618695),
            (4096, 64, 126, 0.5015851451),
            (8, 2, 2, 0.06917757351),
            (1, 2, 255, 0.9744513612),
            (8, 2, 0, 0.0006279487485),
            (128, 2, 0, 3.92583538e-05),
            (4096, 32, 0, 7.667797446e-08),
            (128, 255, 255, 0.4179011439),
            (16192, 255, 255, 0.3948700769),
            (16192, 255, 0, 2.434108308e-09),
            (16192, 2, 255, 0.9351652844),
            (16192, 2, 0, 3.103487615e-07),
            (16192, 255, 1, 4.362398113e-06),
            (8, 255, 255, 0.4343906026),
            // Splits an Entry distrusting its return relayers may choose to buy instead of the
            // deployed one, at quotas the default `quota_range` admits.
            (4096, 2, 102, 0.8589501692),
            (1024, 2, 255, 0.9459344036),
            (16192, 2, 26, 0.5347733964),
        ] {
            let tolerated = tolerated_share_loss(polys, threshold, surplus);
            assert!(
                (tolerated - expected).abs() <= 1e-6 * expected,
                "{polys} x ({threshold} + {surplus}) must tolerate {expected}, got {tolerated}"
            );
        }

        // The deployed split clears the 50 % it was derived for, and falls well short of its
        // break-even ratio — the distinction the Entry's successor gate depends on.
        let deployed =
            PixParams::try_new_for::<TestSpec>(DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD, DEFAULT_SURPLUS_SHARES)
                .context("the deployed split must be valid")?;
        let tolerated = deployed.tolerated_share_loss();
        let break_even = DEFAULT_SURPLUS_SHARES as f64 / deployed.emitted_shares_per_poly() as f64;
        assert!(
            (0.5..0.51).contains(&tolerated),
            "the deployed split must tolerate just over half its shares being lost, got {tolerated}"
        );
        assert!(
            break_even - tolerated > 0.2,
            "the deployed split tolerates {tolerated}, which must sit well below its break-even {break_even}"
        );

        Ok(())
    }

    /// The log-space recovery must agree with the direct sum [`default_surplus_for`] is built on,
    /// wherever the two overlap — at half share loss.
    ///
    /// The two are independent implementations of the same probability, so agreement checks the
    /// log-sum-exp and the exponent rather than the bisection. The absolute floor is for the
    /// single-polynomial corners where a cycle is all but certain to fail: there both take one minus a
    /// failure tail within 10⁻¹⁰ of one, and keep few or none of the digits of a recovery that is
    /// itself far below the 99 % either is ever compared against.
    #[test]
    fn the_general_recovery_agrees_with_the_const_one_at_half_share_loss() {
        for polys in [1u16, 128, 4096, MAX_POLYS_PER_SSA] {
            for threshold in [MIN_POLY_THRESHOLD, 32, 128, MAX_POLY_THRESHOLD] {
                for surplus in [0u8, 1, 79, u8::MAX] {
                    let general = cycle_recovery_at_share_loss(polys, threshold, surplus, 0.5);
                    let half = cycle_recovery_at_half_share_loss(polys, threshold, surplus);
                    assert!(
                        (general - half).abs() <= 1e-9 * general.max(half) + 1e-12,
                        "{polys} x ({threshold} + {surplus}) recovers {general} in log space and {half} directly"
                    );
                }
            }
        }
    }

    /// Every surplus [`default_surplus_for`] derives must tolerate the half share loss it was derived
    /// for, read back through the other function.
    ///
    /// The two share the 99 % target, so this is what pins them to the same definition of
    /// "tolerates". Saturated surpluses are skipped: past a threshold of ~160 the byte cannot carry
    /// what the target needs, and the derivation says so by returning its maximum.
    #[test]
    fn every_derived_surplus_tolerates_half_its_shares_being_lost() {
        for threshold in MIN_POLY_THRESHOLD..=MAX_POLY_THRESHOLD {
            let surplus = default_surplus_for(DEFAULT_POLYS_PER_SSA, threshold);
            if surplus == u8::MAX {
                continue;
            }
            let tolerated = tolerated_share_loss(DEFAULT_POLYS_PER_SSA, threshold, surplus);
            assert!(
                tolerated >= 0.5,
                "{DEFAULT_POLYS_PER_SSA} x ({threshold} + {surplus}) tolerates only {tolerated}"
            );
        }
    }

    /// More surplus never tolerates less, and more polynomials never tolerate more — each further
    /// polynomial is one more that has to survive.
    #[test]
    fn tolerated_share_loss_is_monotone_in_both_surplus_and_polynomials() {
        const POLYS: [u16; 6] = [1, 8, 128, 4096, 8192, MAX_POLYS_PER_SSA];

        for threshold in [MIN_POLY_THRESHOLD, 32, MAX_POLY_THRESHOLD] {
            // 0 to 255 inclusive: 255 is a multiple of 15.
            let surpluses = (0..=u8::MAX).step_by(15).collect::<Vec<_>>();

            for polys in POLYS {
                for pair in surpluses.windows(2) {
                    let (fewer, more) = (pair[0], pair[1]);
                    assert!(
                        tolerated_share_loss(polys, threshold, more) >= tolerated_share_loss(polys, threshold, fewer),
                        "{polys} x {threshold}: surplus {more} tolerates less than {fewer}"
                    );
                }
            }

            for surplus in [0u8, 79, u8::MAX] {
                for pair in POLYS.windows(2) {
                    let (fewer, more) = (pair[0], pair[1]);
                    assert!(
                        tolerated_share_loss(more, threshold, surplus)
                            <= tolerated_share_loss(fewer, threshold, surplus),
                        "({threshold} + {surplus}): {more} polynomials tolerate more than {fewer}"
                    );
                }
            }
        }
    }

    /// Every corner of the accepted range yields a probability, and the degenerate inputs outside it
    /// yield the one they mean rather than a panic or a NaN.
    #[test]
    fn tolerated_share_loss_is_a_probability_across_the_whole_range() {
        for polys in [1u16, MAX_POLYS_PER_SSA] {
            for threshold in [MIN_POLY_THRESHOLD, MAX_POLY_THRESHOLD] {
                for surplus in [0u8, u8::MAX] {
                    let tolerated = tolerated_share_loss(polys, threshold, surplus);
                    assert!(
                        tolerated.is_finite() && (0.0..=1.0).contains(&tolerated),
                        "{polys} x ({threshold} + {surplus}) tolerates {tolerated}"
                    );
                }
            }
        }

        // Nothing to recover, or nothing needed to recover it: any loss is survived.
        assert_eq!(
            1.0,
            tolerated_share_loss(0, DEFAULT_POLY_THRESHOLD, DEFAULT_SURPLUS_SHARES)
        );
        assert_eq!(1.0, tolerated_share_loss(DEFAULT_POLYS_PER_SSA, 0, 0));
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
