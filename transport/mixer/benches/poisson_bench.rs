//! Throughput of the shared-pool Poisson engine (`poisson_channel`), 1–10 MB/s.
//!
//! Same reused-channel methodology as `mixer_throughput_bench` (setup once, before timing), so
//! the numbers are directly comparable with the `mixer_poisson`/`mixer_channel` variants there.

use std::{cell::RefCell, rc::Rc};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_transport_mixer::poisson_channel;

mod common;
use common::{RANDOM_GIBBERISH, SAMPLE_SIZE, drain_one, minimal_delay_mixer_cfg, size_label, sizes};

pub fn mixer_poisson_throughput_reused(c: &mut Criterion) {
    let cfg = minimal_delay_mixer_cfg();
    // Built once, before any timing: runtime + shared-pool channel, reused across sizes/samples.
    let runtime = tokio::runtime::Runtime::new().expect("failed to create runtime");
    let (tx, rx) = poisson_channel::<&'static str>(cfg);
    let rx = Rc::new(RefCell::new(rx));

    let mut group = c.benchmark_group("mixer_throughput_reused");
    group.sample_size(SAMPLE_SIZE);
    for bytes in sizes() {
        let iterations = bytes / RANDOM_GIBBERISH.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_through_mixer_poisson", size_label(bytes))),
            &bytes,
            |b, _| {
                b.to_async(&runtime).iter_custom(|iters| {
                    let tx = tx.clone();
                    let rx = rx.clone();
                    async move {
                        // Preload: get a batch aging in the pool before timing starts, then keep
                        // sends and drains interleaved 1:1 through the timed region instead of
                        // bursting all sends before any drain. Bounded-latency mode's virtual
                        // clock advances from wall time alone, so a burst-then-drain round pays a
                        // near-fixed tail waiting on its own worst-case (highest-tag) packet once
                        // sending stops — a cost that's roughly independent of batch size, so it
                        // dwarfs per-message cost at small sizes and is merely amortized away at
                        // large ones. Interleaving keeps the pool continuously occupied so drains
                        // mostly find an already-aged item waiting, measuring steady-state
                        // throughput instead of that batch-boundary artifact.
                        for _ in 0..iterations {
                            tx.send(RANDOM_GIBBERISH).expect("send must succeed");
                        }

                        let start = std::time::Instant::now();
                        for _ in 0..iters {
                            for _ in 0..iterations {
                                tx.send(RANDOM_GIBBERISH).expect("send must succeed");
                                std::hint::black_box(drain_one(&rx).await.expect("receive must succeed"));
                            }
                        }
                        let elapsed = start.elapsed();

                        // Drain the untimed preload batch so the pool is empty again before the
                        // next sample/iteration, instead of growing unbounded across the run.
                        for _ in 0..iterations {
                            std::hint::black_box(drain_one(&rx).await.expect("receive must succeed"));
                        }

                        elapsed
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, mixer_poisson_throughput_reused);
criterion_main!(benches);
