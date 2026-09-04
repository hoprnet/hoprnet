//! Criterion benchmarks for [`SessionManager::dispatch_message`](crate::SessionManager::dispatch_message).
//!
//! Run with: `cargo bench -p hopr-transport-session --features benchmark,runtime-tokio -- dispatch_bench`
//!
//! ## Benchmark strategy
//!
//! `dispatch_message` is sub-microsecond. Criterion's adaptive sampler would calculate
//! millions of iterations per sample, overflowing internal bounded tokio channels.
//!
//! Solution: each criterion sample is one *batch* of `BATCH_SIZE` calls to `dispatch_message`.
//! `iter_custom` runs the batch once and returns the batch duration as a `Duration`, so
//! criterion records one measurement (batch wall-clock time) per sample.  The batch is large
//! enough that each sample takes several seconds — well above criterion's measurement floor.
//!
//! `block_on` is used only where the async executor is actually required:
//! - `dispatch_session_hit`: the session channel is `bounded_blocking_async`; its `try_send` falls back to
//!   `spawn_blocking` when full, which needs a Tokio context.
//! - All other paths use `bounded_async` channels or no channels at all, so `block_on` is omitted to avoid unnecessary
//!   executor overhead in the timed section.

use std::net::SocketAddr;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use futures::StreamExt;
use hopr_api::types::{
    crypto::{keypairs::ChainKeypair, prelude::Keypair},
    crypto_random::Randomizable,
    internal::{
        prelude::HoprPseudonym,
        routing::{DestinationRouting, RoutingOptions},
    },
    primitive::prelude::Address,
};
use hopr_protocol_app::{
    prelude::{ApplicationDataOut, Tag::Reserved},
    v1::{ApplicationData, ApplicationDataIn, ReservedTag, Tag},
};
use hopr_protocol_start::{StartChallenge, StartInitiation, StartProtocol};
use hopr_transport_session::{
    HoprSessionCapabilities, HoprStartProtocol, IncomingSession, SessionId, SessionManager, SessionManagerConfig,
    SessionTarget,
};
use hopr_utils::network_types::prelude::SealedHost;

// Avoid musl's default allocator due to degraded performance.
//
// https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance
#[cfg(all(feature = "allocator-mimalloc", feature = "allocator-jemalloc"))]
compile_error!("feature \"allocator-jemalloc\" and feature \"allocator-mimalloc\" cannot be enabled at the same time");
#[cfg(all(target_os = "linux", feature = "allocator-mimalloc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
#[cfg(all(target_os = "linux", feature = "allocator-jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Internal tokio channel capacity used in benchmarks.
///
/// Set to `BENCHMARK_CHANNEL_CAPACITY` in `SessionManagerConfig`.  The internal tokio
/// processing task drains the session channel at ~3–4M msg/s; a 1M-message batch takes
/// ~300ms to drain, during which `try_send` on the (already-full) slot channel will
/// block, providing natural back-pressure without overflow.
const BENCHMARK_CHANNEL_CAPACITY: usize = 2_000_000;

/// Number of criterion measurement samples per benchmark.
///
/// 10 000 samples gives a tight confidence interval (~0.2% at p = 0.05) for sub-microsecond
/// benchmarks without excessive runtime.
const BENCHMARK_SAMPLE_COUNT: usize = 10_000;

const START_PROTOCOL_MESSAGE_TAG: Tag = Tag::Reserved(3);

/// Builds valid `ApplicationDataIn` with the Start protocol tag and a dummy encoded payload.
fn make_start_protocol_data() -> ApplicationDataIn {
    let challenge = StartChallenge::MAX;
    let target = SessionTarget::UdpStream(SealedHost::Plain(SocketAddr::from(([127, 0, 0, 1], 13301)).into()));
    let msg = HoprStartProtocol::StartSession(StartInitiation {
        challenge,
        target,
        capabilities: HoprSessionCapabilities::try_from(0u8).unwrap(),
        additional_data: 0,
    });
    let (tag, bytes) = msg.encode().unwrap();
    debug_assert_eq!(tag, START_PROTOCOL_MESSAGE_TAG);
    ApplicationDataIn {
        data: ApplicationData::new(tag, bytes.into_vec()).unwrap(),
        packet_info: Default::default(),
    }
}

