//! Criterion benchmarks for the Exit-side PIX supervisor state machine.
//!
//! Run with: `cargo bench -p hopr-transport-session --features benchmark -- supervisor_bench`
//!
//! ## Why this path and not the others
//!
//! Every other input the supervisor takes is a lifecycle transition: a request goes out, a
//! commitment verifies, a deposit confirms, a cycle recovers. Those happen a handful of times per
//! cycle, and a cycle is minutes long.
//!
//! `SessionPixEvent::RecoveryProgress` is the exception. It arrives **once per acknowledgement
//! batch**, so its rate is set by traffic rather than by the lifecycle — which is exactly why the
//! worker delivers it without backpressure and drops it when the channel is full. At the modelled
//! Exit envelope (30 Sessions × 20 Mbps, ~2 400 shares/s each, ~10 acknowledgements per packet)
//! that is on the order of 7 000 events/s node-wide, every one of which steps this state machine
//! and re-syncs the service gate. Nothing measured it before.
//!
//! ## What the four ids separate
//!
//! `on_recovery_progress` has three exits, and which one a snapshot takes is decided by counters the
//! Entry controls, not by configuration:
//!
//! * **`funded_front`** — the steady state. `shares_seen` and `useful_shares` both advance on the cycle the gate is
//!   charged against, so the payment tier runs (share-order booking included) and the call returns
//!   `ProgressNotification`, which allocates.
//! * **`surplus_only`** — the tail of every conforming cycle. `shares_seen` advances but `useful_shares` does not,
//!   because the polynomial those shares belong to is already complete. Same liveness work, same allocation, no payment
//!   tier.
//! * **`stale`** — a snapshot reordered behind one already seen. The relay-as-Exit pipeline processes acknowledgement
//!   batches concurrently, so this is ordinary traffic, not an error path. Returns before mutating anything, and should
//!   therefore be much the cheapest id here.
//! * **`off_front`** — progress on a cycle queued behind the front of a batch. Exercises the `find_ssa_idx` and
//!   `earliest_live_idx` linear scans over a real batch, and books against the off-front share-order counter. Returns
//!   nothing, because only the funded front may buy service.
//!
//! ## Measurement shape
//!
//! Each criterion iteration times a block of [`EVENTS_PER_SAMPLE`] events rather than a single one.
//! A `handle_event` call is on the order of a hundred nanoseconds, so bracketing each one in
//! `Instant::now()` would measure the clock as much as the state machine. Reporting
//! `Throughput::Elements` over the block makes the `thrpt` column read directly in events/s, which
//! is the unit the ~7 000/s figure above is in.
//!
//! Both the fixture and the block's events are built **outside** the timed span, in the manner of
//! `SsaReconstructor::acknowledge_shares/deferred` in `hopr-protocol-pix`. The fixture is rebuilt
//! whenever the next block would carry `useful_shares` past the cycle's target, since a supervisor
//! is a one-way state machine and a cycle that reaches its target is no longer the shape under test.

use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_api::{
    HoprBalance,
    types::{crypto_random::Randomizable, internal::prelude::HoprPseudonym},
};
use hopr_protocol_pix::{SsaId, SsaIndex, SsaRecoveryProgress};
use hopr_transport_session::{
    DEFAULT_PIX_POLYS_PER_SSA, DEFAULT_PIX_SHARES_PER_POLY, DEFAULT_PIX_SURPLUS_SHARES, LOCAL_PIX_SUITE, PixParams,
    SessionPixAction, SessionPixEvent, SessionPixSupervisor, SupervisorConfig,
};

// Avoid musl's default allocator due to degraded performance.
//
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(feature = "allocator-mimalloc", feature = "allocator-jemalloc"))]
compile_error!("feature \"allocator-jemalloc\" and feature \"allocator-mimalloc\" cannot be enabled at the same time");
#[cfg(all(target_os = "linux", feature = "allocator-mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(target_os = "linux", feature = "allocator-jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Events timed per criterion iteration.
///
/// Large enough that one `Instant::now()` pair is negligible against the block, small enough that a
/// production-width cycle absorbs hundreds of blocks before the fixture has to be rebuilt.
const EVENTS_PER_SAMPLE: u64 = 1024;

