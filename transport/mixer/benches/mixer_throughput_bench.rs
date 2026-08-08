use std::{cell::RefCell, rc::Rc};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{
    SinkExt, StreamExt,
    future::{BoxFuture, poll_fn},
};
use hopr_transport_mixer::{MixerSink, channel, config::MixerConfig, poisson_channel};
use rust_stream_ext_concurrent::then_concurrent::StreamThenConcurrentExt;

const SAMPLE_SIZE: usize = 10;

/// 512 characters long string of random gibberish
const RANDOM_GIBBERISH: &str = "abcdferjskdiq7LGuzjfXMEI2tTCUIZsCDsHnfycUbPcA1boJ48Jm7xBBNIvxsrbK3bNCevOMXYMqrhsVBXfmKy23K7ItgbuObTmqk0ndfceAhugLZveAhp4Xx1vHCAROY69sOTJiia3EBC2aXSBpUfb3WHSJDxHRMHwzCwd0BPj4WFi4Ig884Ph6altlFWzpL3ILsHmLxy9KoPCAtolb3YEegMCI4y9BsoWyCtcZdBHBrqXaSzuJivw5J1DBudj3Z6oORrEfRuFIQLi0l89Emc35WhSyzOdguC1x9PS8AiIAu7UoXlp3VIaqVUu4XGUZ21ABxI9DyMzxGbOOlsrRGFFN9G8di9hqIX1UOZpRgMNmtDwZoyoU2nGLoWGM58buwuvbNkLjGu2X9HamiiDsRIR4vxi5i61wIP6VueVOb68wvbz8csR88OhFsExjGBD9XXtJvUjy1nwdkikBOblNm2FUbyq8aHwHocoMqZk8elbYMHgbjme9d1CxZQKRwOR";

#[inline]
fn minimal_delay_mixer_cfg() -> MixerConfig {
    MixerConfig {
        min_delay: std::time::Duration::from_millis(0),
        delay_range: std::time::Duration::from_millis(1),
        ..MixerConfig::default()
    }
}

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

// Benchmark the throughput of the mixer channel when used in a pipe
#[allow(dead_code)]
fn send_continuous_channel_load_through_sink_pipe(
    item: &'static str,
    iterations: usize,
    cfg: MixerConfig,
) -> BoxFuture<'static, ()> {
    Box::pin(async move {
        let (o_tx, o_rx) = futures::channel::mpsc::unbounded();
        let (tx, mut rx) = channel(cfg);

        let pipe = tokio::task::spawn(o_rx.map(Ok).forward(tx));

        for _ in 0..iterations {
            o_tx.unbounded_send(item).expect("send must succeed");
        }

        for _ in 0..iterations {
            rx.next().await.expect("receive must succeed");
        }

        pipe.abort();
    })
}

fn send_continuous_stream_load(item: &str, iterations: usize, cfg: MixerConfig) -> BoxFuture<'_, ()> {
    Box::pin(async move {
        let (tx, rx) = futures::channel::mpsc::unbounded();

        let mut rx = rx.then_concurrent(
            |v| async move {
                let random_delay = cfg.random_delay();

                tokio::time::sleep(random_delay).await;

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
        minimal_delay_mixer_cfg(),
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

/// Workload volumes spanning the realistic 1–10 MB/s operating range. Each volume is one
/// second of offered load at 1, 5 and 10 MB/s respectively, so the reported throughput shows
/// how much headroom the engine has over the target rate.
fn sizes() -> [usize; 3] {
    [1_000_000, 5_000_000, 10_000_000]
}

fn size_label(bytes: usize) -> String {
    // e.g. "1_MB_per_s" — the volume equals one second of load at this rate.
    format!("{}_MB_per_s", bytes / 1_000_000)
}

/// Drain one item, borrowing the shared receiver only inside each poll (never across the
/// `.await`) so the `Rc<RefCell<_>>` handle can be owned by the future without tripping
/// `clippy::await_holding_refcell_ref`.
async fn drain_one<R: StreamExt + Unpin>(rx: &Rc<RefCell<R>>) -> Option<R::Item> {
    poll_fn(|cx| rx.borrow_mut().poll_next_unpin(cx)).await
}

pub fn mixer_channel_throughput_reused(c: &mut Criterion) {
    let cfg = minimal_delay_mixer_cfg();
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
