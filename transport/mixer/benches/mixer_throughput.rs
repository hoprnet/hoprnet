use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::StreamExt;
use hopr_transport_mixer::{channel, config::MixerConfig};

const SAMPLE_SIZE: usize = 10;

/// 512 characters long string of random gibberish
const RANDOM_GIBBERISH: &str = "abcdferjskdiq7LGuzjfXMEI2tTCUIZsCDsHnfycUbPcA1boJ48Jm7xBBNIvxsrbK3bNCevOMXYMqrhsVBXfmKy23K7ItgbuObTmqk0ndfceAhugLZveAhp4Xx1vHCAROY69sOTJiia3EBC2aXSBpUfb3WHSJDxHRMHwzCwd0BPj4WFi4Ig884Ph6altlFWzpL3ILsHmLxy9KoPCAtolb3YEegMCI4y9BsoWyCtcZdBHBrqXaSzuJivw5J1DBudj3Z6oORrEfRuFIQLi0l89Emc35WhSyzOdguC1x9PS8AiIAu7UoXlp3VIaqVUu4XGUZ21ABxI9DyMzxGbOOlsrRGFFN9G8di9hqIX1UOZpRgMNmtDwZoyoU2nGLoWGM58buwuvbNkLjGu2X9HamiiDsRIR4vxi5i61wIP6VueVOb68wvbz8csR88OhFsExjGBD9XXtJvUjy1nwdkikBOblNm2FUbyq8aHwHocoMqZk8elbYMHgbjme9d1CxZQKRwOR";

async fn send_continuous_load(item: &str, iterations: usize, cfg: MixerConfig) {
    let (tx, mut rx) = channel(cfg);

    for _ in 0..iterations {
        tx.send(item).expect("send must succeed");
    }

    for _ in 0..iterations {
        rx.next().await.expect("receive must succeed");
    }
}

pub fn mixer_channel_throughput(c: &mut Criterion, cfg: MixerConfig) {
    let mut group = c.benchmark_group("mixer_throughput");
    group.sample_size(SAMPLE_SIZE);
    for bytes in [
        2 * 1024 * 2 * RANDOM_GIBBERISH.len(),
        10 * 1024 * 2 * RANDOM_GIBBERISH.len(),
        40 * 1024 * 2 * RANDOM_GIBBERISH.len(),
    ]
    .iter()
    {
        group.throughput(Throughput::Bytes(*bytes as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!(
                "random data with size {}",
                bytesize::ByteSize::b(*bytes as u64)
            )),
            &RANDOM_GIBBERISH.to_string(),
            |b, data| {
                let runtime = criterion::async_executor::AsyncStdExecutor {};

                b.to_async(runtime)
                    .iter(|| send_continuous_load(data.as_str(), bytes / RANDOM_GIBBERISH.len(), cfg));
            },
        );
    }
    group.finish();
}

pub fn mixer_channel_throughput_default_mixing(c: &mut Criterion) {
    mixer_channel_throughput(c, MixerConfig::default());
}

pub fn mixer_channel_throughput_minimal_mixing(c: &mut Criterion) {
    mixer_channel_throughput(
        c,
        MixerConfig {
            min_delay: std::time::Duration::from_millis(0),
            delay_range: std::time::Duration::from_millis(1),
            ..MixerConfig::default()
        },
    );
}

criterion_group!(
    benches,
    mixer_channel_throughput_default_mixing,
    mixer_channel_throughput_minimal_mixing
);
criterion_main!(benches);