/// SSAs per request for the batched id.
///
/// The shipping default is 1, which has no queued cycle and so cannot reach the off-front branch at
/// all. 3 is the top of the deployed 2–3 range and the batch the memory profile models.
const BATCH: u32 = 3;

/// The deployed dimensions, taken from the crate's own constants so they cannot drift.
///
/// The width matters here only through `target_useful_shares()`, which is what decides how many
/// blocks a fixture survives — but taking the real value keeps that number honest rather than
/// convenient.
fn dims() -> PixParams {
    PixParams::try_new(
        DEFAULT_PIX_POLYS_PER_SSA,
        DEFAULT_PIX_SHARES_PER_POLY,
        DEFAULT_PIX_SURPLUS_SHARES,
        LOCAL_PIX_SUITE,
    )
    .expect("the deployed dimensions must be valid")
}

/// A supervisor with every cycle of its first batch funded, plus the ids it was funded with.
struct Fixture {
    supervisor: SessionPixSupervisor,
    ssa_ids: Vec<SsaId<HoprPseudonym>>,
    /// Fixed for the whole benchmark, so no deadline can expire mid-run and no `Instant::now()`
    /// lands in the timed span. The supervisor takes its clock as an argument precisely so a caller
    /// can do this.
    now: Instant,
    target_useful_shares: u64,
}

/// Drives a fresh supervisor's whole first batch to `Recovering`.
///
/// The three events per cycle are the real sequence — the request reaches the wire, the Entry's
/// commitment verifies, the deposit confirms — because there is no other way in: the phases are
/// private and the transitions are what enforce them.
fn funded_fixture(cfg: SupervisorConfig, batch: u32) -> Fixture {
    let params = dims();
    let pseudonym = HoprPseudonym::random();
    let now = Instant::now();
    let (mut supervisor, _) = SessionPixSupervisor::new(cfg, params, pseudonym, now);

    let ssa_ids: Vec<_> = (1..=batch)
        .map(|index| SsaId::new(pseudonym, SsaIndex::new(index).expect("ssa indices are one-based")))
        .collect();

    for ssa_id in &ssa_ids {
        supervisor.handle_event(&SessionPixEvent::SsaRequestSent(*ssa_id), now, 0);
        supervisor.handle_event(&SessionPixEvent::CommitmentVerified(*ssa_id), now, 0);
        supervisor.handle_event(
            &SessionPixEvent::DepositConfirmed {
                ssa_id: *ssa_id,
                amount: HoprBalance::new_base(1000),
            },
            now,
            0,
        );
    }

    Fixture {
        supervisor,
        ssa_ids,
        now,
        target_useful_shares: params.target_useful_shares(),
    }
}

fn progress(ssa_id: SsaId<HoprPseudonym>, useful: u64, seen: u64, target: u64) -> SessionPixEvent {
    SessionPixEvent::RecoveryProgress(SsaRecoveryProgress {
        ssa_id,
        useful_shares: useful,
        shares_seen: seen,
        target_useful_shares: target,
        recovered_polynomials: 0,
    })
}

/// Which branch of `on_recovery_progress` an id drives.
#[derive(Clone, Copy)]
enum Shape {
    FundedFront,
    SurplusOnly,
    Stale,
    OffFront,
}

impl Shape {
    fn id(self) -> &'static str {
        match self {
            Shape::FundedFront => "b1/funded_front",
            Shape::SurplusOnly => "b1/surplus_only",
            Shape::Stale => "b1/stale",
            Shape::OffFront => "b3/off_front",
        }
    }

    fn batch(self) -> u32 {
        match self {
            Shape::OffFront => BATCH,
            _ => 1,
        }
    }

    fn cfg(self) -> SupervisorConfig {
        let mut cfg = SupervisorConfig {
            ssas_per_request: self.batch() as usize,
            ..Default::default()
        };
        if matches!(self, Shape::OffFront) {
            // Every snapshot in this id lands off the front, so the share-order ratio is 1.0 by
            // construction and the shipped 0.25 would close the Session partway through the run —
            // measuring the close path instead of the scan. 1.0 is the documented "allow
            // everything" value for this dial, and the booking work still runs; only the verdict
            // changes.
            cfg.max_off_front_share_fraction = 1.0;
        }
        cfg
    }

    /// The cycle snapshots are addressed to: the front, except for the off-front id, which uses the
    /// last member of the batch.
    fn target_ssa(self, fixture: &Fixture) -> &SsaId<HoprPseudonym> {
        match self {
            Shape::OffFront => fixture.ssa_ids.last().expect("a batch is never empty"),
            _ => fixture.ssa_ids.first().expect("a batch is never empty"),
        }
    }

    /// Whether a snapshot of this shape must produce a `ProgressNotification`.
    ///
    /// Checked once per id before timing starts. Without it a refactor could move the benchmark onto
    /// a different branch of `on_recovery_progress` and the numbers would keep looking plausible.
    fn expects_notification(self) -> bool {
        matches!(self, Shape::FundedFront | Shape::SurplusOnly)
    }
}

