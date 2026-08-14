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
    AWAITING_ACK_ENTRY_BYTES, CONSTANT_TERM_COEFFICIENT, DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA,
    EntryShareGenerator, ExitAcknowledgementShareProcessor, PixGroup, PixGroupRepr, PixParams, PixScalar,
    ShareResolution, SsaGeneratorConfig, SsaId, SsaIndex, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator,
    TOMBSTONE_ENTRY_BYTES, TaggedEncryptedPartialSsaShare, peak_cycle_bytes,
};
use hopr_types::{
    crypto::prelude::{HalfKey, HalfKeyChallenge, Keypair, OffchainKeypair, SimplePseudonym},
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

/// Surplus shares emitted per polynomial beyond the threshold.
///
/// Derived from the threshold — a ratio sized to absorb 20 % share loss — rather than a flat count.
/// The cycle is `polys × (threshold + surplus)` shares long, so this decides how much of the
/// profile's wall clock is spent walking one, and a flat value would mean a different cycle length
/// relative to the threshold at every dimension.
const PROD_SURPLUS: u8 = hopr_protocol_pix::default_surplus_for(PROD_THRESHOLD);

/// Operating point being modelled: per-Session return-path rate, in bytes per second.
///
/// 20 Mbps, the top of the deployed 16–20 Mbps range. This was 1.5 Mbps — **13× low** — which made
/// the modelled cycle thirteen times longer than a real one and, combined with the Session count
/// below, put the modelled Exit at 18.75 MiB/s against a real 20–75 MiB/s.
const RETURN_RATE_BYTES_PER_SEC: f64 = 20_000_000.0 / 8.0;

/// Concurrent Sessions per Exit that the profile is extrapolated to.
///
/// 30, the top of the deployed 10–30 range; it was 100.
const SESSIONS_PER_EXIT: usize = 30;

/// SSAs a Session holds in flight at once.
///
/// The Exit requests deposits in batches of 2–3, so that many cycles are live per Session
/// simultaneously — each with its own commitment set, part builders and awaited shares. The profile
/// walks one cycle and multiplies, so this is the factor that was silently 1.
const SSAS_IN_FLIGHT: usize = 3;

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

/// Measures what one entry in the `awaiting_acks` buffer costs in live heap.
///
/// This is the number `hopr-transport`'s `PixReconstructorConfig` budget is denominated in, and it
/// cannot be read off `size_of` alone: the payload is ~80 B of inline arrays, and moka's per-entry
/// bookkeeping — hash map entry, LRU node, TTL timer-wheel node, the `Arc` around the value —
/// dominates it. Rather than guess a multiple, measure, and let the validation constant cite this.
///
/// The occupancy is reported at two points rather than one because moka grows its internal
/// structures in chunks; a single reading at an unlucky occupancy would report the chunk rather
/// than the entry. One share is built before the baseline and copied in —
/// `TaggedEncryptedPartialSsaShare` is `Copy`, so nothing the loop does allocates and the delta is
/// the cache and nothing else.
///
/// Ignored by default: it holds ~100 000 live cache entries.
#[test]
#[ignore]
fn awaiting_ack_entry_cost() {
    /// Occupancy points to report. The last is the figure to quote.
    const POINTS: [usize; 2] = [20_000, 100_000];

    let entries = POINTS[POINTS.len() - 1];

    println!("\n=== Type sizes ===");
    println!(
        "  HalfKeyChallenge (key)           {:>4} B",
        size_of::<HalfKeyChallenge>()
    );
    println!(
        "  TaggedEncryptedPartialSsaShare   {:>4} B",
        size_of::<TaggedEncryptedPartialSsaShare<TestSpec>>()
    );
    let payload = size_of::<HalfKeyChallenge>() + size_of::<TaggedEncryptedPartialSsaShare<TestSpec>>();
    println!("  payload per entry                {payload:>4} B");

    // One polynomial of two shares is enough: the entries are distinguished by their keys, and the
    // value is the same shape whichever share it holds.
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        threshold: 2,
        polynomials_per_ssa: 1,
        surplus_shares: 0,
    });
    let pseudonym = SimplePseudonym::random();
    let peer = OffchainKeypair::random();
    let _ = generator.new_ssa_commitment(&pseudonym, SsaIndex::MIN).unwrap();
    let share = generator.next_share(&pseudonym, b"nonce").unwrap().unwrap();
    let template = {
        let ack = HalfKey::random();
        let enc_share = share.share.encrypt(&share.id, &ack).unwrap();
        TaggedEncryptedPartialSsaShare::new(pseudonym, b"nonce", enc_share).unwrap()
    };

    // Keys built up front: `HalfKey::to_challenge` is a scalar multiplication, and doing 100 000 of
    // them inside the measured region would put the transient curve state in the reading.
    let challenges = (0..entries)
        .map(|_| HalfKey::random().to_challenge().unwrap())
        .collect::<Vec<_>>();

    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        // Above the largest occupancy point, so nothing is size-evicted, and long enough that
        // nothing expires mid-measurement. Both would make the average report a smaller cache.
        max_awaiting_acks: entries * 2,
        max_ack_await_time: std::time::Duration::from_secs(7200),
        // Exactly the occupancy being measured, so the run doubles as a check on the global budget:
        // fill it to the ceiling and the live heap must land inside the byte figure that ceiling
        // was denominated in.
        max_ack_buffer_bytes: entries * AWAITING_ACK_ENTRY_BYTES,
        ..Default::default()
    });

    let baseline = live_bytes();
    println!("\n=== Awaiting-ack buffer, one peer ===");
    let mut inserted = 0;
    for point in POINTS {
        while inserted < point {
            reconstructor
                .insert_encrypted_share(peer.public(), challenges[inserted], template)
                .unwrap();
            inserted += 1;
        }
        // moka applies writes from an internal queue during maintenance. Queued or applied, the
        // bytes are live either way, so the total is right regardless — this only makes the split
        // between the two stable enough that the two points are comparable.
        std::thread::sleep(std::time::Duration::from_millis(500));

        let held = live_bytes().saturating_sub(baseline);
        println!(
            "  {point:>7} entries                  {:>9.1} MiB live   {:>4} B/entry  ({:>4} B overhead)",
            mib(held),
            held / point,
            (held / point).saturating_sub(payload)
        );
    }

    // The reconstructor is now exactly at its configured budget, so the next share must be refused.
    // That is the whole claim `AWAITING_ACK_ENTRY_BYTES` exists to support: a ceiling counted in
    // entries is a ceiling in bytes.
    let budget = entries * AWAITING_ACK_ENTRY_BYTES;
    let held = live_bytes().saturating_sub(baseline);
    let overrun =
        reconstructor.insert_encrypted_share(peer.public(), HalfKey::random().to_challenge().unwrap(), template);
    assert!(
        overrun.is_err(),
        "a buffer filled to max_ack_buffer_bytes must refuse the next share"
    );
    assert!(
        held <= budget,
        "live heap {held} B exceeds the {budget} B the same occupancy was budgeted at — AWAITING_ACK_ENTRY_BYTES \
         ({AWAITING_ACK_ENTRY_BYTES} B) is understated at {} B/entry",
        held / entries
    );
    println!(
        "  at the configured ceiling         {:>9.1} MiB live against a {:.1} MiB budget ({:.0}% of it)",
        mib(held),
        mib(budget),
        100.0 * held as f64 / budget as f64
    );

    println!(
        "\n  Quote the last row as AWAITING_ACK_ENTRY_BYTES in `hopr-protocol-pix`. Round up: the\n  budget is a \
         ceiling, so under-stating the per-entry cost lets it be exceeded.\n"
    );
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
    // A cycle emits `threshold + surplus` shares per polynomial, and the quota counts every one of
    // them — `pix_params_to_quota` in `transport/session/src/types.rs` includes the surplus because
    // H5 established that it is billed on purchase rather than on claim. This used to be
    // `commitments * QUOTA_BYTES_PER_SHARE`, i.e. threshold only, understating the cycle by 31 % at
    // the deployed surplus and mis-stating the duration by the same factor.
    let emitted_shares = polys * (threshold + PROD_SURPLUS as usize);
    let quota_bytes = emitted_shares as u64 * QUOTA_BYTES_PER_SHARE;
    let cycle_secs = quota_bytes as f64 / RETURN_RATE_BYTES_PER_SEC;

    println!("\n=== Operating point ===");
    println!("  polynomials x threshold          {polys} x {threshold} = {commitments} commitments");
    println!(
        "  emitted shares per cycle         {emitted_shares} (threshold {threshold} + surplus {PROD_SURPLUS}, factor \
         {:.2}x)",
        (threshold + PROD_SURPLUS as usize) as f64 / threshold as f64
    );
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
    reconstructor
        .new_exit_commitment(
            ssa_id,
            PixParams::try_from_config::<TestSpec>(generator.config()).unwrap(),
        )
        .unwrap();
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

    // An Exit holds `SSAS_IN_FLIGHT` cycles per Session, not one: deposits are requested in
    // batches, so several cycles are live at once, each with its own commitment set, part builders
    // and awaited shares. This profile walks a single cycle, so the batch is a multiplier on
    // everything below — and it used to be missing entirely.
    let cycles = SSAS_IN_FLIGHT * SESSIONS_PER_EXIT;

    println!("\n=== Extrapolation to {SESSIONS_PER_EXIT} Sessions x {SSAS_IN_FLIGHT} SSAs in flight ===");
    println!(
        "  peak live state, 1 cycle         {:>9.1} MiB",
        mib(peak_over_baseline)
    );
    println!(
        "  at commitment install, 1 cycle   {:>9.1} MiB",
        mib(install_over_baseline)
    );
    // The per-polynomial cost with an empty share buffer, which is what `peak_cycle_bytes` models
    // for every polynomial of a cycle before any share arrives.
    let modelled_per_poly =
        peak_cycle_bytes::<TestSpec>(&PixParams::try_from_config::<TestSpec>(generator.config()).unwrap()) as usize
            / polys
            - threshold.next_power_of_two() * size_of::<PixScalar<TestSpec>>() * 2;
    println!(
        "  per polynomial at install        {:>9} B  (modelled as {modelled_per_poly} B)",
        install_over_baseline / polys
    );
    println!(
        "  per Session ({SSAS_IN_FLIGHT} cycles)             {:>9.1} MiB at install",
        mib(install_over_baseline * SSAS_IN_FLIGHT)
    );
    println!(
        "  x{cycles} cycles, all in phase        {:>9.2} GiB",
        mib(peak_over_baseline * cycles) / 1024.0
    );
    println!(
        "  x{cycles} cycles, uniformly staggered {:>9.2} GiB",
        mib(install_over_baseline * cycles / 2) / 1024.0
    );
    println!(
        "\n  Staggered assumes the live verifier set decays linearly from install to recovery,\n  so the mean across \
         uniformly-phased cycles is half the post-install figure. Cycles do\n  not stay staggered after an Exit \
         restart, when every Session re-establishes at once.\n\n  Note which multiplier dominates: the batch \
         ({SSAS_IN_FLIGHT}x) and the Session count ({SESSIONS_PER_EXIT}x)\n  multiply, so a batch of 3 across 30 \
         clients is {cycles} concurrent cycles — the same order as the\n  100 Sessions this profile used to model \
         with no batch at all, reached by a different route.\n\n  CAVEAT on the intermediate decay points: the \
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
    // No share has been consumed at install, so this reading is the per-polynomial cost with an
    // empty share buffer — exactly the term `peak_cycle_bytes` adds for every polynomial before any
    // share arrives. It feeds the Session layer's live-cycle budget, so understating it lets that
    // budget be exceeded.
    assert!(
        install_over_baseline / polys <= modelled_per_poly,
        "the modelled per-polynomial cost ({modelled_per_poly} B) is understated at {} B — raise \
         PART_BUILDER_OVERHEAD_BYTES",
        install_over_baseline / polys
    );
}

