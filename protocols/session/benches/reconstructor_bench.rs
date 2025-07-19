#[allow(unused)]
#[path = "../src/utils/mod.rs"]
mod utils;

#[allow(unused)]
#[path = "../src/utils/test.rs"]
mod test;

#[allow(unused)]
#[path = "../src/errors.rs"]
mod errors;

#[allow(unused)]
#[path = "../src/frames.rs"]
mod frames;

#[allow(unused)]
#[path = "../src/processing/mod.rs"]
mod processing;

#[allow(unused)]
#[path = "../src/protocol/mod.rs"]
mod protocol;

use std::{collections::VecDeque, time::Duration};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use futures::{SinkExt, StreamExt, TryStreamExt};
use hopr_crypto_packet::prelude::HoprPacket;
use rand::{Rng, thread_rng};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator, ParallelSlice};

use crate::{
    errors::SessionError,
    frames::{OrderedFrame, Segment},
    processing::{ReassemblerExt, SegmenterExt, SequencerExt},
    test::{linear_half_normal_shuffle, segment},
};

const MTU: usize = HoprPacket::PAYLOAD_SIZE;
const FRAME_SIZE: usize = 1492;

async fn send_one_way(segments: &Vec<Segment>) {
    let (reassm_tx, reassm_rx) = futures::channel::mpsc::unbounded::<Segment>();

    let seq_stream = reassm_rx
        .reassembler(Duration::from_secs(1), 1024)
        .filter_map(|maybe_frame| match maybe_frame {
            Ok(frame) => futures::future::ready(Some(OrderedFrame(frame))),
            Err(_) => futures::future::ready(None),
        })
        .sequencer(Duration::from_secs(1), 1024)
        .and_then(|frame| futures::future::ok(frame.0));

    let r_sink = reassm_tx
        .sink_map_err(|_| SessionError::InvalidSegment)
        .segmenter::<MTU>(FRAME_SIZE);

    let all = hopr_async_runtime::prelude::spawn(seq_stream.try_collect::<Vec<_>>());
    let segments = segments.clone();
    hopr_async_runtime::prelude::spawn(futures::stream::iter(segments).map(Ok).forward(r_sink));

    all.await.unwrap().unwrap();
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