/// Advances the counters one event's worth for the given shape.
///
/// `useful` is what bounds a fixture's life — it is the only counter with a ceiling — so it is
/// returned rather than hidden, and the caller uses it to decide when to rebuild.
fn advance(shape: Shape, useful: &mut u64, seen: &mut u64) {
    match shape {
        // Both tiers: the first `threshold` shares of each polynomial.
        Shape::FundedFront | Shape::OffFront => {
            *useful += 1;
            *seen += 1;
        }
        // Liveness only: the surplus run after a polynomial is already complete.
        Shape::SurplusOnly => *seen += 1,
        // Neither: a snapshot the state machine has already superseded.
        Shape::Stale => {}
    }
}

fn bench_handle_event(c: &mut Criterion) {
    let mut group = c.benchmark_group("SessionPixSupervisor::handle_event");
    group.throughput(Throughput::Elements(EVENTS_PER_SAMPLE));

    for shape in [Shape::FundedFront, Shape::SurplusOnly, Shape::Stale, Shape::OffFront] {
        // Pre-flight on a throwaway fixture: prove the fixture really reaches the branch this id
        // claims to measure, outside any timed region.
        {
            let mut probe = funded_fixture(shape.cfg(), shape.batch());
            let ssa_id = *shape.target_ssa(&probe);
            let target = probe.target_useful_shares;
            // Stale needs a predecessor to be stale *against*; the others are unaffected by it.
            probe
                .supervisor
                .handle_event(&progress(ssa_id, 1, 1, target), probe.now, 1);
            let (useful, seen) = match shape {
                Shape::Stale => (1, 1),
                _ => (2, 2),
            };
            let actions = probe
                .supervisor
                .handle_event(&progress(ssa_id, useful, seen, target), probe.now, seen);
            let notified = actions
                .iter()
                .any(|action| matches!(action, SessionPixAction::ProgressNotification));
            assert_eq!(
                notified,
                shape.expects_notification(),
                "{} does not reach the branch it claims to measure",
                shape.id()
            );
        }

        group.bench_function(BenchmarkId::from_parameter(shape.id()), |b| {
            let mut fixture = funded_fixture(shape.cfg(), shape.batch());
            let mut useful = 0u64;
            let mut seen = 0u64;
            // One event ahead of the block, so `Stale` has something to be stale against from the
            // very first timed call.
            advance(shape, &mut useful, &mut seen);
            if matches!(shape, Shape::Stale) {
                seen = 1;
                fixture.supervisor.handle_event(
                    &progress(*shape.target_ssa(&fixture), 0, seen, fixture.target_useful_shares),
                    fixture.now,
                    seen,
                );
            }

            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    // A cycle that reaches its target is no longer the shape under test, and the
                    // state machine cannot be wound back, so start a fresh Session instead.
                    if useful + EVENTS_PER_SAMPLE >= fixture.target_useful_shares {
                        fixture = funded_fixture(shape.cfg(), shape.batch());
                        useful = 0;
                        seen = 0;
                    }

                    let ssa_id = *shape.target_ssa(&fixture);
                    let target = fixture.target_useful_shares;
                    let served_base = seen;
                    let events: Vec<_> = (0..EVENTS_PER_SAMPLE)
                        .map(|_| {
                            advance(shape, &mut useful, &mut seen);
                            progress(ssa_id, useful, seen, target)
                        })
                        .collect();

                    let start = Instant::now();
                    for (i, event) in events.iter().enumerate() {
                        std::hint::black_box(fixture.supervisor.handle_event(
                            event,
                            fixture.now,
                            served_base + i as u64,
                        ));
                    }
                    total += start.elapsed();
                }
                total
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_handle_event);
criterion_main!(benches);
