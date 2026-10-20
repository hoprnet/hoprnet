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
use hopr_crypto_packet::{HoprPixSpec, prelude::HoprPixGroupElement};
use hopr_protocol_app::{
    prelude::ApplicationDataOut,
    v1::{ApplicationData, ApplicationDataIn, ReservedTag},
};
use hopr_protocol_pix::{GroupEncoding, PixParams, SsaIndex};
use hopr_protocol_start::{
    KeepAliveFlag, KeepAliveMessage, SsaServerCommitmentMessage, StartChallenge, StartInitiation,
};
use hopr_transport_session::{
    DispatchResult, DropReason, HoprSessionCapabilities, HoprStartProtocol, IncomingSession, LOCAL_PIX_SUITE,
    SessionId, SessionManager, SessionManagerConfig, SessionTarget,
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
    make_start_message_data(msg)
}

/// Encodes any [`HoprStartProtocol`] message into the packet form `dispatch_message` takes.
///
/// The Start tag is what `dispatch_message` routes on, exactly as the crate's own `start_packet`
/// test helper builds its packets — but via [`HoprStartProtocol::encode`] rather than
/// `ApplicationData::try_from`, because the bench asserts on the tag it is about to assert against
/// `dispatch_message`'s own routing comparison. The tag comes back from `encode` itself, so the two
/// cannot drift: a future change of [`HoprStartProtocol::START_PROTOCOL_MESSAGE_TAG`] re-tags every
/// packet this bench sends automatically, instead of silently sending one no longer routed.
///
/// A plain `assert_eq!` rather than `debug_assert_eq!`: `[profile.bench]` does not enable
/// `debug-assertions`, so a `debug_assert!` here would compile out of `cargo bench` and the tag
/// guard would never run.
fn make_start_message_data(msg: HoprStartProtocol) -> ApplicationDataIn {
    let (tag, bytes) = msg.encode().unwrap();
    assert_eq!(
        tag,
        HoprStartProtocol::START_PROTOCOL_MESSAGE_TAG,
        "the encoded Start protocol message must carry the Start protocol tag, or dispatch_message would not route it"
    );
    ApplicationDataIn {
        data: ApplicationData::new(tag, bytes.into_vec()).unwrap(),
        packet_info: Default::default(),
    }
}

/// The PIX dimensions this bench builds `SsaRequest`s under.
///
/// Not the deployed ones — the message is only ever decoded and its `params` echoed back, so any
/// legal quadruple exercises the same path — but taken from the crate's constants so they cannot
/// drift into an illegal combination that fails `PixParams::try_new`.
fn bench_pix_params() -> PixParams {
    PixParams::try_new(
        hopr_transport_session::DEFAULT_PIX_POLYS_PER_SSA,
        hopr_transport_session::DEFAULT_PIX_SHARES_PER_POLY,
        hopr_transport_session::DEFAULT_PIX_SURPLUS_SHARES,
        LOCAL_PIX_SUITE,
    )
    .expect("the crate's own PIX dimensions must be valid")
}

/// One SSA commitment at `index`, carrying the identity element.
///
/// As in the crate's `ssa_request_for` test helper: the Exit-side point is not checked in dispatch,
/// so the identity element is enough for the message to be decodable and well-formed.
fn ssa_request_commitments(index: u32) -> std::collections::BTreeMap<SsaIndex, HoprPixGroupElement> {
    let identity = HoprPixGroupElement::try_from(
        hopr_protocol_pix::PixGroup::<HoprPixSpec>::default()
            .to_bytes()
            .as_ref(),
    )
    .expect("identity element must be valid");
    std::collections::BTreeMap::from([(SsaIndex::new(index).expect("non-zero"), identity)])
}

/// An `SsaRequest` for `session_id` with a single identity-element commitment.
fn make_ssa_request_data(session_id: SessionId) -> ApplicationDataIn {
    make_start_message_data(HoprStartProtocol::SsaRequest(SsaServerCommitmentMessage::new(
        session_id,
        bench_pix_params(),
        ssa_request_commitments(1),
        [],
    )))
}

