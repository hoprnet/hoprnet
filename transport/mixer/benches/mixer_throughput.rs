use criterion::{async_executor::FuturesExecutor, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::StreamExt;
use hopr_transport_mixer::{channel, config::MixerConfig};

const SAMPLE_SIZE: usize = 100;

/// 512 characters long string of random gibberish
const RANDOM_GIBBERISH: &str = "abcdferjskdiq7LGuzjfXMEI2tTCUIZsCDsHnfycUbPcA1boJ48Jm7xBBNIvxsrbK3bNCevOMXYMqrhsVBXfmKy23K7ItgbuObTmqk0ndfceAhugLZveAhp4Xx1vHCAROY69sOTJiia3EBC2aXSBpUfb3WHSJDxHRMHwzCwd0BPj4WFi4Ig884Ph6altlFWzpL3ILsHmLxy9KoPCAtolb3YEegMCI4y9BsoWyCtcZdBHBrqXaSzuJivw5J1DBudj3Z6oORrEfRuFIQLi0l89Emc35WhSyzOdguC1x9PS8AiIAu7UoXlp3VIaqVUu4XGUZ21ABxI9DyMzxGbOOlsrRGFFN9G8di9hqIX1UOZpRgMNmtDwZoyoU2nGLoWGM58buwuvbNkLjGu2X9HamiiDsRIR4vxi5i61wIP6VueVOb68wvbz8csR88OhFsExjGBD9XXtJvUjy1nwdkikBOblNm2FUbyq8aHwHocoMqZk8elbYMHgbjme9d1CxZQKRwOR";

async fn send_continuous_load(item: &str, iterations: usize) {
    let (tx, mut rx) = channel(MixerConfig::default());

    for _ in 0..iterations {
        tx.send(item).expect("send must succeed");
    }

    for _ in 0..iterations {
        rx.next().await.expect("receive must succeed");
    }
}

pub fn mixer_channel_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixer_throughput");
    group.sample_size(SAMPLE_SIZE);
    const ITERATIONS: usize = 2 * 1024 * 10; // 10MB

    group.throughput(Throughput::Elements(1));
    group.bench_with_input(
        BenchmarkId::from_parameter(format!(
            "random data with size {}",
            bytesize::ByteSize::b((RANDOM_GIBBERISH.len() * ITERATIONS) as u64)
        )),
        &RANDOM_GIBBERISH.to_string(),
        |b, data| {
            b.to_async(FuturesExecutor)
                .iter(|| send_continuous_load(data.as_str(), ITERATIONS));
        },
    );
    group.finish();
}

criterion_group!(benches, mixer_channel_throughput,);
criterion_main!(benches);
