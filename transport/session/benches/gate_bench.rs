//! Criterion benchmarks for the PIX egress gate's fast path.
//!
//! Run with: `cargo bench -p hopr-transport-session --features benchmark -- gate_bench`
//! And again with `--features benchmark,telemetry` for the before/after this exists to produce.
//!
//! ## Why this path
//!
//! [`ServiceGate::try_acquire_sync`] is on **every outgoing data packet of every supervised
//! Session**. At the modelled Exit envelope — 30 Sessions × 20 Mbps — that is tens of thousands of
//! calls a second on one node, which makes it the one place in the PIX telemetry work where a few
//! nanoseconds matter. Everything else added by the aggregate telemetry runs once per supervisor
//! turn or once per Session.
//!
//! The bounded-telemetry change touches this function twice, and the benchmark exists to price both:
//!
//! * `try_acquire_sync` now returns a [`GateVerdict`] rather than a `bool`. Two words instead of one, no branch and no
//!   allocation — expected to be free, but "expected to be free" is what a benchmark is for.
//! * the predeposit branch performs a second `fetch_add`, for the served split. The funded branch deliberately does
//!   not, which is why the two are measured separately rather than as one average.
//!
//! ## What the ids separate
//!
//! * **`funded`** — the steady state, and the one that carries essentially all of a Session's egress. A load, a
//!   compare-exchange on `served`, and the epoch RMW. Nothing was added here, so this id is the guard against having
//!   added something by accident.
//! * **`predeposit`** — an unfunded front spending its allowance. Two compare-exchanges and now two `fetch_add`s.
//!   Bounded by `max_predeposit_packets` per front rotation, so its share of real traffic is small, but it is the
//!   branch the split counter was added to.
//! * **`refused_share_lag`** — a funded gate at its ceiling. The path that now constructs a block episode in the
//!   caller; measured here without the episode, so the gate's own refusal cost is separated from the telemetry the
//!   manager wraps it in.
//! * **`refused_predeposit`** — an unfunded gate with nothing left. Same, on the other branch.
//! * **`contended`** — the same funded acquisition from several threads at once, which is how a Session with a deep
//!   egress buffer actually drives this. The compare-exchange retry loop is the thing being priced.
//!
//! ## Measurement shape
//!
//! Each iteration times a block of [`ACQUIRES_PER_SAMPLE`] calls rather than a single one: an
//! acquisition is on the order of ten nanoseconds, so bracketing each in `Instant::now()` would
//! measure the clock rather than the gate. `Throughput::Elements` over the block makes the `thrpt`
//! column read directly in packets/s.
//!
//! The admitting ids need a gate that can admit a whole block, so the fixture is rebuilt outside
//! the timed span whenever the next block would exhaust it — a gate is a one-way counter, and one
//! that has run out is the `refused_*` shape rather than the shape under test.

use std::{hint::black_box, sync::Arc};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_transport_session::{GateVerdict, ServiceGate};

/// Calls timed per criterion iteration.
const ACQUIRES_PER_SAMPLE: usize = 4_096;

/// Threads used by the `contended` id.
const CONTENDING_THREADS: usize = 4;

/// A ceiling no block can reach, for ids that must not be refused by it.
const UNREACHABLE_CEILING: u64 = u64::MAX;

#[derive(Clone, Copy)]
enum Shape {
    Funded,
    Predeposit,
    RefusedShareLag,
    RefusedPredeposit,
}

impl Shape {
    fn id(self) -> &'static str {
        match self {
            Self::Funded => "funded",
            Self::Predeposit => "predeposit",
            Self::RefusedShareLag => "refused_share_lag",
            Self::RefusedPredeposit => "refused_predeposit",
        }
    }

    /// A gate in this shape, able to serve a whole block without changing shape.
    fn gate(self) -> Arc<ServiceGate> {
        match self {
            Self::Funded => {
                let gate = ServiceGate::new(0, UNREACHABLE_CEILING);
                gate.release_service();
                gate
            }
            // Sized so one block fits: a budget that ran out mid-block would silently turn this id
            // into `refused_predeposit` for the remainder of it.
            Self::Predeposit => ServiceGate::new(ACQUIRES_PER_SAMPLE as u64, UNREACHABLE_CEILING),
            Self::RefusedShareLag => {
                // Funded with a ceiling of zero: every call takes the funded branch and is refused
                // by the ceiling, without any state changing.
                let gate = ServiceGate::new(0, 0);
                gate.release_service();
                gate
            }
            // Strict prepay and unfunded: refuses on the allowance, for ever.
            Self::RefusedPredeposit => ServiceGate::new(0, UNREACHABLE_CEILING),
        }
    }

    /// Whether a gate in this shape is spent after a block and must be rebuilt.
    fn is_consumable(self) -> bool {
        matches!(self, Self::Predeposit)
    }

    fn expected(self) -> bool {
        matches!(self, Self::Funded | Self::Predeposit)
    }
}

fn bench_try_acquire_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("gate_try_acquire_sync");
    group.throughput(Throughput::Elements(ACQUIRES_PER_SAMPLE as u64));

    for shape in [
        Shape::Funded,
        Shape::Predeposit,
        Shape::RefusedShareLag,
        Shape::RefusedPredeposit,
    ] {
        group.bench_function(BenchmarkId::from_parameter(shape.id()), |b| {
            // `iter_batched` rather than `iter`, so the rebuild a consumable shape needs is set-up
            // rather than measurement. The shapes that do not consume their gate reuse one, which
            // is what `Shape::gate` returning a fresh `Arc` each call would otherwise cost them.
            let reusable = (!shape.is_consumable()).then(|| shape.gate());
            b.iter_batched(
                || reusable.clone().unwrap_or_else(|| shape.gate()),
                |gate| {
                    for _ in 0..ACQUIRES_PER_SAMPLE {
                        let verdict = gate.try_acquire_sync().expect("a live gate is not poisoned");
                        debug_assert_eq!(shape.expected(), matches!(verdict, GateVerdict::Admitted));
                        black_box(verdict);
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// The same funded acquisition under contention, which is how a real Session drives it.
///
/// Separate from the ids above because the cost being measured is different: not the instruction
/// count of one call, but how often the compare-exchange on `served` is lost and retried. That is
/// what decides whether the gate is a bottleneck on a node running many Sessions at line rate.
fn bench_contended(c: &mut Criterion) {
    let mut group = c.benchmark_group("gate_try_acquire_sync_contended");
    group.throughput(Throughput::Elements((ACQUIRES_PER_SAMPLE * CONTENDING_THREADS) as u64));

    group.bench_function(BenchmarkId::from_parameter(CONTENDING_THREADS), |b| {
        let gate = ServiceGate::new(0, UNREACHABLE_CEILING);
        gate.release_service();

        b.iter(|| {
            // Threads are spawned inside the timed span deliberately: the alternative is a
            // long-lived pool parked on a barrier, and the barrier wake would be most of what a
            // block this short measured. The spawn cost is constant across runs, so a
            // before/after comparison — which is what this benchmark is for — is unaffected.
            std::thread::scope(|scope| {
                for _ in 0..CONTENDING_THREADS {
                    let gate = gate.clone();
                    scope.spawn(move || {
                        for _ in 0..ACQUIRES_PER_SAMPLE {
                            black_box(gate.try_acquire_sync().expect("a live gate is not poisoned"));
                        }
                    });
                }
            });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_try_acquire_sync, bench_contended);
criterion_main!(benches);