/// A `KeepAlive` for `session_id` carrying only the balancer-state flag.
fn make_keep_alive_data(session_id: SessionId) -> ApplicationDataIn {
    make_start_message_data(HoprStartProtocol::KeepAlive(KeepAliveMessage {
        session_id,
        flags: KeepAliveFlag::BalancerState.into(),
        additional_data: 0,
    }))
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

/// Benchmark: the Start-protocol admission filter's per-variant lookups.
///
/// `dispatch_message` runs every Start message through
/// `start_message_is_for_local_state` before enqueueing it, dropping with
/// `DropReason::StaleStartMessage` anything naming no local Session state. The filter exists
/// because Start messages share one bounded worker pool, and several of its handlers park a
/// worker on state a bogus message will never have — `handle_ssa_request` waits up to a second
/// for a Session slot a random pseudonym never gets. Before the filter, ~110 concurrent bogus
/// requests occupied every handler and the bounded Start queue then dropped all traffic behind
/// them, including the KeepAlives of live Sessions (#8514).
///
/// # Why this group and not `dispatch_start_protocol`
///
/// [`dispatch_start_protocol`] sends only `StartSession`, which is the one arm the filter answers
/// with a constant `true` — no lookup of any kind. A regression that turned the *lookup* arms into
/// something expensive is therefore invisible to it. Each case below pins one arm whose cost is a
/// real cache probe:
///
/// * `ssa_request_forward_slot` — the legitimate, hot case: the Exit asking its own Entry for SSA commitments of a
///   Session this node holds as an outgoing (`Forward`) slot. The filter pays `pending_outgoing.contains_key` (a miss)
///   and then `sessions.get` plus a `Forward` variant match. Enqueued (`Processed`), and so also reaches the credit —
///   as does `keep_alive_registered` below, the only other case here that passes the filter.
/// * `ssa_request_stranger` — the attack case: an `SsaRequest` for a pseudonym this node never issued. The filter pays
///   the same two probes and both miss, then drops without enqueueing. This is the path the DoS floods with, and its
///   cost must stay a cache-miss lookup: anything that grows it — a log line per drop, a lock, a decode of the payload
///   — is itself a lever a peer can ride, which is exactly what the filter was written to remove. Read it against
///   `ssa_request_forward_slot`: the gap between them is what a *hit* adds, and the absolute is what every packet on
///   the wire costs when under flood.
/// * `keep_alive_registered` — the steady-state control-traffic case: a `KeepAlive` for a live Session. One
///   `sessions.contains_key` probe, then the enqueue, the `Forward` check and the credit that
///   `dispatch_start_protocol`'s `session_hit_forward` also measures — so the interesting read is against that case,
///   which carries the identical tail but no `contains_key`.
///
/// Like the neighbouring groups, none of these fixtures is new: `make_manager_with_session` and
/// `make_manager_without_session` are exactly what `dispatch_start_protocol` uses, so a change in
/// the filter's cost cannot be masked by a change in the fixture. The intended branch is checked
/// once per benchmark *run* — before `b.iter` is ever called — by a plain `assert!(matches!(...))`
/// over the same `dispatch_message` call the closure then measures, outside the timed section:
/// asserting inside `b.iter` would put the assert in the measurement, and it is only ever needed
/// once, since the fixture is fixed for the life of the group. It is a plain `assert!` rather than
/// `debug_assert!` because `[profile.bench]` does not enable `debug-assertions`, so a
/// `debug_assert!` would compile out of `cargo bench` and the check would silently never run. The
/// timed closures themselves discard the result exactly as the existing ones do.
pub fn dispatch_start_protocol_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_start_protocol_filter");
    group.sample_size(BENCHMARK_SAMPLE_COUNT);

    // `SsaRequest` for a Session held as an outgoing (Forward) slot: the Exit's own Entry. Passes
    // the filter on the `sessions.get` + `Forward` match, and is enqueued.
    {
        let (msg_rx, sn_rx, manager, session_id, runtime) = make_manager_with_session(forward_routing);
        let in_data = make_ssa_request_data(session_id);

        assert!(
            matches!(
                manager.dispatch_message(session_id, in_data.clone()),
                Ok(DispatchResult::Processed)
            ),
            "fixture must take the filter's Forward-slot arm: the SsaRequest names a session held as an outgoing \
             (Forward) slot, so dispatch_message must enqueue it, not drop it"
        );

        group.bench_function("ssa_request_forward_slot", |b| {
            b.iter(|| {
                let _ = manager.dispatch_message(session_id, in_data.clone()).ok();
            });
        });

        drop(std::mem::ManuallyDrop::into_inner(msg_rx));
        drop(std::mem::ManuallyDrop::into_inner(sn_rx));
        runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    }

    // `SsaRequest` for a pseudonym this node never issued: the DoS flood. Fails the filter after
    // two cache-miss probes and is dropped before reaching the queue.
    {
        let (msg_rx, sn_rx, manager, runtime) = make_manager_without_session();
        let stranger = HoprPseudonym::random();
        let in_data = make_ssa_request_data(stranger);

        assert!(
            matches!(
                manager.dispatch_message(stranger, in_data.clone()),
                Ok(DispatchResult::Dropped(DropReason::StaleStartMessage))
            ),
            "fixture must take the filter's drop arm: the SsaRequest names a pseudonym this node never issued, so \
             dispatch_message must drop it before enqueueing"
        );

        group.bench_function("ssa_request_stranger", |b| {
            b.iter(|| {
                let _ = manager.dispatch_message(stranger, in_data.clone()).ok();
            });
        });

        drop(std::mem::ManuallyDrop::into_inner(msg_rx));
        drop(std::mem::ManuallyDrop::into_inner(sn_rx));
        runtime.shutdown_timeout(std::time::Duration::from_millis(100));
    }

    // `KeepAlive` for a live Session: the steady-state control traffic the flood above must not be
    // able to starve. Passes the filter on a single `contains_key`, and is enqueued.
    {
        let (msg_rx, sn_rx, manager, session_id, runtime) = make_manager_with_session(forward_routing);
        let in_data = make_keep_alive_data(session_id);

        assert!(
            matches!(
                manager.dispatch_message(session_id, in_data.clone()),
                Ok(DispatchResult::Processed)
            ),
            "fixture must take the filter's KeepAlive arm: the KeepAlive names a registered session, so \
             dispatch_message must enqueue it, not drop it"
        );

        group.bench_function("keep_alive_registered", |b| {
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
    dispatch_start_protocol_filter,
    dispatch_session_hit,
    dispatch_session_miss,
    dispatch_unrelated
);
criterion_main!(benches);
