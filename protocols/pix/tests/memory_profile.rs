//! Resident-memory profile of the Exit-side reconstructor at production dimensions.
//!
//! Complements the CPU benchmarks: `cargo bench` answers "how fast", this answers "how much
//! state does one Session's SSA cycle hold, and when does it peak".
//!
//! Run with:
//!
//! ```text
//! cargo test --release -p hopr-protocol-pix --test memory_profile -- --ignored --nocapture
//! ```
//!
//! `--release` matters: the point sizes below are the optimised layouts, and a debug build
//! takes tens of minutes to walk a production-width cycle.
//!
//! ## Why a tracking allocator rather than RSS
//!
//! Resident set size is a poor instrument here. `malloc` does not return freed pages to the
//! kernel promptly, so RSS reports a high-water mark that never comes back down and cannot
//! show the release-on-reconstruction decay this profile exists to measure. The tracking
//! allocator below reports *live* heap bytes, which is what an operator sizing an Exit needs.

mod common;

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

use common::TestSpec;
use hopr_protocol_pix::{
    CONSTANT_TERM_COEFFICIENT, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, EntryShareGenerator,
    ExitAcknowledgementShareProcessor, PixGroup, PixGroupRepr, PixScalar, ShareResolution, SsaGeneratorConfig, SsaId,
    SsaIndex, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator, TaggedEncryptedPartialSsaShare,
};
use hopr_types::{
    crypto::prelude::{HalfKey, Keypair, OffchainKeypair, SimplePseudonym},
    crypto_random::Randomizable,
    internal::prelude::{Acknowledgement, VerifiedAcknowledgement},
};

/// Deployed dimensions, mirroring `DEFAULT_PIX_POLYS_PER_SSA` / `DEFAULT_PIX_SHARES_PER_POLY`
/// in `transport/session/src/types.rs`.
const PROD_POLYS_PER_SSA: u16 = DEFAULT_POLYS_PER_SSA;
const PROD_THRESHOLD: u8 = DEFAULT_POLY_THRESHOLD;

/// Mirrors `MIN_COMMITMENTS_PER_SSA_COMMIT_MSG` in `transport/session/src/manager.rs`.
const COMMITMENTS_PER_SSA_COMMIT_MSG: usize = 28;

/// Mirrors `HoprPacket::PAYLOAD_SIZE`.
const QUOTA_BYTES_PER_SHARE: u64 = 1038;

/// Operating point being modelled: per-Session return-path rate, in bytes per second.
const RETURN_RATE_BYTES_PER_SEC: f64 = 1_500_000.0 / 8.0;

/// Concurrent Sessions per Exit that the profile is extrapolated to.
const SESSIONS_PER_EXIT: usize = 100;

/// Acknowledgements per `acknowledge_shares` call.
///
/// Production shape: the Exit ack pipeline calls it once per received acknowledgement packet,
/// which holds at most `MAX_ACKNOWLEDGEMENTS_BATCH_SIZE`
/// (`protocols/hopr/src/codec/encoder.rs`). Feeding a quarter-cycle in one call instead would
/// allocate hundreds of megabytes of transient intermediates inside
/// `verify_expected_acknowledgements`, and the high-water mark would then measure the test
/// harness rather than the reconstructor.
const ACK_BATCH: usize = 10;

struct TrackingAllocator;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            record_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            record_alloc(new_size);
        }
        new_ptr
    }
}

fn record_alloc(size: usize) {
    let live = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    PEAK.fetch_max(live, Ordering::Relaxed);
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

fn live_bytes() -> usize {
    LIVE.load(Ordering::Relaxed)
}

fn mib(bytes: usize) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

/// Prints one measurement point relative to the baseline taken before the reconstructor existed.
///
/// The delta is signed on purpose: past the midpoint of a cycle it goes *negative*, because the
/// Entry-side generator frees more of its polynomial queue than the Exit still holds. Clamping
/// that to zero would hide the confound rather than expose it.
fn report(label: &str, baseline: usize) {
    let live = live_bytes();
    println!(
        "  {label:<46} {:>9.1} MiB live  ({:>+9.1} MiB vs baseline)",
        mib(live),
        mib(live) - mib(baseline)
    );
}

/// Produces `count` shares, caches them as awaited encrypted shares, and returns the
/// acknowledgements that redeem them.
fn stage_shares(
    reconstructor: &SsaReconstructor<TestSpec>,
    generator: &SsaShareGenerator<TestSpec>,
    peer: &OffchainKeypair,
    pseudonym: SimplePseudonym,
    counter: &mut u64,
    count: usize,
) -> Vec<Acknowledgement> {
    let mut acks = Vec::with_capacity(count);
    for _ in 0..count {
        let msg = counter.to_be_bytes();
        *counter += 1;
        let share = generator
            .next_share(&pseudonym, &msg)
            .unwrap()
            .expect("generator must not be exhausted");
        let ack = HalfKey::random();
        let ack_challenge = ack.to_challenge().unwrap();
        let enc_share = share.share.encrypt(&share.id, &ack).unwrap();
        reconstructor
            .insert_encrypted_share(
                peer.public(),
                ack_challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, &msg, enc_share).unwrap(),
            )
            .unwrap();
        acks.push(VerifiedAcknowledgement::new(ack, peer).leak());
    }
    acks
}

