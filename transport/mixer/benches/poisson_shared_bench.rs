//! Experiment: dedicated-thread `poisson_channel` vs shared-pool `poisson_shared_channel`.
//!
//! Same reused-channel methodology as `mixer_throughput_bench` (setup once, before timing) over
//! the 1–10 MB/s range, so the two engines are directly comparable.

use std::{cell::RefCell, rc::Rc};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{StreamExt, future::poll_fn};
use hopr_transport_mixer::{config::MixerConfig, poisson_channel, poisson_shared_channel};

const SAMPLE_SIZE: usize = 10;
const RANDOM_GIBBERISH: &str = "abcdferjskdiq7LGuzjfXMEI2tTCUIZsCDsHnfycUbPcA1boJ48Jm7xBBNIvxsrbK3bNCevOMXYMqrhsVBXfmKy23K7ItgbuObTmqk0ndfceAhugLZveAhp4Xx1vHCAROY69sOTJiia3EBC2aXSBpUfb3WHSJDxHRMHwzCwd0BPj4WFi4Ig884Ph6altlFWzpL3ILsHmLxy9KoPCAtolb3YEegMCI4y9BsoWyCtcZdBHBrqXaSzuJivw5J1DBudj3Z6oORrEfRuFIQLi0l89Emc35WhSyzOdguC1x9PS8AiIAu7UoXlp3VIaqVUu4XGUZ21ABxI9DyMzxGbOOlsrRGFFN9G8di9hqIX1UOZpRgMNmtDwZoyoU2nGLoWGM58buwuvbNkLjGu2X9HamiiDsRIR4vxi5i61wIP6VueVOb68wvbz8csR88OhFsExjGBD9XXtJvUjy1nwdkikBOblNm2FUbyq8aHwHocoMqZk8elbYMHgbjme9d1CxZQKRwOR";

#[inline]
fn minimal_delay_mixer_cfg() -> MixerConfig {
    MixerConfig {
        min_delay: std::time::Duration::from_millis(0),
        delay_range: std::time::Duration::from_millis(1),
        ..MixerConfig::default()
    }
}

fn sizes() -> [usize; 3] {
    [1_000_000, 5_000_000, 10_000_000]
}

fn size_label(bytes: usize) -> String {
    format!("{}_MB_per_s", bytes / 1_000_000)
}

async fn drain_one<R: StreamExt + Unpin>(rx: &Rc<RefCell<R>>) -> Option<R::Item> {
    poll_fn(|cx| rx.borrow_mut().poll_next_unpin(cx)).await
}

pub fn dedicated(c: &mut Criterion) {
    let cfg = minimal_delay_mixer_cfg();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let (tx, rx) = poisson_channel::<&'static str>(cfg);
    let rx = Rc::new(RefCell::new(rx));

    let mut group = c.benchmark_group("poisson_shared_experiment");
    group.sample_size(SAMPLE_SIZE);
    for bytes in sizes() {
        let iterations = bytes / RANDOM_GIBBERISH.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_dedicated_thread", size_label(bytes))),
            &bytes,
            |b, _| {
                b.to_async(&runtime).iter_custom(|iters| {
                    let tx = tx.clone();
                    let rx = rx.clone();
                    async move {
                        let start = std::time::Instant::now();
                        for _ in 0..iters {
                            for _ in 0..iterations {
                                tx.send(RANDOM_GIBBERISH).expect("send");
                            }
                            for _ in 0..iterations {
                                std::hint::black_box(drain_one(&rx).await.expect("recv"));
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

pub fn shared(c: &mut Criterion) {
    let cfg = minimal_delay_mixer_cfg();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let (tx, rx) = poisson_shared_channel::<&'static str>(cfg);
    let rx = Rc::new(RefCell::new(rx));

    let mut group = c.benchmark_group("poisson_shared_experiment");
    group.sample_size(SAMPLE_SIZE);
    for bytes in sizes() {
        let iterations = bytes / RANDOM_GIBBERISH.len();
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_shared_pool", size_label(bytes))),
            &bytes,
            |b, _| {
                b.to_async(&runtime).iter_custom(|iters| {
                    let tx = tx.clone();
                    let rx = rx.clone();
                    async move {
                        let start = std::time::Instant::now();
                        for _ in 0..iters {
                            for _ in 0..iterations {
                                tx.send(RANDOM_GIBBERISH).expect("send");
                            }
                            for _ in 0..iterations {
                                std::hint::black_box(drain_one(&rx).await.expect("recv"));
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

criterion_group!(benches, dedicated, shared);
criterion_main!(benches);
