#[allow(unused)]
#[path = "../src/session/utils/test.rs"]
mod test;

#[allow(unused)]
#[path = "../src/session/frames.rs"]
mod frames;

#[allow(unused)]
#[path = "../src/session/processing/mod.rs"]
mod processing;

use std::{collections::VecDeque, time::Duration};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{StreamExt, TryStreamExt};
use hopr_crypto_packet::prelude::HoprPacket;
use rand::{Rng, thread_rng};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator, ParallelSlice};

use crate::{
    frames::Segment,
    processing::{frame_reconstructor, segment},
    test::linear_half_normal_shuffle,
};

async fn send_one_way(segments: &Vec<Segment>) {
    let (r_sink, seq_stream) = frame_reconstructor(Duration::from_secs(5), 8192, futures::sink::drain());

    let all = hopr_async_runtime::prelude::spawn(seq_stream.try_collect::<Vec<_>>());
    let segments = segments.clone();
    hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

    all.await.unwrap();
}

pub fn frame_reconstructor_randomized_benchmark(c: &mut Criterion) {
    static KB: usize = 1024;
    static FRAME_LEN: usize = 1492;
    static MTU: usize = HoprPacket::PAYLOAD_SIZE;

    let mut group = c.benchmark_group("frame_reconstructor_randomized_benchmark");

    group.sample_size(100000);
    group.measurement_time(Duration::from_secs(30));

    for size in [16 * KB, 64 * KB, 128 * KB, 1024 * KB].iter() {
        let mut data = vec![0u8; *size];
        thread_rng().fill(&mut data[..]);
        let segments = data
            .par_chunks(FRAME_LEN)
            .enumerate()
            .flat_map(|(id, chunk)| segment(chunk, MTU, (id + 1) as u32).unwrap())
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