/// Walks one production-width SSA cycle and reports live heap at each phase.
///
/// Ignored by default: it holds a full production commitment matrix and walks every share of a
/// cycle, so it runs for several minutes and allocates hundreds of megabytes.
#[test]
#[ignore]
fn exit_reconstructor_memory_profile_at_production_dimensions() {
    let polys = PROD_POLYS_PER_SSA as usize;
    let threshold = PROD_THRESHOLD as usize;
    let commitments = polys * threshold;
    let quota_bytes = commitments as u64 * QUOTA_BYTES_PER_SHARE;
    let cycle_secs = quota_bytes as f64 / RETURN_RATE_BYTES_PER_SEC;

    println!("\n=== Operating point ===");
    println!("  polynomials x threshold          {polys} x {threshold} = {commitments} commitments");
    println!(
        "  quota per cycle                  {:.1} MiB",
        mib(quota_bytes as usize)
    );
    println!(
        "  per-Session return rate          {:.2} Mbps ({:.0} shares/s)",
        RETURN_RATE_BYTES_PER_SEC * 8.0 / 1e6,
        RETURN_RATE_BYTES_PER_SEC / QUOTA_BYTES_PER_SHARE as f64
    );
    println!(
        "  cycle duration                   {:.0} s ({:.1} min)",
        cycle_secs,
        cycle_secs / 60.0
    );
    println!(
        "  cycles across {SESSIONS_PER_EXIT} Sessions        one every {:.1} s",
        cycle_secs / SESSIONS_PER_EXIT as f64
    );

    println!("\n=== Type sizes ===");
    println!(
        "  PixGroup     (decoded point)     {:>4} B",
        size_of::<PixGroup<TestSpec>>()
    );
    println!(
        "  PixGroupRepr (wire form)         {:>4} B",
        size_of::<PixGroupRepr<TestSpec>>()
    );
    println!(
        "  PixScalar                        {:>4} B",
        size_of::<PixScalar<TestSpec>>()
    );
    println!(
        "  commitment matrix, decoded       {:>9.1} MiB ({commitments} x {} B)",
        mib(commitments * size_of::<PixGroup<TestSpec>>()),
        size_of::<PixGroup<TestSpec>>()
    );

    // Generate the Entry-side cycle outside the measured region, then drop everything that is
    // not part of the Exit's state before taking the baseline.
    // `surplus_shares: 0` so the generator emits exactly `threshold` shares per polynomial and
    // `polys * threshold` acknowledgements walk the cycle to full recovery. Surplus does not
    // affect the peak — that occurs at commitment install, before any share arrives — but it
    // would leave the tail of the cycle unrecovered and make the endpoint meaningless.
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        threshold: PROD_THRESHOLD,
        polynomials_per_ssa: PROD_POLYS_PER_SSA,
        surplus_shares: 0,
    });
    let pseudonym = SimplePseudonym::random();
    let peer = OffchainKeypair::random();
    let commitment = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap();
    let commitment_proof = commitment.commitment_proof;
    let mut constant_terms = commitment
        .verifiers
        .get(&CONSTANT_TERM_COEFFICIENT)
        .cloned()
        .unwrap_or_default();
    constant_terms.sort_unstable_by_key(|(poly_index, _)| *poly_index);
    assert_eq!(polys, constant_terms.len());

    let baseline = live_bytes();
    // Re-arm the high-water mark so "peak" means the Exit's peak, not the Entry-side
    // `new_ssa_commitment` transient that dominates the process maximum.
    PEAK.store(baseline, Ordering::Relaxed);
    println!("\n=== Exit reconstructor, one Session, one cycle ===");
    println!("  (baseline excludes the Entry-side generator and the wire-form matrix)");
    println!(
        "  baseline                                       {:>9.1} MiB",
        mib(baseline)
    );

    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        // Stretched so nothing expires mid-profile; the point here is the size of live state,
        // not the reclamation schedule.
        max_ack_await_time: std::time::Duration::from_secs(7200),
        incomplete_commitment_lifetime: std::time::Duration::from_secs(7200),
        unused_verifier_lifetime: std::time::Duration::from_secs(7200),
        ..Default::default()
    });
    let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
    reconstructor.new_exit_commitment(ssa_id, polys, threshold).unwrap();
    report("after new_exit_commitment", baseline);

    // The whole wire order: one constant term per polynomial. The closing message publishes the
    // SSA commitment, the part accumulator and every part builder at once, so this is the
    // expected peak. At the modelled line rate it lands inside the first fraction of a percent
    // of the cycle: ~0.3 MiB of commitments against 519 MiB of payload.
    for chunk in constant_terms.chunks(COMMITMENTS_PER_SSA_COMMIT_MSG) {
        reconstructor
            .insert_coefficient_commitments(ssa_id, 0, Some(commitment_proof), chunk.iter().copied())
            .unwrap();
    }
    let after_install = live_bytes();
    report("after the constant-term pass (all verifiers)", baseline);

    // Drain the cycle. Shares are produced in polynomial order, so parts complete progressively
    // and Tier 1 releases each verifier on reconstruction — the decay this profile is for.
    let mut counter: u64 = 0;
    let mut recovered = false;
    let quarter = commitments / 4;
    for q in 1..=4 {
        let mut fed = 0;
        while fed < quarter {
            let n = ACK_BATCH.min(quarter - fed);
            let acks = stage_shares(&reconstructor, &generator, &peer, pseudonym, &mut counter, n);
            let resolutions = reconstructor.acknowledge_shares(*peer.public(), acks).unwrap();
            recovered |= resolutions
                .iter()
                .any(|r| matches!(r, ShareResolution::RecoveredSsa(_)));
            fed += n;
        }
        report(&format!("after {}% of the cycle's shares", q * 25), baseline);
    }

    // The high-water mark was re-armed at the baseline, and acknowledgements are fed in
    // production-shaped batches, so this is the reconstructor's own peak rather than a
    // harness artefact.
    let peak_over_baseline = PEAK.load(Ordering::Relaxed).saturating_sub(baseline);
    let install_over_baseline = after_install.saturating_sub(baseline);

    println!("\n=== Extrapolation to {SESSIONS_PER_EXIT} Sessions ===");
    println!(
        "  peak live state, 1 Session       {:>9.1} MiB",
        mib(peak_over_baseline)
    );
    println!(
        "  at commitment install, 1 Session {:>9.1} MiB",
        mib(install_over_baseline)
    );
    println!(
        "  x{SESSIONS_PER_EXIT} Sessions, all in phase        {:>9.2} GiB",
        mib(peak_over_baseline * SESSIONS_PER_EXIT) / 1024.0
    );
    println!(
        "  x{SESSIONS_PER_EXIT} Sessions, uniformly staggered {:>9.2} GiB",
        mib(install_over_baseline * SESSIONS_PER_EXIT / 2) / 1024.0
    );
    println!(
        "\n  Staggered assumes the live verifier set decays linearly from install to recovery,\n  so the mean across \
         uniformly-phased Sessions is half the post-install figure. Cycles do\n  not stay staggered after an Exit \
         restart, when every Session re-establishes at once.\n\n  CAVEAT on the intermediate decay points: the \
         Entry-side generator pops each polynomial\n  off its queue as it is exhausted, freeing memory in the same \
         process, so those readings\n  go negative against the baseline and understate the Exit's remaining live \
         state. The\n  install figure is clean — no share has been consumed at that point — and so is the\n  \
         endpoint, which is what makes the return to baseline a meaningful leak check.\n"
    );

    assert!(recovered, "a production-width cycle must recover the SSA");
    assert!(
        peak_over_baseline >= install_over_baseline,
        "the peak cannot be below the commitment-install figure"
    );
}