/// Builds valid `ApplicationDataIn` with the Session protocol tag and fixed payload.
fn make_session_data(size: usize) -> ApplicationDataIn {
    ApplicationDataIn {
        data: ApplicationData::new(ReservedTag::Session, vec![0u8; size]).unwrap(),
        packet_info: Default::default(),
    }
}

/// Builds `ApplicationDataIn` with a tag that matches neither protocol.
fn make_unrelated_data() -> ApplicationDataIn {
    ApplicationDataIn {
        data: ApplicationData::new(99u64, b"unrelated".to_vec()).unwrap(),
        packet_info: Default::default(),
    }
}

/// Sink type used in benchmarks.
type BenchSink = futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>;

/// Application data type sent over session channels.
type BenchSessionData = hopr_protocol_app::v1::ApplicationDataIn;

/// An outgoing (Entry-side) slot's routing.
///
/// The destination is never read — `dispatch_message` only matches on the *variant*, because what
/// it is asking is which side of the Session this node is on. A random address is therefore as
/// faithful as a real one and does not drag a peer fixture into the benchmark.
fn forward_routing(session_id: SessionId) -> DestinationRouting {
    let destination: Address = (&ChainKeypair::random()).into();
    DestinationRouting::Forward {
        destination: Box::new(destination.into()),
        pseudonym: Some(session_id),
        forward_options: RoutingOptions::Hops(hopr_api::types::primitive::bounded::BoundedSize::MIN),
        return_options: RoutingOptions::Hops(hopr_api::types::primitive::bounded::BoundedSize::MIN).into(),
    }
}

/// Creates a [`SessionManager`] with one session pre-populated in the cache.
///
/// `routing` receives the freshly minted [`SessionId`] because the id is created here, and both
/// routing variants want to name it: `Return` carries it directly, `Forward` carries it as the
/// Session's fixed pseudonym. Which variant the slot holds decides whether `dispatch_message`
/// credits Start-protocol traffic to `returned_packets`, so it is a parameter rather than a
/// constant.
#[allow(clippy::type_complexity)]
fn make_manager_with_session(
    routing: impl FnOnce(SessionId) -> DestinationRouting,
) -> (
    std::mem::ManuallyDrop<futures::channel::mpsc::UnboundedReceiver<(DestinationRouting, ApplicationDataOut)>>,
    std::mem::ManuallyDrop<futures::channel::mpsc::Receiver<IncomingSession>>,
    SessionManager<BenchSink>,
    SessionId,
    tokio::runtime::Runtime,
) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let session_id = SessionId::random();

    let cfg = SessionManagerConfig {
        maximum_sessions: BENCHMARK_CHANNEL_CAPACITY,
        session_forward_capacity: BENCHMARK_CHANNEL_CAPACITY,
        ..Default::default()
    };
    let manager = SessionManager::new(cfg);

    let (msg_sender, msg_receiver) = futures::channel::mpsc::unbounded();
    let (session_notifier, session_notifier_rx) = futures::channel::mpsc::channel(1);

    runtime.block_on(async {
        manager
            .start(msg_sender, session_notifier, None, None)
            .expect("manager.start() must succeed");
    });

    let session_rx = manager.pre_populate_session_with_receiver(session_id, routing(session_id));

    // Background drain task: keeps the pre-populated session's tx channel from filling up.
    // `pre_populate_session_with_receiver` provides the rx end so the benchmark can drive it.
    runtime.spawn(async move {
        session_rx.into_stream().for_each(|_: BenchSessionData| async {}).await;
    });

    (
        std::mem::ManuallyDrop::new(msg_receiver),
        std::mem::ManuallyDrop::new(session_notifier_rx),
        manager,
        session_id,
        runtime,
    )
}

