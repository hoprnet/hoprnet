use std::{hint::black_box, num::NonZero, ops::RangeBounds, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use hopr_api::{
    chain::{ChannelEntry, RedeemableTicket, WinningProbability},
    tickets::TicketFactory,
    types::{crypto::prelude::*, crypto_random::Randomizable, internal::prelude::*, primitive::prelude::*},
};
use hopr_ticket_manager::{HoprTicketManager, RedbStore, RedbTicketQueue};

const TICKET_VALUE: u64 = 10;

pub fn generate_owned_tickets(
    issuer: &ChainKeypair,
    recipient: &ChainKeypair,
    count: usize,
    epochs: impl RangeBounds<u32> + Iterator<Item = u32>,
) -> anyhow::Result<Vec<RedeemableTicket>> {
    let mut tickets = Vec::new();
    for epoch in epochs {
        for i in 0..count {
            let hk1 = HalfKey::random();
            let hk2 = HalfKey::random();

            let ticket = TicketBuilder::default()
                .counterparty(recipient)
                .index(i as u64)
                .channel_epoch(epoch)
                .win_prob(WinningProbability::ALWAYS)
                .amount(TICKET_VALUE)
                .challenge(Challenge::from_hint_and_share(
                    &hk1.to_challenge()?,
                    &hk2.to_challenge()?,
                )?)
                .build_signed(issuer, &Default::default())?
                .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?)
                .into_redeemable(recipient, &Default::default())?;

            tickets.push(ticket);
        }
    }

    tickets.sort();
    Ok(tickets)
}

fn ticket_insert_redb_bench(c: &mut Criterion) {
    let issuer = ChainKeypair::random();
    let recipient = ChainKeypair::random();
    let tickets = generate_owned_tickets(&issuer, &recipient, 1, 1..=1).unwrap();

    let store = RedbStore::new_temp().unwrap();
    let manager = HoprTicketManager::new_with_factory(store).0;

    c.bench_function("ticket_insert_redb", |b| {
        b.iter(|| {
            manager.insert_incoming_ticket(black_box(tickets[0])).unwrap();
        });
    });
}

fn unrealized_value_redb_bench(c: &mut Criterion) {
    let issuer = ChainKeypair::random();
    let recipient = ChainKeypair::random();
    let tickets = generate_owned_tickets(&issuer, &recipient, 10, 1..=1).unwrap();
    let channel = ChannelEntry::builder()
        .between(&issuer, &recipient)
        .amount(100)
        .ticket_index(10)
        .status(ChannelStatus::Open)
        .epoch(1)
        .build()
        .unwrap();

    let store = RedbStore::new_temp().unwrap();
    let (manager, factory) = HoprTicketManager::new_with_factory(store);
    for ticket in tickets {
        manager.insert_incoming_ticket(ticket).unwrap();
    }

    c.bench_function("unrealized_value_redb", |b| {
        b.iter(|| {
            factory.remaining_incoming_channel_stake(black_box(&channel)).unwrap();
        })
    });
}

fn create_multihop_ticket_redb_bench(c: &mut Criterion) {
    let src = ChainKeypair::random();
    let dst = ChainKeypair::random();
    let channel_entry = ChannelEntry::builder()
        .between(&src, &dst)
        .amount(100)
        .ticket_index(1)
        .status(ChannelStatus::Open)
        .epoch(1)
        .build()
        .unwrap();

    let store = RedbStore::new_temp().unwrap();
    let (_, factory) = HoprTicketManager::new_with_factory(store);

    c.bench_function("create_multihop_ticket_redb", |b| {
        b.iter(|| {
            factory
                .new_multihop_ticket(
                    black_box(&channel_entry),
                    black_box(NonZero::new(2).unwrap()),
                    black_box(WinningProbability::ALWAYS),
                    black_box(HoprBalance::from(10)),
                )
                .unwrap();
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(1000)
        .measurement_time(Duration::from_secs(30));
    targets = ticket_insert_redb_bench,
    unrealized_value_redb_bench,
    create_multihop_ticket_redb_bench
);
criterion_main!(benches);
