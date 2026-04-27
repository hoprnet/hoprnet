use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{StreamExt, future::BoxFuture};
use hopr_transport_mixer::{channel, config::MixerConfig};
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
    f: impl Fn(&'static str, usize, MixerConfig) -> BoxFuture<'static, ()>,
) {
    let mut group = c.benchmark_group("mixer_throughput");
    group.sample_size(SAMPLE_SIZE);
    for bytes in [
        10 * 1024 * 2 * RANDOM_GIBBERISH.len(),
        40 * 1024 * 2 * RANDOM_GIBBERISH.len(),
    ]
    .iter()
    {
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

fn send_continuous_channel_load(item: &str, iterations: usize, cfg: MixerConfig) -> BoxFuture<'_, ()> {
    Box::pin(async move {
        let (tx, mut rx) = channel(cfg);

        for _ in 0..iterations {
            tx.send(item).expect("send must succeed");
        }

        for _ in 0..iterations {
            rx.next().await.expect("receive must succeed");
        }
    })
}

// Benchmark the throughput of the mixer channel when used in a pipe
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

pub fn mixer_channel_throughput_minimal_mixing(c: &mut Criterion) {
    mixer_throughput(
        c,
        minimal_delay_mixer_cfg(),
        "mixer_channel",
        send_continuous_channel_load,
    );
}

pub fn mixer_channel_throughput_through_sink_minimal_mixing(c: &mut Criterion) {
    mixer_throughput(
        c,
        minimal_delay_mixer_cfg(),
        "mixer_channel_sink_pipe",
        send_continuous_channel_load_through_sink_pipe,
    );
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

pub fn mixer_stream_throughput_minimal_mixing(c: &mut Criterion) {
    mixer_throughput(
        c,
        minimal_delay_mixer_cfg(),
        "mixer_stream",
        send_continuous_stream_load,
    );
}

criterion_group!(
    benches,
    mixer_channel_throughput_minimal_mixing,
    mixer_channel_throughput_through_sink_minimal_mixing,
    mixer_stream_throughput_minimal_mixing
);
criterion_main!(benches);