/// Walks one production-width cycle in the **worst share order an Entry can choose**, and reports
/// the peak that order produces.
///
/// This is the figure `hopr_protocol_pix::peak_cycle_bytes` claims to bound, and therefore the one
/// the Session layer's live-cycle budget is denominated in. The sibling profile above feeds shares
/// polynomial-major, as the shipped generator emits them, so its peak is the *conforming* one — an
/// order of magnitude lower, and no bound at all on a peer running something else.
///
/// The order here holds every polynomial one share short of its threshold.
/// `SsaPartBuilder::release_verification_state` frees a share buffer when its polynomial
/// reconstructs, and a polynomial one share short never reconstructs, so every buffer in the cycle
/// stays live at once. Nothing in the protocol forbids it: the Entry decides which polynomial each
/// share belongs to, and this costs it the same quota either way.
///
/// Ignored by default, for the same reason as its sibling: it walks a full production-width cycle.
#[test]
#[ignore]
fn exit_reconstructor_worst_case_share_order() {
    let polys = PROD_POLYS_PER_SSA as usize;
    let threshold = PROD_THRESHOLD as usize;

    // `surplus_shares: 0`, so the generator emits exactly `threshold` shares per polynomial and the
    // withheld one below is the last of them. A surplus run would deliver the withheld share after
    // all, reconstruct the polynomial, and release the buffer this profile exists to measure.
    let generator = SsaShareGenerator::<TestSpec>::new(SsaGeneratorConfig {
        threshold: PROD_THRESHOLD,
        polynomials_per_ssa: PROD_POLYS_PER_SSA,
        surplus_shares: 0,
    });
    let params = PixParams::try_from_config::<TestSpec>(generator.config()).unwrap();
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

    // Emit the whole cycle **before** the baseline is taken, keeping only what will be delivered.
    //
    // Emission is round-robin over a window of polynomials, so the generator hands each polynomial's
    // shares out interleaved with its neighbours'; counting per polynomial and dropping the last one
    // it emits is what leaves every buffer one short. No reordering of the generator is needed —
    // only a choice about which shares are delivered, which is exactly the choice an Entry has.
    //
    // Staged up front rather than pulled inside the measured region because the generator frees each
    // polynomial as it exhausts it, in this same process. That release runs concurrently with the
    // Exit's accumulation and cancels most of it out: measured the other way round this cycle reads
    // as +7.5 MiB rather than the ~37 MiB it actually holds. It is the same confound the sibling
    // profile documents for its intermediate readings, and here it would swallow the whole result.
    let mut held = vec![0usize; polys];
    let mut staged = Vec::with_capacity(polys * (threshold - 1));
    for counter in 0..(polys * threshold) as u64 {
        let msg = counter.to_be_bytes();
        let share = generator
            .next_share(&pseudonym, &msg)
            .unwrap()
            .expect("generator must not be exhausted");

        let poly = usize::from(share.id.poly_index());
        if held[poly] + 1 >= threshold {
            // The share that would complete this polynomial: emitted, never delivered.
            continue;
        }
        held[poly] += 1;
        staged.push((msg, share));
    }
    // Nothing of the Entry's may be freed during the measurement.
    drop(generator);
    drop(held);

    let baseline = live_bytes();
    PEAK.store(baseline, Ordering::Relaxed);
    println!("\n=== Exit reconstructor, worst-case share order ===");
    println!("  {polys} polynomials x {} shares held, none completed", threshold - 1);
    println!(
        "  (baseline holds the {} staged shares, so only the Exit's state moves)",
        staged.len()
    );

    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        max_ack_await_time: std::time::Duration::from_secs(7200),
        incomplete_commitment_lifetime: std::time::Duration::from_secs(7200),
        unused_verifier_lifetime: std::time::Duration::from_secs(7200),
        ..Default::default()
    });
    let ssa_id = SsaId::new(pseudonym, SsaIndex::MIN);
    reconstructor.new_exit_commitment(ssa_id, params).unwrap();
    for chunk in constant_terms.chunks(COMMITMENTS_PER_SSA_COMMIT_MSG) {
        reconstructor
            .insert_coefficient_commitments(ssa_id, 0, Some(commitment_proof), chunk.iter().copied())
            .unwrap();
    }
    report("after the constant-term pass (all verifiers)", baseline);

    // Iterated by reference: consuming `staged` would free it as the Exit fills up and reintroduce
    // exactly the confound it was staged to avoid.
    let mut pending = Vec::with_capacity(ACK_BATCH);
    for (msg, share) in &staged {
        let ack = HalfKey::random();
        let ack_challenge = ack.to_challenge().unwrap();
        // Cloned rather than consumed: `staged` has to stay whole for the length of the measurement.
        let enc_share = share.share.clone().encrypt(&share.id, &ack).unwrap();
        reconstructor
            .insert_encrypted_share(
                peer.public(),
                ack_challenge,
                TaggedEncryptedPartialSsaShare::new(pseudonym, msg, enc_share).unwrap(),
            )
            .unwrap();
        pending.push(VerifiedAcknowledgement::new(ack, &peer).leak());

        if pending.len() == ACK_BATCH {
            let batch = std::mem::replace(&mut pending, Vec::with_capacity(ACK_BATCH));
            let resolutions = reconstructor.acknowledge_shares(*peer.public(), batch).unwrap();
            assert!(
                !resolutions
                    .iter()
                    .any(|r| matches!(r, ShareResolution::RecoveredSsa(_))),
                "no polynomial may complete under this order, so the SSA cannot recover"
            );
        }
    }
    if !pending.is_empty() {
        reconstructor.acknowledge_shares(*peer.public(), pending).unwrap();
    }
    let delivered = staged.len();

    let peak_over_baseline = PEAK.load(Ordering::Relaxed).saturating_sub(baseline);
    let modelled = peak_cycle_bytes::<TestSpec>(&params) as usize;
    report("with every polynomial one share short", baseline);

    println!(
        "\n  delivered                        {delivered} of {} emitted shares",
        polys * threshold
    );
    println!(
        "  worst-case peak, 1 cycle         {:>9.1} MiB",
        mib(peak_over_baseline)
    );
    println!(
        "  peak_cycle_bytes(params)         {:>9.1} MiB  ({:.0}% used)",
        mib(modelled),
        100.0 * peak_over_baseline as f64 / modelled as f64
    );
    println!(
        "  of which share buffers           {:>9.1} MiB  ({polys} x {} slots x {} B)",
        mib(polys * threshold.next_power_of_two() * size_of::<PixScalar<TestSpec>>() * 2),
        threshold.next_power_of_two(),
        size_of::<PixScalar<TestSpec>>() * 2
    );
    let buffer_slots = threshold.next_power_of_two() * size_of::<PixScalar<TestSpec>>() * 2;
    println!(
        "  per polynomial, modelled         {:>9} B     measured {:>9} B",
        modelled / polys,
        peak_over_baseline / polys
    );
    println!(
        "    minus share buffers            {:>9} B     measured {:>9} B  <- PART_BUILDER_OVERHEAD_BYTES covers the \
         gap",
        modelled / polys - buffer_slots,
        (peak_over_baseline / polys).saturating_sub(buffer_slots)
    );
    println!(
        "\n  Quote the model, not the measurement, in the Session layer's budget: the measurement is\n  one \
         allocator's answer at one set of dimensions, and the model is what a Session is\n  charged for the \
         dimensions its peer actually offered.\n"
    );

    assert_eq!(
        polys * (threshold - 1),
        delivered,
        "every polynomial must end one share short"
    );
    assert!(
        peak_over_baseline <= modelled,
        "peak_cycle_bytes ({modelled} B) is understated: the worst share order reached {peak_over_baseline} B"
    );
}