/// Creates a [`SessionManager`] in the started state with no sessions in the cache.
#[allow(clippy::type_complexity)]
fn make_manager_without_session() -> (
    std::mem::ManuallyDrop<futures::channel::mpsc::UnboundedReceiver<(DestinationRouting, ApplicationDataOut)>>,
    std::mem::ManuallyDrop<futures::channel::mpsc::Receiver<IncomingSession>>,
    SessionManager<BenchSink>,
    tokio::runtime::Runtime,
) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let cfg = SessionManagerConfig {
        maximum_sessions: BENCHMARK_CHANNEL_CAPACITY,
        session_forward_capacity: BENCHMARK_CHANNEL_CAPACITY,
        ..Default::default()
    };
    let manager = SessionManager::new(cfg);

    let (msg_sender, msg_receiver) = futures::channel::mpsc::unbounded();
    let (session_notifier, session_notifier_rx) = futures::channel::mpsc::channel(1);

    runtime.block_on(async {
        manager
            .start(msg_sender, session_notifier, None, None)
            .expect("manager.start() must succeed");
    });

    (
        std::mem::ManuallyDrop::new(msg_receiver),
        std::mem::ManuallyDrop::new(session_notifier_rx),
        manager,
        runtime,
    )
}

/// Benchmark: dispatching a Start protocol message.
///
/// Exercises the `start_protocol_tx.try_send()` path — and, since the Exit's Start-protocol
/// traffic is credited to the Entry's `returned_packets`, the session lookup that credit needs.
///
/// # Why three ids and not one
///
/// `dispatch_message` runs a `sessions.get()` on *every* Start message so an Exit → Entry packet
/// can be counted: it consumed a return SURB and therefore unlocked a PIX share, which is what the
/// Entry's successor gate is denominated in. The lookup's cost depends on two things this
/// benchmark used to fix silently, both at their cheapest:
///
/// * whether the session is in the cache. `single` **misses**, which is the empty-manager case and no longer the
///   production one — an Entry Session receiving Start traffic is by definition registered.
/// * which side of the Session the slot is. Only `DestinationRouting::Forward` — the Entry — reaches the counter; a
///   `Return` slot is the Exit receiving its own peer's traffic and is deliberately not credited.
///
/// So `single` stays as the miss control and the two `session_hit_*` ids measure the arms that
/// actually run. Read each against `single` — that gap is what a registered session costs the
/// Start path over an empty manager, which is the comparison the miss-only fixture could not make.
///
/// Do **not** read the gap *between* the two hit ids as the cost of the credit. They are not a
/// controlled pair: the slot each clones carries a different `routing_opts` payload —
/// `Forward` boxes a `NodeId` and holds two `RoutingOptions`, `Return` holds a `SurbMatcher` — so
/// any difference mixes that with the `fetch_add`. Isolating the credit would need two `Forward`
/// slots differing only in whether the counter fires, which is not constructible from outside the
/// crate. In practice the two sit inside each other's run-to-run spread, which is the useful
/// finding: at this scale the lookup dominates and neither the payload nor the increment is
/// separable from it.
pub fn dispatch_start_protocol(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_start_protocol");
    group.sample_size(BENCHMARK_SAMPLE_COUNT);

    // Cache miss: no session registered, so the lookup is the shortest it can be.
    {
        let (msg_rx, sn_rx, manager, runtime) = make_manager_without_session();
        let id = HoprPseudonym::random();
        let in_data = make_start_protocol_data();

        group.bench_function("single", |b| {
            b.iter(|| {
                let _ = manager.dispatch_message(id, in_data.clone()).ok();
            });
        });

        // Drop receivers first so channels close and processing tasks get EOF.
        drop(std::mem::ManuallyDrop::into_inner(msg_rx));
        drop(std::mem::ManuallyDrop::into_inner(sn_rx));
        // Explicit shutdown prevents the tokio runtime's Drop from blocking forever.
        runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    }

    // Cache hit on an outgoing (Entry) session: lookup, slot clone, and the credit.
    {
        let (msg_rx, sn_rx, manager, session_id, runtime) = make_manager_with_session(forward_routing);
        let in_data = make_start_protocol_data();

        group.bench_function("session_hit_forward", |b| {
            b.iter(|| {
                let _ = manager.dispatch_message(session_id, in_data.clone()).ok();
            });
        });

        drop(std::mem::ManuallyDrop::into_inner(msg_rx));
        drop(std::mem::ManuallyDrop::into_inner(sn_rx));
        runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    }

    // Cache hit on an incoming (Exit) session: lookup and slot clone, no credit.
    {
        let (msg_rx, sn_rx, manager, session_id, runtime) =
            make_manager_with_session(|id| DestinationRouting::Return(id.into()));
        let in_data = make_start_protocol_data();

        group.bench_function("session_hit_return", |b| {
            b.iter(|| {
                let _ = manager.dispatch_message(session_id, in_data.clone()).ok();
            });
        });

        drop(std::mem::ManuallyDrop::into_inner(msg_rx));
        drop(std::mem::ManuallyDrop::into_inner(sn_rx));
        runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    }

    group.finish();
}

