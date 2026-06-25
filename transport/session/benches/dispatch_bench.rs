//! Criterion benchmarks for [`SessionManager::dispatch_message`](crate::SessionManager::dispatch_message).
//!
//! Run with: `cargo bench -p hopr-transport-session --features benchmark,runtime-tokio -- dispatch_bench`

use std::net::SocketAddr;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hopr_api::types::{
    crypto_random::Randomizable,
    internal::{prelude::HoprPseudonym, routing::DestinationRouting},
};
use hopr_protocol_app::{
    prelude::ApplicationDataOut,
    v1::{ApplicationData, ApplicationDataIn, Tag},
};
use hopr_protocol_start::{StartChallenge, StartInitiation, StartProtocol};
use hopr_transport_session::{
    ByteCapabilities, SESSION_APPLICATION_TAG, SESSION_FORWARD_CAPACITY, SessionId, SessionManager,
    SessionManagerConfig, SessionTarget,
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

const START_PROTOCOL_MESSAGE_TAG: Tag = Tag::Reserved(3);

/// Builds valid `ApplicationDataIn` with the Start protocol tag and a dummy encoded payload.
fn make_start_protocol_data() -> ApplicationDataIn {
    let challenge = StartChallenge::MAX;
    let target = SessionTarget::UdpStream(SealedHost::Plain(SocketAddr::from(([127, 0, 0, 1], 13301)).into()));
    let msg = StartProtocol::<SessionId, SessionTarget, ByteCapabilities>::StartSession(StartInitiation {
        challenge,
        target,
        capabilities: ByteCapabilities::try_from(0u8).unwrap(),
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
        data: ApplicationData::new(SESSION_APPLICATION_TAG, vec![0u8; size]).unwrap(),
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
///
/// The `SessionManager` requires `S: Sink<(DestinationRouting, ApplicationDataOut)>`.
/// `futures::channel::mpsc::UnboundedSender` satisfies this bound.
type BenchSink = futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>;

/// Creates a [`SessionManager`] with one session pre-populated in the cache, so that
/// `dispatch_message` hits the session-lookup + channel-send path.
///
/// The internal receiver is actively drained by a background task so that `try_send`
/// never blocks during the benchmark iterations.
fn make_manager_with_session() -> (SessionManager<BenchSink>, SessionId) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session_id = SessionId::random();

    let cfg = SessionManagerConfig {
        session_forward_capacity: SESSION_FORWARD_CAPACITY,
        ..Default::default()
    };
    let manager = SessionManager::new(cfg);

    let (msg_sender, _msg_receiver) = futures::channel::mpsc::unbounded();
    let (session_notifier, _session_notifier_rx) = futures::channel::mpsc::channel(1);
    manager
        .start(msg_sender, session_notifier)
        .expect("manager.start() must succeed");

    // The helper constructs the SessionSlot internally.
    manager.pre_populate_session(session_id, DestinationRouting::Return(session_id.into()));

    // Drain the session receiver so the channel never fills up.
    let (_session_tx, session_rx) =
        crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(SESSION_FORWARD_CAPACITY);
    // AsyncRx::recv() is the idiomatic polling API — use it in a loop.
    runtime.spawn(async move {
        loop {
            match session_rx.recv().await {
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });

    (manager, session_id)
}

/// Creates a [`SessionManager`] in the started state with no sessions in the cache.
fn make_manager_without_session() -> SessionManager<BenchSink> {
    let cfg = SessionManagerConfig {
        session_forward_capacity: SESSION_FORWARD_CAPACITY,
        ..Default::default()
    };
    let manager = SessionManager::new(cfg);

    let (msg_sender, _msg_receiver) = futures::channel::mpsc::unbounded();
    let (session_notifier, _session_notifier_rx) = futures::channel::mpsc::channel(1);
    manager
        .start(msg_sender, session_notifier)
        .expect("manager.start() must succeed");

    manager
}

/// Benchmark: dispatching a Start protocol message.
///
/// Exercises the `start_protocol_tx.try_send()` path.
pub fn dispatch_start_protocol(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_start_protocol");
    group.sample_size(50_000);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    let manager = make_manager_without_session();
    let pseudonym = HoprPseudonym::random();
    let in_data = make_start_protocol_data();

    group.bench_function("single", |b| {
        b.iter(|| {
            manager
                .dispatch_message(pseudonym, in_data.clone())
                .expect("dispatch must succeed")
        });
    });
    group.finish();
}

/// Benchmark: dispatching a Session data message where the session exists in the cache.
///
/// Exercises the moka cache lookup + `session_slot.session_tx.try_send()` path.
pub fn dispatch_session_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_session_hit");
    group.sample_size(50_000);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    #[inline]
    fn payload_sizes() -> &'static [usize] {
        if cfg!(feature = "all-benchmarks") {
            &[16, 256, 1024, 1018]
        } else {
            &[1018]
        }
    }

    for size in payload_sizes() {
        let (manager, session_id) = make_manager_with_session();
        let in_data = make_session_data(*size);
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("cache_hit", size), size, |b, _| {
            b.iter(|| {
                manager
                    .dispatch_message(session_id, in_data.clone())
                    .expect("dispatch must succeed")
            });
        });
    }
    group.finish();
}

/// Benchmark: dispatching a Session data message where the session is NOT in the cache.
///
/// Exercises the moka cache lookup + `None` branch.  This is the "attacker noise" path —
/// packets that arrive for sessions the node has forgotten.
pub fn dispatch_session_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_session_miss");
    group.sample_size(50_000);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    let manager = make_manager_without_session();
    let unknown_session_id = SessionId::random();
    let in_data = make_session_data(1018);

    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            manager
                .dispatch_message(unknown_session_id, in_data.clone())
                .expect("dispatch must succeed")
        });
    });
    group.finish();
}

/// Benchmark: dispatching a message with a tag that matches neither protocol.
///
/// The shortest path — just a tag comparison and `Ok(DispatchResult::Unrelated(...))`.
pub fn dispatch_unrelated(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_unrelated");
    group.sample_size(50_000);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Elements(1));

    let manager = make_manager_without_session();
    let pseudonym = HoprPseudonym::random();
    let in_data = make_unrelated_data();

    group.bench_function("single", |b| {
        b.iter(|| {
            manager
                .dispatch_message(pseudonym, in_data.clone())
                .expect("dispatch must succeed")
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    dispatch_start_protocol,
    dispatch_session_hit,
    dispatch_session_miss,
    dispatch_unrelated
);
criterion_main!(benches);