/// Measures what one entry in the retirement tombstone set costs in live heap.
///
/// Same instrument and same reason as [`awaiting_ack_entry_cost`]: `size_of` sees a ~20 B `SsaId`
/// key against a unit value, and moka's per-entry bookkeeping is the rest. `MAX_RETIRED_SSAS` is
/// denominated in this figure, and a tombstone evicted for want of capacity permits exactly the
/// resurrection it exists to prevent — so it is rounded up.
///
/// Ignored by default: it holds ~100 000 live cache entries.
#[test]
#[ignore]
fn tombstone_entry_cost() {
    const POINTS: [usize; 2] = [20_000, 100_000];

    let entries = POINTS[POINTS.len() - 1];
    println!("\n=== Type sizes ===");
    println!(
        "  SsaId (key)                      {:>4} B",
        size_of::<SsaId<SimplePseudonym>>()
    );

    let reconstructor = SsaReconstructor::<TestSpec>::new(SsaReconstructorConfig {
        // Far longer than the run, so nothing expires mid-measurement and understates the entry.
        unused_verifier_lifetime: std::time::Duration::from_secs(7200),
        ..Default::default()
    });

    // Distinct pseudonyms as well as distinct indices: an `SsaId` is the pair, and a real node
    // accumulates tombstones across Sessions rather than within one.
    let ids = (0..entries)
        .map(|i| SsaId::new(SimplePseudonym::random(), SsaIndex::new(1 + (i as u32 % 1024)).unwrap()))
        .collect::<Vec<_>>();

    let baseline = live_bytes();
    println!("\n=== Tombstone set ===");
    let mut inserted = 0;
    for point in POINTS {
        while inserted < point {
            reconstructor.retire_ssa(ids[inserted]);
            inserted += 1;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));

        let live = live_bytes().saturating_sub(baseline);
        println!(
            "  {point:>7} entries                  {:>9.1} MiB live   {:>4} B/entry",
            mib(live),
            live / point
        );
    }

    let live = live_bytes().saturating_sub(baseline);
    println!(
        "\n  Quote the last row as TOMBSTONE_ENTRY_BYTES. MAX_RETIRED_SSAS x that figure is the\n  ceiling the \
         Session layer's live-cycle budget has to leave room for.\n"
    );
    assert!(
        live / entries <= TOMBSTONE_ENTRY_BYTES,
        "TOMBSTONE_ENTRY_BYTES ({TOMBSTONE_ENTRY_BYTES} B) is understated at {} B/entry",
        live / entries
    );
}