/// Benchmark: dispatching a Session data message where the session exists in the cache.
///
/// Exercises the moka cache lookup + `session_slot.session_tx.try_send()` path.
/// `block_on` IS required — the session channel is `bounded_blocking_async` and its `try_send`
/// falls back to `spawn_blocking` when the buffer is full, which needs a Tokio context.
pub fn dispatch_session_hit(c: &mut Criterion) {
    #[inline]
    fn payload_sizes() -> &'static [usize] {
        if cfg!(feature = "all-benchmarks") {
            &[16, 256, 1024, 1018]
        } else {
            &[1018]
        }
    }

    let mut group = c.benchmark_group("dispatch_session_hit");
    group.sample_size(BENCHMARK_SAMPLE_COUNT);

    for &size in payload_sizes() {
        // `Return`, i.e. the Exit side, which is the side that receives Session *data*. Held
        // deliberately at the value this fixture always used, so the id stays comparable.
        let (msg_rx, sn_rx, manager, session_id, runtime) =
            make_manager_with_session(|id| DestinationRouting::Return(id.into()));
        let in_data = make_session_data(size);

        group.bench_with_input(BenchmarkId::new("cache_hit", size), &size, |b, _| {
            b.iter(|| {
                let _ = manager.dispatch_message(session_id, in_data.clone()).ok();
            });
        });

        drop(std::mem::ManuallyDrop::into_inner(msg_rx));
        drop(std::mem::ManuallyDrop::into_inner(sn_rx));
        runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    }
    group.finish();
}

/// Benchmark: dispatching a Session data message where the session is NOT in the cache.
pub fn dispatch_session_miss(c: &mut Criterion) {
    let (msg_rx, sn_rx, manager, runtime) = make_manager_without_session();
    let unknown_session_id = SessionId::random();
    let in_data = make_session_data(1018);

    let mut group = c.benchmark_group("dispatch_session_miss");
    group.sample_size(BENCHMARK_SAMPLE_COUNT);

    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            let _ = manager
                .dispatch_message(unknown_session_id.clone(), in_data.clone())
                .ok();
        });
    });
    group.finish();

    drop(std::mem::ManuallyDrop::into_inner(msg_rx));
    drop(std::mem::ManuallyDrop::into_inner(sn_rx));
    runtime.shutdown_timeout(std::time::Duration::from_millis(100));
}

/// Benchmark: dispatching a message with a tag that matches neither protocol.
///
/// The shortest path — just two tag comparisons and `Ok(DispatchResult::Unrelated(...))`.
pub fn dispatch_unrelated(c: &mut Criterion) {
    let (msg_rx, sn_rx, manager, runtime) = make_manager_without_session();
    let id = HoprPseudonym::random();
    let in_data = make_unrelated_data();

    let mut group = c.benchmark_group("dispatch_unrelated");
    group.sample_size(BENCHMARK_SAMPLE_COUNT);

    group.bench_function("single", |b| {
        b.iter(|| {
            let _ = manager.dispatch_message(id.clone(), in_data.clone()).ok();
        });
    });
    group.finish();

    drop(std::mem::ManuallyDrop::into_inner(msg_rx));
    drop(std::mem::ManuallyDrop::into_inner(sn_rx));
    runtime.shutdown_timeout(std::time::Duration::from_millis(100));
}

criterion_group!(
    benches,
    dispatch_start_protocol,
    dispatch_session_hit,
    dispatch_session_miss,
    dispatch_unrelated
);
criterion_main!(benches);
