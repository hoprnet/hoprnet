//! Routing and session tracing overhead. Compare the same harness and logging level on both
//! revisions with `cargo bench -p hopr-transport --features testing,telemetry,runtime-tokio,serde
//! --bench routing_tracing_bench --profile release`. Set HOPR_BENCH_LOG to info or debug;
//! formatted output goes to a sink so filesystem/terminal speed does not dominate the comparison.

use std::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};
use futures::{AsyncWriteExt, stream};
use hopr_api::{
    OffchainPublicKey,
    graph::{
        NetworkGraphUpdate, NetworkGraphWrite,
        traits::{Balance, EdgeObservableWrite, EdgeWeightType},
    },
    types::{
        crypto::prelude::{Keypair, OffchainKeypair},
        internal::{prelude::*, routing::DestinationRouting},
        primitive::{prelude::Address, traits::BytesRepresentable},
    },
};
use hopr_network_graph::ChannelGraph;
use hopr_protocol_hopr::MemorySurbStore;
use hopr_transport::{
    path::{HoprGraphPathSelector, PathPlanner, PathPlannerConfig, PathSelector},
    session::{HoprSession, HoprSessionConfig},
    testing::stubs::StubChainApi,
};
use tracing::Instrument;

const CANDIDATES: usize = 16;
type Planner = PathPlanner<MemorySurbStore, StubChainApi, HoprGraphPathSelector<ChannelGraph>>;

struct Fixture {
    source: OffchainPublicKey,
    destination: OffchainPublicKey,
    selector: HoprGraphPathSelector<ChannelGraph>,
    planner: Planner,
    routing: DestinationRouting,
}

/// A funded, measured, bidirectional diamond with sixteen interchangeable one-relay routes.
fn fixture() -> Fixture {
    let keys: Vec<_> = (0..CANDIDATES + 2)
        .map(|i| {
            *OffchainKeypair::from_secret(&[i as u8 + 1; 32])
                .expect("deterministic benchmark key")
                .public()
        })
        .collect();
    let addresses: Vec<_> = (0..keys.len()).map(|i| Address::from([i as u8 + 1; 20])).collect();
    let source = keys[0];
    let destination = keys[CANDIDATES + 1];
    let graph = ChannelGraph::new(source);
    graph.set_ticket_face_value(Balance::one());
    let mut chain = StubChainApi::builder().me(addresses[0]);
    for (key, address) in keys.iter().zip(&addresses) {
        chain = chain.peer(key, *address);
    }
    for relay in 1..=CANDIDATES {
        for (from, to) in [(0, relay), (relay, CANDIDATES + 1), (CANDIDATES + 1, relay), (relay, 0)] {
            graph.upsert_edge(&keys[from], &keys[to], |observed| {
                observed.record(EdgeWeightType::Connected(true));
                observed.record(EdgeWeightType::Immediate(Ok(Duration::from_millis(10))));
                observed.record(EdgeWeightType::Intermediate(Ok(Duration::from_millis(10))));
                observed.record(EdgeWeightType::Balance(Some(Balance::from(1_000_000_000u64))));
            });
            chain = chain.channel(
                ChannelEntry::builder()
                    .between(addresses[from], addresses[to])
                    .amount(1_000_000_000)
                    .ticket_index(1)
                    .status(ChannelStatus::Open)
                    .epoch(1)
                    .build()
                    .expect("benchmark channel"),
            );
        }
    }
    let config = PathPlannerConfig {
        max_cached_paths: CANDIDATES,
        min_paths_anonymity_floor: CANDIDATES,
        cache_ttl: Duration::from_secs(3600),
        refresh_period: Duration::from_secs(3600),
        ..Default::default()
    };
    let selector = HoprGraphPathSelector::new(
        source,
        graph,
        config.max_cached_paths,
        config.edge_penalty,
        config.min_ack_rate,
        config.min_paths_anonymity_floor,
    );
    let planner = PathPlanner::new(
        source,
        MemorySurbStore::default(),
        chain.build(),
        selector.clone(),
        config,
    );
    let routing = DestinationRouting::Forward {
        destination: Box::new(NodeId::Offchain(destination)),
        pseudonym: Some(HoprPseudonym::from([7u8; HoprPseudonym::SIZE])),
        forward_options: RoutingOptions::Hops(1.try_into().expect("one relay")),
        return_options: Some(RoutingOptions::Hops(1.try_into().expect("one relay"))),
    };
    Fixture {
        source,
        destination,
        selector,
        planner,
        routing,
    }
}

fn routing_tracing(c: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("benchmark runtime");
    let f = fixture();
    assert_eq!(
        f.selector
            .select_path(f.source, f.destination, 1)
            .expect("search")
            .len(),
        CANDIDATES
    );
    let session_span = tracing::debug_span!("session", session_id = "benchmark");
    let (warm, ..) = runtime
        .block_on(
            f.planner
                .resolve_routing(0, 2, f.routing.clone())
                .instrument(session_span.clone()),
        )
        .expect("warm both routing cache entries");
    let ResolvedTransportRouting::Forward { return_paths, .. } = warm else {
        panic!("forward fixture");
    };
    assert_eq!(return_paths.len(), 2, "benchmark must draw two return routes");
    let mut forward_only = f.routing.clone();
    if let DestinationRouting::Forward { return_options, .. } = &mut forward_only {
        *return_options = None;
    }

    let mut group = c.benchmark_group("routing_tracing");
    group.bench_function("selector_16_candidates", |b| {
        let _entered = session_span.enter();
        b.iter(|| black_box(f.selector.select_path(f.source, f.destination, 1).expect("search")));
    });
    group.bench_function("cached_forward", |b| {
        b.to_async(&runtime).iter(|| async {
            black_box(
                f.planner
                    .resolve_routing(0, 0, forward_only.clone())
                    .instrument(session_span.clone())
                    .await
                    .expect("cached forward"),
            )
        });
    });
    group.bench_function("cached_forward_return_2", |b| {
        b.to_async(&runtime).iter(|| async {
            black_box(
                f.planner
                    .resolve_routing(0, 2, f.routing.clone())
                    .instrument(session_span.clone())
                    .await
                    .expect("cached forward and return routes"),
            )
        });
    });
    group.bench_function("rebuild_16_candidates", |b| {
        b.to_async(&runtime).iter(|| async {
            black_box(
                f.planner
                    .recompute_paths_from(&f.source)
                    .instrument(session_span.clone())
                    .await,
            )
        });
    });

    let mut session = HoprSession::new(
        HoprPseudonym::from([7u8; HoprPseudonym::SIZE]),
        f.routing.clone(),
        HoprSessionConfig::default(),
        (futures::sink::drain(), stream::pending()),
        None,
    )
    .expect("raw session writing to a ready sink");
    let payload = [0x42; 300];
    group.bench_function("session_write_300_bytes", |b| {
        b.iter(|| {
            runtime
                .block_on(session.write_all(black_box(&payload)))
                .expect("session write")
        });
    });
    group.finish();
}

fn configuration() -> Criterion {
    let level = std::env::var("HOPR_BENCH_LOG")
        .unwrap_or_else(|_| "info".into())
        .parse::<tracing::Level>()
        .expect("HOPR_BENCH_LOG is a tracing level");
    test_log::tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::sink)
        .with_ansi(false)
        .without_time()
        .init();
    Criterion::default()
        .sample_size(50)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(5))
        .nresamples(20_000)
}

criterion_group! { name = benches; config = configuration(); targets = routing_tracing }
criterion_main!(benches);
