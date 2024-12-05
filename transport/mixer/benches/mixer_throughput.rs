use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const SAMPLE_SIZE: usize = 100_000;

const RANDOM_GIBBERISH: &str = "q7LGuzjfXMEI2tTCUIZsCDsHnfycUbPcA1boJ48Jm7xBBNIvxsrbK3bNCevOMXYMqrhsVBXfmKy23K7ItgbuObTmqk0ndfceAhugLZveAhp4Xx1vHCAROY69sOTJiia3EBC2aXSBpUfb3WHSJDxHRMHwzCwd0BPj4WFi4Ig884Ph6altlFWzpL3ILsHmLxy9KoPCAtolb3YEegMCI4y9BsoWyCtcZdBHBrqXaSzuJivw5J1DBudj3Z6oORrEfRuFIQLi0l89Emc35WhSyzOdguC1x9PS8AiIAu7UoXlp3VIaqVUu4XGUZ21ABxI9DyMzxGbOOlsrRGFFN9G8di9hqIX1UOZpRgMNmtDwZoyoU2nGLoWGM58buwuvbNkLjGu2X9HamiiDsRIR4vxi5i61wIP6VueVOb68wvbz8csR88OhFsExjGBD9XXtJvUjy1nwdkikBOblNm2FUbyq8aHwHocoMqZk8elbYMHgbjme9d1CxZQKRwOR";

pub fn mixer_channel_throughput(c: &mut Criterion) {
    // Prepare the benchmark pre-requisites

    let mut group = c.benchmark_group("mixer_throughput");
    group.sample_size(SAMPLE_SIZE);

    for hop in [0, 1, 2, 3].iter() {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(format!("{hop} hop")), hop, |b, &hop| {
            b.iter(|| {
                // https://bheisler.github.io/criterion.rs/book/user_guide/benchmarking_async.html
                // operation to benchmark
            });
        });
    }
    group.finish();
}

criterion_group!(benches, mixer_channel_throughput,);
criterion_main!(benches);
