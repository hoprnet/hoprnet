use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hopr_crypto_packet::prelude::validate_unacknowledged_ticket;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::HoprBalance;

// Avoid musl's default allocator due to degraded performance
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const SAMPLE_SIZE: usize = 100_000;

pub fn validate_ticket_bench(c: &mut Criterion) {
    let source = ChainKeypair::random();
    let dest = ChainKeypair::random();

    let channel = ChannelBuilder::new(&source, &dest)
        .with_balance(HoprBalance::new_base(1000))
        .with_ticket_index(1)
        .with_status(ChannelStatus::Open)
        .with_epoch(1)
        .build();

    let ticket = TicketBuilder::default()
        .counterparty(&dest)
        .balance(HoprBalance::new_base(100))
        .index(1)
        .eth_challenge(Default::default())
        .win_prob(WinningProbability::ALWAYS)
        .channel_epoch(1)
        .build_signed(&source, &Hash::default())
        .unwrap();

    let mut group = c.benchmark_group("validate_ticket_bench");
    group.sample_size(SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));
    group.bench_function("validate_unack_ticket", |b| {
        b.iter(|| {
            validate_unacknowledged_ticket(
                ticket.clone().leak(),
                &channel,
                HoprBalance::new_base(10),
                WinningProbability::ALWAYS,
                HoprBalance::new_base(200),
                &Default::default(),
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, validate_ticket_bench);
criterion_main!(benches);
