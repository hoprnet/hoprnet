#[path = "../src/utils.rs"]
mod utils;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_random::Randomizable;
use hopr_internal_types::path::ValidatedPath;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_crypto_types::prelude::*;
use hopr_protocol_hopr::PacketEncoder;
use crate::utils::{create_blokli_client, create_decoder, create_encoder, create_node, PEERS};

fn hopr_encoder_bench(c: &mut Criterion) {
    let blokli_client = create_blokli_client().unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let encoder = runtime.block_on(async {
        let sender = create_node(0, &blokli_client).await.unwrap();
        create_encoder(&sender)
    });

    let data = b"some random message to encode and decode";
    let routing = ResolvedTransportRouting::Forward {
        pseudonym: HoprPseudonym::random(),
        forward_path: ValidatedPath::direct(
            *PEERS[1].1.public(),
            PEERS[1].0.public().to_address()
        ),
        return_paths: vec![],
    };

    let mut group = c.benchmark_group("hopr_encoder");
    group.throughput(Throughput::Elements(1));

    group.bench_function("encode_packet", move |b| {
        b.to_async(&runtime).iter(|| async {
            encoder.encode_packet(&data, routing.clone(), None).await.unwrap()
        })
    });
}


fn hopr_decoder_bench(_c: &mut Criterion) {

}

criterion_group!(benches, hopr_encoder_bench, hopr_decoder_bench);
criterion_main!(benches);