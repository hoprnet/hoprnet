use criterion::*;

fn benchmark(c: &mut Criterion) {
    c.bench_function("write 100 1K chunks", |b| {
        use futures::prelude::*;

        let data = [1; 1024];

        b.iter_batched(
            sluice::pipe::pipe,
            |(reader, mut writer)| {
                let producer = async {
                    for _ in 0u8..100 {
                        writer.write_all(&data).await.unwrap();
                    }
                    writer.close().await.unwrap();
                };

                let consumer = async {
                    let mut sink = futures::io::sink();
                    futures::io::copy(reader, &mut sink).await.unwrap();
                };

                futures::executor::block_on(future::join(producer, consumer));
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
