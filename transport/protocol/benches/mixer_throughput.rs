use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

const SAMPLE_SIZE: usize = 100_000;

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
