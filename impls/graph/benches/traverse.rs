use std::{hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hopr_api::{
    OffchainKeypair, OffchainPublicKey,
    graph::{
        NetworkGraphTraverse, NetworkGraphWrite,
        traits::{EdgeObservableWrite, EdgeWeightType},
    },
};
use hopr_crypto_types::prelude::Keypair;
use hopr_network_graph::{ChannelGraph, costs::SimpleHoprCostFn};

// ── Graph construction helpers ───────────────────────────────────────────────

/// Deterministically derives an [`OffchainPublicKey`] from a 32-bit index.
///
/// The first 4 bytes of the Ed25519 seed encode `i` in little-endian; the
/// remaining bytes are a fixed non-zero fill so that every seed is unique and
/// valid for all index values in `0..u32::MAX`.
fn key_from_index(i: u32) -> OffchainPublicKey {
    let mut secret = [0x5Au8; 32];
    secret[..4].copy_from_slice(&i.to_le_bytes());
    *OffchainKeypair::from_secret(&secret)
        .expect("deterministic secret is always valid for Ed25519")
        .public()
}

/// Builds a [`ChannelGraph`] with `node_count` nodes and a controlled edge
/// density using a ring-plus-chords topology.
///
/// Every edge is annotated with the full set of observations (connected,
/// immediate QoS, intermediate QoS, and payment-channel capacity), so it
/// satisfies any of the built-in cost functions without extra setup.
///
/// # Parameters
///
/// * `node_count` — number of nodes in the graph (including `me`).
/// * `density` — target edge-to-node ratio (e.g. `10`, `100`, `1_000`). Each node receives outgoing edges to the next
///   `min(node_count − 1, density)` nodes in the ring (wrapping around), giving `node_count × min(node_count − 1,
///   density)` total directed edges.
///
/// # Edge counts by (node_count, density)
///
/// | nodes | density | edges/node | total edges  |
/// |------:|--------:|-----------:|-------------:|
/// |    10 |      10 |          9 |           90 |
/// |    10 |     100 |          9 |           90 |
/// |    10 |   1 000 |          9 |           90 |
/// |   100 |      10 |         10 |        1 000 |
/// |   100 |     100 |         99 |        9 900 |
/// |   100 |   1 000 |         99 |        9 900 |
/// | 1 000 |      10 |         10 |       10 000 |
/// | 1 000 |     100 |        100 |      100 000 |
/// | 1 000 |   1 000 |        999 |      999 000 |
///
/// When `density ≥ node_count − 1` the graph is fully connected (no
/// self-loops), which is the theoretical maximum for a simple directed graph.
///
/// Returns the populated graph and the ordered key list (`keys[0]` is the
/// local node identity, i.e., `me`).
fn build_graph(node_count: usize, density: usize) -> (ChannelGraph, Vec<OffchainPublicKey>) {
    let edges_per_node = (node_count - 1).min(density);
    let keys: Vec<OffchainPublicKey> = (0..node_count as u32).map(key_from_index).collect();

    let graph = ChannelGraph::new(keys[0]);
    for &key in &keys[1..] {
        graph.add_node(key);
    }

    // Ring + chords: node i → (i+1)%N, (i+2)%N, …, (i+edges_per_node)%N.
    for i in 0..node_count {
        for step in 1..=edges_per_node {
            let j = (i + step) % node_count;
            graph.upsert_edge(&keys[i], &keys[j], |obs| {
                obs.record(EdgeWeightType::Connected(true));
                obs.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(50))));
                obs.record(EdgeWeightType::Capacity(Some(1_000)));
            });
        }
    }

    (graph, keys)
}

// ── Benchmark: NetworkGraphTraverse::simple_paths ────────────────────────────

