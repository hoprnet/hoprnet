use std::{cell::RefCell, rc::Rc};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{SinkExt, StreamExt, future::BoxFuture};
use hopr_transport_mixer::{MixerSink, channel, config::MixerConfig, poisson_channel};
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;

mod common;
use common::{RANDOM_GIBBERISH, SAMPLE_SIZE, drain_one, minimal_delay_mixer_cfg, size_label, sizes};

pub fn mixer_throughput(
    c: &mut Criterion,
    cfg: MixerConfig,
    description: &str,
    sizes: &[usize],
    f: impl Fn(&'static str, usize, MixerConfig) -> BoxFuture<'static, ()>,
) {
    let mut group = c.benchmark_group("mixer_throughput");
    group.sample_size(SAMPLE_SIZE);
    for bytes in sizes {
        group.throughput(Throughput::Bytes(*bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!(
                "random_data_size_{}_through_{}",
                bytesize::ByteSize::b(*bytes as u64).to_string().replace(" ", "_"),
                description
            )),
            bytes,
            |b, _| {
                let runtime = tokio::runtime::Runtime::new().expect("failed to create runtime");

                b.to_async(runtime)
                    .iter(|| f(RANDOM_GIBBERISH, bytes / RANDOM_GIBBERISH.len(), cfg));
            },
        );
    }
    group.finish();
}

fn send_continuous_stream_load(item: &str, iterations: usize, _cfg: MixerConfig) -> BoxFuture<'_, ()> {
    Box::pin(async move {
        let (tx, rx) = futures::channel::mpsc::unbounded();

        // Baseline floor: a raw `then_concurrent` stream with no artificial hold, so it measures
        // the plumbing overhead alone — the counterpart of `minimal_delay_mixer_cfg`, whose
        // near-passthrough config the real mixer variants run against.
        let mut rx = rx.then_concurrent(
            |v| async move {
                tokio::time::sleep(std::time::Duration::ZERO).await;

                v
            },
            None,
        );

        for _ in 0..iterations {
            tx.unbounded_send(item).expect("send must succeed");
        }

        for _ in 0..iterations {
            rx.next().await.expect("receive must succeed");
        }
    })
}

fn send_continuous_sink_load(item: &'static str, iterations: usize, cfg: MixerConfig) -> BoxFuture<'static, ()> {
    Box::pin(async move {
        // Pre-allocate the inner mpsc large enough to absorb all flushed items at once,
        // avoiding backpressure that would require concurrent draining.
        let (tx, mut rx) = futures::channel::mpsc::channel(iterations);
        let mut sink = MixerSink::new(tx, cfg);

        for _ in 0..iterations {
            sink.start_send_unpin(item).expect("start_send must succeed");
        }

        sink.flush().await.expect("flush must succeed");

        for _ in 0..iterations {
            rx.next().await.expect("receive must succeed");
        }
    })
}

pub fn mixer_sink_throughput_minimal_mixing(c: &mut Criterion) {
    mixer_throughput(
        c,
        // Zero-delay uniform config so the sink measures per-message overhead, not the delay.
        MixerConfig::new_uniform(std::time::Duration::ZERO, std::time::Duration::ZERO),
        "mixer_sink",
        &[
            10 * 1024 * 2 * RANDOM_GIBBERISH.len(),
            40 * 1024 * 2 * RANDOM_GIBBERISH.len(),
        ],
        send_continuous_sink_load,
    );
}

pub fn mixer_stream_throughput_minimal_mixing(c: &mut Criterion) {
    mixer_throughput(
        c,
        minimal_delay_mixer_cfg(),
        "mixer_stream",
        &[40 * 1024 * 2 * RANDOM_GIBBERISH.len()],
        send_continuous_stream_load,
    );
}

/// Fair steady-state throughput: the channel (and, for the Poisson engine, its OS thread and
/// runtime) is created **once for the whole benchmark, before any timing**, then reused across
/// every size and sample. This isolates the per-message send+drain cost from the one-time
/// construction, which a per-iteration or per-sample setup would otherwise fold in.
fn reused_channel_group(c: &mut Criterion) -> criterion::BenchmarkGroup<'_, criterion::measurement::WallTime> {
    let mut group = c.benchmark_group("mixer_throughput_reused");
    group.sample_size(SAMPLE_SIZE);
    group
}

pub fn mixer_channel_throughput_reused(c: &mut Criterion) {
    // Zero-delay uniform config so the channel measures per-message overhead, not the delay.
    let cfg = MixerConfig::new_uniform(std::time::Duration::ZERO, std::time::Duration::ZERO);
    // Built once, before any benchmarking: runtime + channel, shared across every size/sample.
    let runtime = tokio::runtime::Runtime::new().expect("failed to create runtime");
    let (tx, rx) = channel::<&'static str>(cfg);
    let rx = Rc::new(RefCell::new(rx));

    let mut group = reused_channel_group(c);
    for bytes in sizes() {
        let iterations = bytes / RANDOM_GIBBERISH.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_through_mixer_channel", size_label(bytes))),
            &bytes,
            |b, _| {
                b.to_async(&runtime).iter_custom(|iters| {
                    let tx = tx.clone();
                    let rx = rx.clone();
                    async move {
                        let start = std::time::Instant::now();
                        for _ in 0..iters {
                            for _ in 0..iterations {
                                tx.send(RANDOM_GIBBERISH).expect("send must succeed");
                            }
                            for _ in 0..iterations {
                                std::hint::black_box(drain_one(&rx).await.expect("receive must succeed"));
                            }
                        }
                        start.elapsed()
                    }
                });
            },
        );
    }
    group.finish();
}

pub fn mixer_poisson_throughput_reused(c: &mut Criterion) {
    let cfg = minimal_delay_mixer_cfg();
    // Built once, before any benchmarking: runtime + channel + engine OS thread, shared across
    // every size/sample — so thread allocation is never inside (nor repeated per) the measurement.
    let runtime = tokio::runtime::Runtime::new().expect("failed to create runtime");
    let (tx, rx) = poisson_channel::<&'static str>(cfg);
    let rx = Rc::new(RefCell::new(rx));

    let mut group = reused_channel_group(c);
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
                        let start = std::time::Instant::now();
                        for _ in 0..iters {
                            for _ in 0..iterations {
                                tx.send(RANDOM_GIBBERISH).expect("send must succeed");
                            }
                            for _ in 0..iterations {
                                std::hint::black_box(drain_one(&rx).await.expect("receive must succeed"));
                            }
                        }
                        start.elapsed()
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    mixer_sink_throughput_minimal_mixing,
    mixer_stream_throughput_minimal_mixing,
    mixer_channel_throughput_reused,
    mixer_poisson_throughput_reused
);
criterion_main!(benches);
