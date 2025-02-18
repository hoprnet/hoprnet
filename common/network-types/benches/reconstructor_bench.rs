use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures::{StreamExt, TryStreamExt};
use rand::{thread_rng, Rng};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator, ParallelSlice};
use std::collections::VecDeque;
use std::time::Duration;

use hopr_network_types::prelude::{frame_reconstructor, Frame, Segment};
use hopr_network_types::session::utils::test::linear_half_normal_shuffle;
use hopr_network_types::session::FrameId;

async fn send_one_way(segments: &Vec<Segment>) {
    let (r_sink, seq_stream) = frame_reconstructor(Duration::from_secs(5), 8192);

    let all = hopr_async_runtime::prelude::spawn(seq_stream.try_collect::<Vec<_>>());
    let segments = segments.clone();
    hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

    all.await.unwrap();
}

pub fn frame_reconstructor_randomized_benchmark(c: &mut Criterion) {
    static KB: usize = 1024;
    static FRAME_LEN: usize = 1492;
    static MTU: usize = 462;

    let mut group = c.benchmark_group("frame_reconstructor_randomized_benchmark");

    group.sample_size(100000);
    group.measurement_time(Duration::from_secs(30));

    for size in [16 * KB, 64 * KB, 128 * KB, 1024 * KB].iter() {
        let mut data = vec![0u8; *size];
        thread_rng().fill(&mut data[..]);
        let segments = data
            .par_chunks(FRAME_LEN)
            .enumerate()
            .flat_map(|(id, chunk)| {
                Frame {
                    frame_id: (id + 1) as FrameId,
                    data: chunk.into(),
                }
                .segment(MTU)
                .unwrap()
            })
            .collect::<VecDeque<_>>();
        let segments = linear_half_normal_shuffle(&mut thread_rng(), segments, 4.0);

        let runtime = tokio::runtime::Runtime::new().unwrap();

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &segments, |b, data| {
            b.to_async(&runtime).iter(|| send_one_way(data));
        });
    }
    group.finish();
}

criterion_group!(benches, frame_reconstructor_randomized_benchmark,);
criterion_main!(benches);