/// Benchmarks [`NetworkGraphTraverse::simple_paths`] for 2-hop, 3-hop, and
/// 4-hop routes across all combinations of three node counts (10 / 100 / 1 000)
/// and three edge densities (10× / 100× / 1 000×).
///
/// The benchmark ID encodes both dimensions, e.g. `2-hop/100nodes/100x`.
///
/// Destination nodes are chosen to sit at the "apex" of the ring where the
/// maximum number of simple paths converge from `me`:
///
/// * **2-hop apex** — node `(epn + 1)`: up to `epn` distinct 2-hop paths, where `epn = min(node_count − 1, density)`.
/// * **3-hop apex** — node `(2 × epn + 1)`: up to `epn²` 3-hop paths.
/// * **4-hop apex** — node `(3 × epn + 1)`: up to `epn³` 4-hop paths.
///
/// `take_count` is capped at 10 to reflect realistic production usage and to
/// keep the benchmark runtime bounded at high densities.
fn bench_simple_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("NetworkGraphTraverse/simple_paths");

    for size in [10_usize, 100, 1_000] {
        for density in [10_usize, 100, 1_000] {
            let (graph, keys) = build_graph(size, density);
            let me = &keys[0];
            let epn = (size - 1).min(density); // effective edges per node

            let dst_2hop = &keys[(epn + 1).min(size - 1)];
            let dst_3hop = &keys[(2 * epn + 1).min(size - 1)];
            let dst_4hop = &keys[(3 * epn + 1).min(size - 1)];

            let param = format!("{size}nodes/{density}x");

            group.bench_with_input(BenchmarkId::new("2-hop", &param), &size, |b, _| {
                b.iter(|| {
                    black_box(graph.simple_paths(me, black_box(dst_2hop), 2, Some(10), SimpleHoprCostFn::new(2)))
                });
            });

            group.bench_with_input(BenchmarkId::new("3-hop", &param), &size, |b, _| {
                b.iter(|| {
                    black_box(graph.simple_paths(me, black_box(dst_3hop), 3, Some(10), SimpleHoprCostFn::new(3)))
                });
            });

            group.bench_with_input(BenchmarkId::new("4-hop", &param), &size, |b, _| {
                b.iter(|| {
                    black_box(graph.simple_paths(me, black_box(dst_4hop), 4, Some(10), SimpleHoprCostFn::new(4)))
                });
            });
        }
    }

    group.finish();
}

// ── Benchmark: NetworkGraphTraverse::simple_loopback_to_self ─────────────────

/// Benchmarks [`NetworkGraphTraverse::simple_loopback_to_self`] for 2-hop,
/// 3-hop, and 4-hop loopback routes across all combinations of three node
/// counts (10 / 100 / 1 000) and three edge densities (10× / 100× / 1 000×).
///
/// The benchmark ID encodes both dimensions, e.g. `3-hop/1000nodes/100x`.
///
/// In the ring topology, `me`'s directly connected neighbours — the destination
/// set for the internal path search — grow with the density (up to
/// `node_count − 1`), so higher-density cases stress the destination-set fan-out
/// in addition to the path-enumeration depth.
///
/// `take_count` is capped at 10.
fn bench_simple_loopback(c: &mut Criterion) {
    let mut group = c.benchmark_group("NetworkGraphTraverse/simple_loopback_to_self");

    for size in [10_usize, 100, 1_000] {
        for density in [10_usize, 100, 1_000] {
            let (graph, _keys) = build_graph(size, density);
            let param = format!("{size}nodes/{density}x");

            group.bench_with_input(BenchmarkId::new("2-hop", &param), &size, |b, _| {
                b.iter(|| black_box(graph.simple_loopback_to_self(2, Some(10))));
            });

            group.bench_with_input(BenchmarkId::new("3-hop", &param), &size, |b, _| {
                b.iter(|| black_box(graph.simple_loopback_to_self(3, Some(10))));
            });

            group.bench_with_input(BenchmarkId::new("4-hop", &param), &size, |b, _| {
                b.iter(|| black_box(graph.simple_loopback_to_self(4, Some(10))));
            });
        }
    }

    group.finish();
}

// ── Entry point ──────────────────────────────────────────────────────────────

criterion_group!(benches, bench_simple_paths, bench_simple_loopback);
criterion_main!(benches);
