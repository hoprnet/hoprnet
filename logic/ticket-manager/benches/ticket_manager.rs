use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

fn ticket_insert_redb_bench(c: &mut Criterion) {
    todo!()
}

fn unrealized_value_redb_bench(c: &mut Criterion) {
    todo!()
}

fn create_multihop_ticket_redb_bench(c: &mut Criterion) {
    todo!()
}

criterion_group!(benches, ticket_insert_redb_bench, unrealized_value_redb_bench, create_multihop_ticket_redb_bench);
criterion_main!(benches);