use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex, OnceLock, atomic::Ordering},
    time::{Duration, Instant},
};

use anyhow::anyhow;
use futures::{Sink, SinkExt, StreamExt, TryStreamExt, channel::oneshot, future::AbortHandle};
use futures_time::future::FutureExt as TimeExt;
use hopr_api::types::{
    crypto_random::Randomizable,
    internal::{
        prelude::HoprPseudonym,
        routing::{DestinationRouting, RoutingOptions},
    },
    primitive::prelude::Address,
};
use hopr_crypto_packet::{
    HoprPixSpec,
    prelude::{HOPR_PIX_COMMITMENT_PROOF_SIZE, HoprPacket, HoprPixCommitmentProof, HoprPixGroupElement},
};
use hopr_protocol_app::prelude::*;
use hopr_protocol_pix::{
    EntryShareGenerator, ExitAcknowledgementShareProcessor, GroupEncoding, MAX_POLYS_PER_SSA, PixParams, PixSpec,
    SsaCommitmentGuard, SsaId, SsaIndex, SsaReconstructor, SsaShareGenerator,
};
use hopr_protocol_start::{
    ErrorIdentifier, KeepAliveFlag, KeepAliveMessage, SsaClientCommitmentMessage, SsaServerCommitmentMessage,
    StartChallenge, StartErrorReason, StartErrorType, StartEstablished, StartInitiation,
};
use hopr_utils::runtime::AbortableList;
use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "telemetry")]
use crate::telemetry::{
    self, SessionLifecycleState, initialize_session_metrics, remove_session_metrics_state, set_session_balancer_data,
    set_session_state,
};
use crate::{
    AgreedSsaQuota, Capabilities, Capability, HoprSession, HoprSessionOutPixEvent, IncomingSession, SESSION_MTU,
    SessionClientConfig, SessionId, SessionTarget, SurbBalancerConfig,
    balancer::{
        AtomicSurbFlowEstimator, BalancerStateValues, RateController, RateLimitSinkExt, SurbBalancer,
        SurbControllerWithCorrection,
        pid::{PidBalancerController, PidControllerGains},
        simple::SimpleBalancerController,
    },
    errors::{self, SessionManagerError, TransportSessionError},
    supervision::{
        ActionRx, ServiceGate, SessionPixAction, SessionPixCloseReason, SessionPixEvent, SessionPixSupervisorHandle,
        SupervisorConfig, spawn_supervisor_worker,
    },
    types::{
        ClosureReason, DEFAULT_PIX_PARAMS, DEFAULT_PIX_QUOTA_RANGE_SPAN, DEFAULT_PIX_SSA_QUOTA, HoprPixDepositData,
        HoprSessionCapabilities, HoprSessionConfig, HoprSessionInPixEvent, HoprStartProtocol, LOCAL_PIX_SUITE,
        SESSION_APPLICATION_TAG, SsaQuota, pix_params_to_quota,
    },
    utils,
    utils::{SurbNotificationMode, insert_into_next_slot},
};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ACTIVE_SESSIONS: hopr_api::types::telemetry::SimpleGauge = hopr_api::types::telemetry::SimpleGauge::new(
        "hopr_session_num_active_sessions",
        "Number of currently active HOPR sessions"
    ).unwrap();
    static ref METRIC_NUM_ESTABLISHED_SESSIONS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
        "hopr_session_established_sessions_count",
        "Number of sessions that were successfully established as an Exit node"
    ).unwrap();
    static ref METRIC_NUM_INITIATED_SESSIONS: hopr_api::types::telemetry::SimpleCounter = hopr_api::types::telemetry::SimpleCounter::new(
        "hopr_session_initiated_sessions_count",
        "Number of sessions that were successfully initiated as an Entry node"
    ).unwrap();
    static ref METRIC_RECEIVED_SESSION_ERRS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_received_error_count",
        "Number of HOPR session errors received from an Exit node",
        &["kind"]
    ).unwrap();
    static ref METRIC_DISPATCHED_MSGS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_dispatched_messages",
        "Number dispatched HOPR session messages and their classification",
        &["kind"]
    ).unwrap();
    static ref METRIC_SENT_SESSION_ERRS: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_sent_error_count",
        "Number of HOPR session errors sent to an Entry node",
        &["kind"]
    ).unwrap();
}

/// One outgoing data packet on its way to the wire.
type EgressItem = (DestinationRouting, ApplicationDataOut);

/// The result of asking the PIX egress gate for permission to send one packet.
///
/// `Left` is the answer the gate could give synchronously — a permit, or a refusal because the
/// Session is being torn down. `Right` is the parked case, and the only one that allocates.
type EgressPermit = futures::future::Either<
    std::future::Ready<Result<EgressItem, std::io::Error>>,
    futures::future::BoxFuture<'static, Result<EgressItem, std::io::Error>>,
>;

/// Passes one outgoing data packet through a Session's PIX egress gate, if it has one.
///
/// Returns a future rather than awaiting so this can sit in a `Sink::with` without the combinator
/// having to box anything in the common case. The gate answers synchronously whenever service is
/// available — a relaxed load and a compare-exchange — and only the exhausted-budget path allocates,
/// which is a path that is about to block regardless.
///
/// A Session that negotiated no PIX passes through with no gate at all, so an un-supervised Session
/// pays one `Option` check per packet and nothing else.
fn acquire_egress_permit(
    gate: Option<Arc<ServiceGate>>,
    routing: DestinationRouting,
    data: ApplicationDataOut,
) -> EgressPermit {
    let Some(gate) = gate else {
        return futures::future::Either::Left(std::future::ready(Ok((routing, data))));
    };

    match gate.try_acquire_sync() {
        Ok(true) => futures::future::Either::Left(std::future::ready(Ok((routing, data)))),
        Err(_) => futures::future::Either::Left(std::future::ready(Err(std::io::Error::other(
            crate::supervision::GateClosed,
        )))),
        // Budget exhausted: park until the supervisor releases service, reports progress, or gives
        // up on the Session entirely.
        Ok(false) => futures::future::Either::Right(Box::pin(async move {
            gate.acquire()
                .await
                .map(|_| (routing, data))
                .map_err(std::io::Error::other)
        })),
    }
}

#[tracing::instrument(level = "debug", skip(session_data))]
fn close_session(session_id: SessionId, session_data: SessionSlot, reason: ClosureReason) {
    debug!("closing session");

    #[cfg(feature = "telemetry")]
    {
        set_session_state(&session_id, SessionLifecycleState::Closed);
        remove_session_metrics_state(&session_id, session_data.pix_egress_gate.get().is_some());
    }

    if reason != ClosureReason::EmptyRead {
        // Closing the data sender will also cause it to close from the read side
        debug!("data tx channel closed on session");
    }

    // Poison the egress gate before aborting anything. A writer parked on an exhausted predeposit
    // budget is waiting for a supervisor that is about to stop existing, so it has to be failed
    // rather than left pending — aborting the tasks first would remove the only thing that could
    // still have woken it.
    if let Some(gate) = session_data.pix_egress_gate.get() {
        gate.poison();
    }

    // Terminate any additional tasks spawned by the Session. This is also what releases the PIX
    // reconstructor state: the action driver holds a commitment guard per live cycle, and aborting
    // it drops them.
    session_data.abort_handles.lock().abort_all();

    // And return the memory those cycles were admitted against, now rather than whenever the last
    // clone of this slot happens to be dropped. Idempotent, so the slot's own `Drop` — including
    // the cache's deferred one — is free to run afterwards.
    if let Some(reservation) = session_data.cycle_budget.as_ref() {
        reservation.release();
    }

    #[cfg(all(feature = "telemetry", not(test)))]
    METRIC_ACTIVE_SESSIONS.decrement(1.0);
}

fn initiation_timeout_max_one_way(base: Duration, hops: usize) -> Duration {
    base * (hops as u32)
}

/// Conservative lower bound on how many coefficient commitments fit into one `SsaCommit` message.
///
/// Mirrors the sizing in `SsaClientCommitmentMessage::new_multiple`: payload minus the fixed
/// prefix (`ssa_index` + `coefficient_index` + `num_polys` + the Start header) and minus a generous
/// allowance for the CBOR-encoded session id, divided by the per-entry cost
/// (`PolynomialIndex` + one serialized group element). Using a *lower* bound here means the derived
/// message count is an over-estimate, which is the safe direction for sizing a queue.
///
/// The commitment proof of knowledge is subtracted as well. Every message carries it, since every
/// message is a constant-term message.
const MIN_COMMITMENTS_PER_SSA_COMMIT_MSG: usize = {
    const FIXED_PREFIX: usize = 12;
    // A `SessionId` is a 10-byte pseudonym; 64 bytes is a large allowance for its CBOR framing.
    const CBOR_SESSION_ID_ALLOWANCE: usize = 64;
    const PER_ENTRY: usize = size_of::<hopr_protocol_pix::PolynomialIndex>() + size_of::<HoprPixGroupElement>();

    let usable = ApplicationData::PAYLOAD_SIZE
        .saturating_sub(FIXED_PREFIX + CBOR_SESSION_ID_ALLOWANCE + HOPR_PIX_COMMITMENT_PROOF_SIZE);
    let per_msg = usable / PER_ENTRY;
    if per_msg == 0 { 1 } else { per_msg }
};

/// Slack added on top of the PIX commitment burst to cover non-PIX Start protocol traffic
/// (session initiations, establishments, errors, keep-alives).
const START_PROTOCOL_CHANNEL_RESERVE: usize = 10;

/// Ceiling on the per-session term of [`start_protocol_channel_capacity`].
///
/// A session contributes to this channel only while its Start exchange is in flight — a handful of
/// messages between initiation and establishment — after which it is silent apart from PIX, which
/// the commitment term already covers. So the queue depth tracks concurrent *handshakes*, not the
/// total number of sessions the node may manage, and `maximum_managed_sessions` (validated up to
/// 100 000) is the wrong quantity to size a pre-allocated ring from.
const MAX_CONCURRENT_START_EXCHANGES: usize = 10_000;

/// Capacity of the Start protocol ingress channel.
///
/// This channel is fed by [`SessionManager::dispatch_message`] with `try_send`, and an overflow
/// **drops** the message. For most Start messages that is recoverable, but a dropped `SsaCommit`
/// is not: there is no NACK or retransmission, so the corresponding coefficient cell stays empty
/// forever, the commitment never completes, every subsequent share fails to verify, and the cycle
/// dies on the supervisor's `max_deposit_wait` deadline.
///
/// PIX changed this channel's load from roughly one message per session to the *entire* commitment
/// set of an SSA cycle, chunked into packet-sized messages, plus a reserve for ordinary Start
/// traffic. Batching multiplies that: an Exit that asks for
/// [`ssas_per_request`](crate::SupervisorConfig::ssas_per_request) SSAs at once gets that many
/// cycles' commitment sets back-to-back, all landing here, so the per-cycle term is scaled by it.
///
/// The per-cycle burst is bounded by two independent limits, and the capacity takes the smaller:
///
/// * `quota_range.end() / PAYLOAD_SIZE` is `polys × (threshold + surplus)`, an over-estimate by that whole second
///   factor, since a cycle carries one constant term per polynomial and nothing else. The quota alone does not reveal
///   how the product splits, so the over-estimate cannot be undone from it.
/// * [`MAX_POLYS_PER_SSA`] is the number of polynomials [`SessionManager::check_pix_params`] will accept, whatever the
///   quota says. It therefore bounds the commitments a cycle can ever deliver.
///
/// Clamping to the second matters because this capacity is *reserved*, not merely enforced:
/// `crossfire`'s array flavour pre-allocates every slot when the channel is built. An
/// operator-settable `quota_range` feeding an unclamped derivation is an allocation with no upper
/// bound — at `quota_range.end() = 1e13` it asks for 77 GB. Over-provisioning is still the safe
/// direction within the clamp, and the surviving margin is large: the default dimensions burst
/// ≈ 320 messages against a capacity term of ≈ 648.
///
/// The batch factor is bounded by [`MAX_SSA_BATCH_SIZE`] for the same allocation reason, and is
/// clamped here rather than being taken on trust from the config, so that callers which build a
/// `SessionManagerConfig` without going through [`SessionManager::new`] cannot inflate it.
///
/// The session term is clamped for the same reason. `maximum_managed_sessions` validates up to
/// 100 000, and one slot holds a `(HoprPseudonym, HoprStartProtocol)` sized by the enum's largest
/// variant, so an operator raising the session limit would silently buy a multi-megabyte startup
/// allocation. Ordinary Start traffic is one message per session *in flight*, not one per session
/// the node will ever manage, so [`MAX_CONCURRENT_START_EXCHANGES`] is the honest bound.
fn start_protocol_channel_capacity(cfg: &SessionManagerConfig) -> usize {
    let max_commitments =
        (*cfg.pix_config.quota_range.end() / HoprPacket::PAYLOAD_SIZE as u64).min(MAX_POLYS_PER_SSA as u64);
    let max_commit_msgs = max_commitments.div_ceil(MIN_COMMITMENTS_PER_SSA_COMMIT_MSG as u64);
    let ssas_per_request = cfg.pix_config.supervision.ssas_per_request.clamp(1, MAX_SSA_BATCH_SIZE) as u64;

    // `usize::try_from` cannot fail on 64-bit targets; saturate rather than panic on 32-bit ones.
    usize::try_from(max_commit_msgs.saturating_mul(ssas_per_request))
        .unwrap_or(usize::MAX)
        .saturating_add(cfg.maximum_sessions.min(MAX_CONCURRENT_START_EXCHANGES))
        .saturating_add(START_PROTOCOL_CHANNEL_RESERVE)
}

/// Minimum time the SURB buffer must endure if no SURBs are being produced.
pub const MIN_SURB_BUFFER_DURATION: Duration = Duration::from_secs(1);
/// Minimum time between SURB buffer notifications to the Entry.
pub const MIN_SURB_BUFFER_NOTIFICATION_PERIOD: Duration = Duration::from_secs(1);

/// The first challenge value used in Start protocol to initiate a session.
pub(crate) const MIN_CHALLENGE: StartChallenge = 1;

/// Maximum time to wait for counterparty to receive the target number of SURBs.
const SESSION_READINESS_TIMEOUT: Duration = Duration::from_secs(10);

/// How long the Entry holds a *near-miss* `SsaRequest` while the Exit's returned data catches up to
/// the successor boundary.
///
/// Absorbs mixer reordering between the returned packets and the `SsaRequest` they earned — the two
/// travel the same mixed path, so the request can arrive ahead of the last few packets that
/// justified it. Nothing longer: a sustained shortfall never reaches this wait, because the gate only
/// enters it for a request already within one emission window of the boundary.
///
/// Comfortably inside the Exit's `max_ssa_delivery_time` (20 s by default, batch-scaled), which is
/// what closes the Session as `CommitmentTimeout` if no `SsaCommit` follows. That budget also has to
/// cover generating the commitments and shipping the burst, so this takes a small slice of it.
const SSA_SUCCESSOR_SERVICE_WAIT: Duration = Duration::from_secs(2);

/// How often [`SSA_SUCCESSOR_SERVICE_WAIT`] re-reads the returned-packet count.
///
/// Polled rather than notified: the alternative is waking a waiter from the Session receive path,
/// which would put a branch on every inbound packet to serve a case that arises once per SSA cycle.
const SSA_SUCCESSOR_SERVICE_POLL: Duration = Duration::from_millis(250);

/// Minimum timeout until an unfinished frame is discarded.
const MIN_FRAME_TIMEOUT: Duration = Duration::from_millis(10);

/// Hard ceiling on both SSA batch-size knobs, whatever the configuration says.
///
/// Deliberately far below the wire limit (`StartProtocol::MAX_SSAS_PER_REQUEST`, 27), which only
/// bounds what can be *decoded*. The real cost is paid on both sides of the exchange, and neither is
/// small at the profiled dimensions:
///
/// * Entry: every entry in the batch is a full `new_ssa_commitment` (hundreds of thousands of EC commitments), its own
///   burst of thousands of `SsaCommit` packets, and its own `ReadyToDeposit` — i.e. its own on-chain deposit.
/// * Exit: every entry is a live reconstructor cycle, held until that cycle recovers — worst case ≈41 MiB at the
///   deployed dimensions (`hopr_protocol_pix::peak_cycle_bytes`), so ≈820 MiB per Session at this ceiling, before the
///   pipelining factor. That is what a Session reserves against [`IncomingSessionPixConfig::max_live_cycle_bytes`], so
///   raising the batch size directly divides how many PIX Sessions the node will accept.
///
/// It also bounds the supervisor's deadline scaling: a batch multiplies both
/// [`max_ssa_delivery_time`](crate::SupervisorConfig::max_ssa_delivery_time) and
/// [`max_deposit_wait`](crate::SupervisorConfig::max_deposit_wait), and that product is what decides
/// how long a Session may be served unincentivized.
///
/// Both [`SupervisorConfig::ssas_per_request`](crate::SupervisorConfig::ssas_per_request) and
/// [`SessionManagerConfig::max_ssas_per_ssa_request`] are clamped to `1..=Self` where they are read,
/// so a programmatically built config that never calls `validate()` cannot exceed it.
pub const MAX_SSA_BATCH_SIZE: usize = 20;

/// SSA batches whose reconstructor state can be live on the Exit at the same moment.
///
/// Two, and structurally so. Only the *last* cycle of a batch may ask for a successor, and it asks
/// once (`supervision`'s "Rolling SSAs"), so a batch has at most one successor outstanding. The
/// predecessor cannot add a third: full recovery calls `remove_cycle` immediately, and what survives
/// until `RetireSsa` is the supervisor's own record and a tombstone, not cycle state.
///
/// Used as the pipelining factor when a Session reserves against
/// [`IncomingSessionPixConfig::max_live_cycle_bytes`].
pub const MAX_OVERLAPPING_BATCHES: u64 = 2;

/// What a PIX Session at these dimensions costs the node's live-cycle budget.
///
/// The worst case one cycle can hold, times every cycle that can be live at once. Both factors come
/// from configuration rather than observation, because the charge is made before the Session exists.
/// `ssas_per_request` is clamped for the reason [`SessionManager::new`] clamps it — a
/// programmatically built config must not be able to understate the reservation and then overrun it.
pub fn cycle_budget_for(params: &PixParams, ssas_per_request: usize) -> u64 {
    hopr_protocol_pix::peak_cycle_bytes::<HoprPixSpec>(params)
        .saturating_mul(ssas_per_request.clamp(1, MAX_SSA_BATCH_SIZE) as u64)
        .saturating_mul(MAX_OVERLAPPING_BATCHES)
}

/// The largest reservation any offer inside a `quota_range` ending at `quota_bytes` can produce.
///
/// The quota fixes `polys × (threshold + surplus)` but not the split, and the two terms of
/// [`hopr_protocol_pix::peak_cycle_bytes`] pull opposite ways — the share buffers want a high
/// threshold, the per-polynomial overhead wants many polynomials — so the maximum is found by
/// walking the thresholds rather than by a closed form. 254 iterations, once, at config load.
///
/// `surplus = 0` throughout, which is what makes each candidate the worst of its threshold: the
/// surplus is priced into the quota but holds no reconstructor state, so any surplus at all buys
/// fewer polynomials for the same quota.
///
/// Used to reject a [`IncomingSessionPixConfig::max_live_cycle_bytes`] that could never admit even
/// one Session at the dimensions its own `quota_range` advertises.
pub fn max_cycle_budget_for_quota(quota_bytes: u64, ssas_per_request: usize) -> u64 {
    let quota_shares = quota_bytes / HoprPacket::PAYLOAD_SIZE as u64;

    (hopr_protocol_pix::MIN_POLY_THRESHOLD..=hopr_protocol_pix::MAX_POLY_THRESHOLD)
        .filter_map(|threshold| {
            let polys = u16::try_from((quota_shares / threshold as u64).min(MAX_POLYS_PER_SSA as u64)).ok()?;
            PixParams::try_new(polys, threshold, 0, LOCAL_PIX_SUITE).ok()
        })
        .map(|params| cycle_budget_for(&params, ssas_per_request))
        .max()
        .unwrap_or_default()
}

/// Default for [`SessionManagerConfig::max_ssas_per_ssa_request`] — how many SSA commitments an Entry
/// accepts in a single [`SsaServerCommitmentMessage`].
///
/// Pipelining needs at most one cycle in flight ahead of the active one, so 2 leaves room for an Exit
/// batching at the default without turning one inbound packet into an unbounded amount of Entry work
/// and on-chain deposits.
pub const DEFAULT_MAX_SSAS_PER_SSA_REQUEST: usize = 2;

/// Default for [`SupervisorConfig::ssas_per_request`](crate::SupervisorConfig::ssas_per_request) —
/// how many SSAs an Exit asks for in a single [`SsaServerCommitmentMessage`].
///
/// One, so that the default configuration produces exactly the unbatched exchange: same wire bytes,
/// same supervisor deadlines, same Start protocol channel capacity.
pub const DEFAULT_SSAS_PER_SSA_REQUEST: usize = 1;

/// Timeout when sending Start protocol messages to the sink
const EXTERNAL_SEND_TIMEOUT: Duration = Duration::from_millis(200);

// Needs to use an UnboundedSender instead of oneshot
// because Moka cache requires the value to be Clone, which oneshot Sender is not.
// It also cannot be enclosed in an Arc, since calling `send` consumes the oneshot Sender.
// The Session initiation cache is only present on the Entry (client) side.
type SessionInitiationCache = moka::sync::Cache<
    StartChallenge,
    crossfire::MTx<crossfire::mpsc::One<Result<StartEstablished<SessionId>, StartErrorType<SessionId>>>>,
>;

/// Handles to streams and tasks spawned by the Session.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum SessionHandles {
    /// Handle to the stream that facilitates ingress of data from the HOPR network into the Session.
    Ingress,
    /// Handle to the process that sends keep-alive messages to the Session recipient (Exit).
    KeepAlive,
    /// Handle to the process that monitors and balances SURBs.
    Balancer,
    /// Handle to the task that executes the PIX supervisor's actions.
    ///
    /// One per Session rather than one per cycle: the supervisor multiplexes every SSA it tracks
    /// onto a single action stream.
    PixActionDriver,
    /// Handle to the process that awaits the PIX deposit for one SSA.
    ///
    /// Carries the inner `SsaIndex` value so that each cycle's observer is independent — pipelining
    /// must not cancel an earlier cycle's.
    PixDepositObserver(u32),
}

impl std::fmt::Display for SessionHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ingress => write!(f, "Ingress"),
            Self::KeepAlive => write!(f, "KeepAlive"),
            Self::Balancer => write!(f, "Balancer"),
            Self::PixActionDriver => write!(f, "PixActionDriver"),
            Self::PixDepositObserver(idx) => write!(f, "PixDepositObserver({idx})"),
        }
    }
}

/// The PIX dimensions this Session negotiated, on the Exit side.
///
/// Once carried the SSA index and a fault counter as well. Both moved to the supervisor: it decides
/// *when* a cycle is requested, so it is also what allocates the index, and it is what enforces the
/// fault limit. Leaving a second copy of either here would have meant two authorities for one fact.
#[derive(Debug)]
struct SessionSsaState {
    /// The dimensions this Session negotiated, as they went on the wire.
    params: PixParams,
    /// Highest SSA index this Session has had the generator commit to, `0` before the first batch.
    ///
    /// A deliberate second copy of a fact the generator already holds, and the only one of the three
    /// that came back. The generator's own watermark lives in a cache with an idle retention, so
    /// "absent" there conflates *never committed* with *state discarded* — and those must not be
    /// treated alike, because the first is the opening batch every Session begins with and the second
    /// is a Session whose Entry can no longer serve the cycles it already committed to. This copy
    /// lives exactly as long as the Session does, so it can tell them apart. See the successor gate in
    /// [`SessionManager::handle_ssa_request`].
    ///
    /// Entry-side only; an Exit never commits and leaves it at zero. [`SsaIndex`] is a `NonZero<u32>`,
    /// which is what makes `0` an unambiguous "none" rather than a sentinel that has to be defended.
    committed_ssa_watermark: std::sync::atomic::AtomicU32,
    /// [`SessionSlot::returned_packets`] as it stood when this Session first committed.
    ///
    /// The successor gate counts service *since* that instant, not since the Session opened. Before
    /// the first commitment the generator holds no polynomials, so the SURBs going out carry no
    /// shares — and an Exit may legitimately be served up to `max_predeposit_packets` of them before
    /// any deposit exists. Crediting that prefix would let the Exit bank unpaid service against the
    /// first cycle it *is* paid for. Everything after this point rides a share.
    returned_at_first_commit: std::sync::atomic::AtomicU64,
}

impl SessionSsaState {
    pub fn new(params: PixParams) -> Self {
        Self {
            params,
            committed_ssa_watermark: std::sync::atomic::AtomicU32::new(0),
            returned_at_first_commit: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Highest SSA index committed for this Session, `None` before the first batch.
    #[inline]
    pub fn committed_watermark(&self) -> Option<SsaIndex> {
        SsaIndex::new(self.committed_ssa_watermark.load(Ordering::Relaxed))
    }

    /// Raises the committed-index watermark to `index` if it is higher, taking the returned-packet
    /// baseline on the first commitment.
    ///
    /// `fetch_max` rather than a store: the gate serialises requests per pseudonym, but a Session's
    /// slot is shared and monotonicity here must not depend on that lock staying where it is. Its
    /// *previous* value is also what makes the baseline race-free without leaning on that lock —
    /// exactly one caller can observe a prior watermark of zero, so exactly one stores.
    #[inline]
    pub fn note_committed(&self, index: SsaIndex, returned_packets: u64) {
        if self.committed_ssa_watermark.fetch_max(index.get(), Ordering::Relaxed) == 0 {
            self.returned_at_first_commit.store(returned_packets, Ordering::Relaxed);
        }
    }

    /// Exit → Entry packets received since this Session's first commitment.
    ///
    /// Saturating rather than wrapping: `returned_packets` is monotonic and the baseline is a past
    /// value of it, so an underflow is impossible — and if one ever became possible, reporting zero
    /// service closes the successor gate rather than opening it.
    #[inline]
    pub fn served_since_first_commit(&self, returned_packets: u64) -> u64 {
        returned_packets.saturating_sub(self.returned_at_first_commit.load(Ordering::Relaxed))
    }

    /// Data quota one SSA cycle of these dimensions covers.
    ///
    /// The whole cycle, surplus included, as per [`pix_params_to_quota`]: every share a cycle emits
    /// rides one Exit → Entry packet, so all of them are priced.
    #[inline]
    pub const fn quota_per_ssa(&self) -> SsaQuota {
        pix_params_to_quota(&self.params)
    }
}

#[derive(Clone)]
pub(crate) struct SessionSlot {
    // Sender does not need to be in Arc, because the receiver part is always
    // wrapped inside DropAbortable wrapper, with abort handle added to `abort_handles`.
    session_tx: crossfire::MTx<crossfire::mpsc::Array<ApplicationDataIn>>,
    routing_opts: DestinationRouting,
    // Additional tasks spawned by the Session.
    abort_handles: Arc<parking_lot::Mutex<AbortableList<SessionHandles>>>,
    // Allows reconfiguring of the SURB balancer on-the-fly
    // Set on both Entry and Exit sides.
    surb_mgmt: Arc<BalancerStateValues>,
    // SURB flow updates happening outside of Session protocol
    // (e.g., due to Start protocol messages).
    surb_estimator: AtomicSurbFlowEstimator,
    // Contains currently active SSA for this Session and its quota
    current_ssa_state: Arc<OnceLock<SessionSsaState>>,
    /// Handle to this Session's PIX supervisor, on the Exit side of a PIX-enabled Session.
    ///
    /// Populated before the [`HoprSession`] is constructed, so the egress adapters below observe a
    /// gate rather than racing its installation. Empty on the Entry side and on non-PIX Sessions —
    /// the Exit is authoritative for the lifecycle, so the Entry runs no supervisor.
    pix_supervisor: Arc<OnceLock<SessionPixSupervisorHandle>>,
    /// The egress gate every outgoing data packet of a supervised Session must pass.
    ///
    /// Held separately from `pix_supervisor` because the egress path touches it per packet and has
    /// no use for the rest of the handle.
    pix_egress_gate: Arc<OnceLock<Arc<ServiceGate>>>,
    /// Exit → Entry Session packets received on this Session, ever.
    ///
    /// Entry-side only; stays zero on the Exit, which is the side that *sends* them. Each such
    /// packet consumed one return SURB, and a SURB carries at most one PIX share which the Exit can
    /// only decrypt by using it — so this counts shares the Exit has unlocked, measured without
    /// asking it. That is what makes it usable as a deposit gate: see the successor gate in
    /// [`SessionManager::handle_ssa_request`].
    ///
    /// Deliberately *not* [`surb_estimator`](Self::surb_estimator)`.consumed`, which increments on
    /// the same event on this side today. That field is documented as an *estimate* feeding the PID
    /// balancer, and it is only wired when `surb_management` is enabled; this one decides whether
    /// money is spent and must be neither. Carrying the increment twice is the cheaper mistake: a
    /// receive path that forgets this counter makes the gate stricter, never laxer.
    returned_packets: Arc<std::sync::atomic::AtomicU64>,
    /// This Session's share of the node's live reconstructor-cycle budget.
    ///
    /// Exit-side and PIX-only; `None` everywhere else, since nothing else holds cycle state. Behind
    /// an `Arc` so the budget is returned when the last clone of this slot goes: a slot is cloned
    /// into the Session cache and into its [`SessionSlotGuard`], so no single one of them can be
    /// made responsible for the release.
    ///
    /// [`close_session`] returns it explicitly, which is what makes the release simultaneous with
    /// the closure; the `Drop` behind the `Arc` is the backstop for anything that bypasses that
    /// function. Both are safe to run because the release is idempotent.
    cycle_budget: Option<Arc<CycleBudgetReservation>>,
}

/// One Session's reservation against the node's live reconstructor-cycle budget.
///
/// Charged when a PIX Session is accepted and returned when its Session closes. Deliberately a
/// projection rather than a measurement: the alternative is to weigh the reconstructor's actual
/// state and refuse a share once it is too large, which loses the cycle of whichever Session
/// happened to arrive last rather than of the one that inflated it. A reservation taken up front can
/// only ever refuse a Session that does not exist yet.
#[derive(Debug)]
pub(crate) struct CycleBudgetReservation {
    bytes: u64,
    outstanding: Arc<std::sync::atomic::AtomicU64>,
    /// Whether [`release`](Self::release) has already run, so it can be called from both the close
    /// path and `Drop` without the budget being returned twice.
    released: std::sync::atomic::AtomicBool,
}

impl CycleBudgetReservation {
    /// Returns the reservation to the node's budget. Idempotent.
    ///
    /// Called explicitly by [`close_session`], and by `Drop` as the backstop.
    ///
    /// Both, and not just `Drop`, because the slot lives in a `moka` cache: `remove` hands the value
    /// back but drops the cache's own clone during a later maintenance pass, so a purely
    /// refcount-driven release would return the budget at an unpredictable time — and a node whose
    /// Sessions all closed could still refuse the next one. The explicit call makes the release
    /// simultaneous with the closure that caused it; the flag is what keeps the deferred drop from
    /// crediting it a second time.
    fn release(&self) {
        if self.released.swap(true, Ordering::Relaxed) {
            return;
        }
        // Saturating, so a release the flag somehow failed to suppress could not wrap the counter
        // into a budget that admits everything.
        let outstanding = self
            .outstanding
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |held| {
                Some(held.saturating_sub(self.bytes))
            })
            .unwrap_or_default()
            .saturating_sub(self.bytes);
        trace!(
            released = self.bytes,
            outstanding, "released live-cycle budget reservation"
        );
    }
}

impl Drop for CycleBudgetReservation {
    fn drop(&mut self) {
        self.release();
    }
}

/// RAII guard that rolls back a freshly inserted [`SessionSlot`] unless the
/// session setup is explicitly [committed](SessionSlotGuard::commit).
///
/// Establishing a session involves several fallible steps *after* the slot has
/// been inserted into the Session cache (constructing the [`HoprSession`],
/// notifying about the new session, sending the establishment message, ...).
/// If any of these steps fails, the already inserted slot would otherwise linger
/// in the cache until idle eviction, blocking the pseudonym (and counting towards
/// `maximum_sessions`) in the meantime.
///
/// Dropping this guard without committing removes the slot and tears down the
/// partially initialized session. Since Moka's removal is asynchronous and Rust
/// has no asynchronous `Drop`, the cleanup is performed on a spawned task.
struct SessionSlotGuard<'a> {
    sessions: &'a moka::sync::Cache<SessionId, SessionSlot>,
    active_sessions: Arc<std::sync::atomic::AtomicUsize>,
    session_id: SessionId,
    committed: bool,
}

impl<'a> SessionSlotGuard<'a> {
    fn new(
        sessions: &'a moka::sync::Cache<SessionId, SessionSlot>,
        session_id: SessionId,
        active_sessions: Arc<std::sync::atomic::AtomicUsize>,
    ) -> Self {
        Self {
            sessions,
            active_sessions,
            session_id,
            committed: false,
        }
    }

    /// Marks the session as successfully established, preventing the slot from
    /// being rolled back when this guard is dropped.
    fn commit(&mut self) {
        self.committed = true;

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_ACTIVE_SESSIONS.increment(1.0);
    }
}

impl Drop for SessionSlotGuard<'_> {
    fn drop(&mut self) {
        if !self.committed {
            // The session setup failed after the slot was inserted: remove it so it does
            // not block the pseudonym until idle eviction.
            let session_id = self.session_id;
            warn!(%session_id, "rolling back partially established session slot after setup failure");
            if let Some(slot) = self.sessions.remove(&session_id) {
                self.active_sessions.fetch_sub(1, Ordering::Relaxed);
                close_session(session_id, slot, ClosureReason::Eviction);
            }
        }
    }
}

/// Indicates the result of processing a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchResult {
    /// Session or Start protocol message has been processed successfully.
    Processed,
    /// The message was not related to Start or Session protocol.
    Unrelated(ApplicationDataIn),
}

/// Configuration of the PIX protocol for incoming Sessions on Exit nodes.
#[derive(Clone, Debug, PartialEq, smart_default::SmartDefault, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IncomingSessionPixConfig {
    /// If set to true, incoming Session without the [`Capability::UsePIX`] will be rejected.
    ///
    /// Default `false`.
    #[default(false)]
    pub enforce_pix: bool,
    /// Acceptable range of data quota per one SSA in bytes.
    ///
    /// If an Entry sends PIX parameters for SSA reconstruction that are outside this quota range,
    /// the incoming Session will be rejected.
    ///
    /// The default is derived from the default PIX dimensions
    /// ([`crate::DEFAULT_PIX_POLYS_PER_SSA`] × ([`crate::DEFAULT_PIX_SHARES_PER_POLY`] +
    /// [`crate::DEFAULT_PIX_SURPLUS_SHARES`])) rather than hard-coded, so that an Entry running the
    /// default configuration is always accepted. The upper bound is exactly
    /// [`DEFAULT_PIX_SSA_QUOTA`]: the range expresses how much data this Exit is willing to serve
    /// per SSA cycle, and accepting more than our own nominal dimensions would raise both that
    /// exposure and the reconstructor memory held per Session. An Exit that wants to serve Entries
    /// configured with larger dimensions must widen this range explicitly.
    ///
    /// The quota it is compared against counts the surplus — `polys × (threshold + surplus) ×
    /// PAYLOAD_SIZE` — so this bounds the traffic actually served rather than the fraction of it the
    /// threshold accounts for. It used to bound only the latter, which understated the exposure by
    /// the surplus factor: 1.25× at the deployed dimensions.
    ///
    /// Defaults to `DEFAULT_PIX_SSA_QUOTA / 4 ..= DEFAULT_PIX_SSA_QUOTA`
    /// (≈ 162 MiB to ≈ 649 MiB, inclusive).
    #[default(_code = "DEFAULT_PIX_SSA_QUOTA / DEFAULT_PIX_QUOTA_RANGE_SPAN..=DEFAULT_PIX_SSA_QUOTA")]
    pub quota_range: std::ops::RangeInclusive<u64>,
    /// Ceiling on the live Exit-side reconstructor state this node will commit to, in bytes.
    ///
    /// **This, not [`SessionManagerConfig::maximum_sessions`], is what bounds reconstructor
    /// memory.** A PIX Session reserves `MAX_OVERLAPPING_BATCHES × ssas_per_request ×
    /// hopr_protocol_pix::peak_cycle_bytes(offered params)` when it is accepted and holds it until
    /// it closes; a Session that does not fit is refused with
    /// [`StartErrorReason::NoSlotsAvailable`] before any state is allocated for it. The reservation
    /// is computed from the parameters the *peer* offered, so a Session asking for smaller
    /// dimensions costs proportionally less of the budget.
    ///
    /// Enforced at admission rather than validated as a product of the configuration, for the reason
    /// `SsaReconstructorConfig::max_ack_buffer_bytes` gives: `maximum_sessions` validates to
    /// 100 000, and the resulting product is a number no node could hold, so validating it would
    /// only ever reject the shipping defaults. Counting what has actually been committed to is
    /// indifferent to how the ceiling was configured.
    ///
    /// Default is 3 GiB. Derived from the deployed operating point rather than picked: at the
    /// default dimensions a Session reserves ≈82 MiB, so this admits ≈37 concurrent PIX Sessions,
    /// comfortably covering the 10–30 clients per Exit the calibration profile models. The same
    /// defaults with `maximum_sessions = 100` and no budget imply ≈8 GiB.
    ///
    /// A ceiling on *reservations*, not an allocation: nothing is claimed up front, and because the
    /// reservation is denominated at the adversarial peak, a node serving conforming peers holds an
    /// order of magnitude less than it has reserved. The number exists to stop an Exit selling more
    /// service than its memory can hold, not to describe what it will typically use.
    #[default(3 * 1024 * 1024 * 1024)]
    pub max_live_cycle_bytes: u64,
    /// Deadlines, fault limits and service budget the Exit-side PIX supervisor enforces on a
    /// Session.
    ///
    /// These live together rather than spread across this struct because they are only meaningful
    /// as a set: [`crate::validate_pix_supervision`] checks them against each other and against the
    /// reconstructor's lifetimes, and the node's config validator runs that at load time.
    pub supervision: SupervisorConfig,
}

impl IncomingSessionPixConfig {
    /// The supervisor configuration for Sessions accepted under these settings.
    pub fn supervisor_config(&self) -> SupervisorConfig {
        self.supervision.clone()
    }
}

/// Configuration for the [`SessionManager`].
#[derive(Clone, Debug, PartialEq, smart_default::SmartDefault)]
pub struct SessionManagerConfig {
    /// The maximum chunk of data that can be written to the Session's input buffer.
    ///
    /// Default is 1500.
    #[default(1500)]
    pub frame_mtu: usize,

    /// The maximum time for an incomplete frame to stay in the Session's output buffer.
    ///
    /// Must exceed the worst-case SURB replenishment latency on the return path.
    /// The SURB balancer replenishes via KeepAlive every ~2 s; the retry loop in
    /// the routing resolver sleeps 5 ms per attempt and can block for the full
    /// replenishment cycle.  Setting this below the KeepAlive period causes the
    /// sequencer to discard frames that arrive after a SURB-starved peer finally
    /// gets new SURBs and sends its echo.
    ///
    /// Default is 3 s.
    #[default(Duration::from_secs(3))]
    pub max_frame_timeout: Duration,

    /// Maximum number of segments to buffer in the downstream transport of a Session's socket.
    /// If 0 is given, the transport is unbuffered.
    ///
    /// Default is 0.
    #[default(0)]
    pub max_buffered_segments: usize,

    /// Abandon the frame due next once this many later frames are waiting behind it, rather than
    /// holding them for the whole of [`Self::max_frame_timeout`].
    ///
    /// The two answer different questions. `max_frame_timeout` is how long a *missing* frame is
    /// waited for, and is set here to exceed the ~2 s SURB KeepAlive so a starved peer's late echo
    /// is not discarded. This bounds how much *already received* data is held hostage during that
    /// wait -- a cost paid once per gap, which compounds as later frames queue up.
    ///
    /// Without it, a session that cannot retransmit waits the full timeout for a frame that will
    /// never arrive. Measured on a 5-node cluster after killing a return relayer: 98.5 % of bytes
    /// returned over the wire, 0.60 % reached the application, and the application-side
    /// inter-arrival median sat exactly on the 3 s timeout.
    ///
    /// The right value tracks reordering depth -- throughput x latency spread / frame size -- so
    /// it is deployment-specific. Tune with `SessionConfig::max_frames_behind_gap`; too low converts
    /// ordinary reordering into loss, too high leaves the stall in place.
    ///
    /// Default is 256, which is roughly 3-4x the reordering depth of the cluster it was measured
    /// on (~0.5 MB/s over paths spread across 10-260 ms, 1500 B frames, so ~70-85 frames in
    /// flight out of order). The margin is not cosmetic: at 64 -- about one reordering depth --
    /// a *healthy* baseline dropped from 100 % to 92.2 % arrival, because ordinary reordering was
    /// being read as loss. At 256 the same baseline returned 100 % while a return relayer killed
    /// mid-session still recovered 97.1 %, against 0.60 % with the bound absent.
    ///
    /// Default is 256.
    #[default(Some(256))]
    pub max_frames_behind_gap: Option<usize>,

    /// The base timeout for initiation of Session initiation.
    ///
    /// The actual timeout is adjusted according to the number of hops for that Session:
    /// `t = initiation_time_out_base * (num_forward_hops + num_return_hops + 2)`
    ///
    /// Default is 500 milliseconds.
    #[default(Duration::from_millis(500))]
    pub initiation_timeout_base: Duration,

    /// Timeout for Session to be closed due to inactivity.
    ///
    /// Default is 180 seconds.
    #[default(Duration::from_secs(180))]
    pub idle_timeout: Duration,

    /// Minimum interval at which an establishing Session's cache slot is refreshed
    /// ("touched") to keep [`idle_timeout`](Self::idle_timeout) from evicting it while
    /// SURBs pre-load.
    ///
    /// The touch period is [`idle_timeout`](Self::idle_timeout)` / 2`, floored to this
    /// value so it never drops below a sane minimum for very short idle timeouts.
    ///
    /// Default is 100 milliseconds.
    #[default(Duration::from_millis(100))]
    pub min_session_touch_period: Duration,

    /// The sampling interval for SURB balancer.
    /// It will make SURB control decisions regularly at this interval.
    ///
    /// Default is 100 milliseconds.
    #[default(Duration::from_millis(100))]
    pub balancer_sampling_interval: Duration,

    /// Initial packets per second egress rate on an incoming Session.
    ///
    /// This only applies to incoming Sessions without the [`Capability::NoRateControl`] flag set.
    ///
    /// Default is 10 packets/second.
    #[default(10)]
    pub initial_return_session_egress_rate: usize,

    /// Minimum period of time for which a SURB buffer at the Exit must
    /// endure if no SURBs are being received.
    ///
    /// In other words, it is the minimum period of time an Exit must withstand when
    /// no SURBs are received from the Entry at all. To do so, the egress traffic
    /// will be shaped accordingly to meet this requirement.
    ///
    /// This only applies to incoming Sessions without the [`Capability::NoRateControl`] flag set.
    ///
    /// Default is 5 seconds, minimum is 1 second.
    #[default(Duration::from_secs(5))]
    pub minimum_surb_buffer_duration: Duration,

    /// Indicates the maximum number of SURBs in the SURB buffer to be requested when creating a new Session.
    ///
    /// This value is theoretically capped by the size of the global transport SURB ring buffer,
    /// so values greater than that do not make sense. This value should be ideally set equal
    /// to the size of the global transport SURB RB.
    ///
    /// Default is 10 000 SURBs.
    #[default(10_000)]
    pub maximum_surb_buffer_size: usize,

    /// If set, the Session recipient (Exit) will notify the Session initiator (Entry) about
    /// its SURB balance for the Session using keep-alive packets periodically.
    ///
    /// Keep in mind that each notification also costs 1 SURB, so the notification period should
    /// not be too frequent.
    ///
    /// Default is None (no notification sent to the client), minimum is 1 second.
    #[default(None)]
    pub surb_balance_notify_period: Option<Duration>,

    /// If set, the Session initiator (Entry) will notify the Session recipient (Exit) about
    /// the local SURB balancer target using keep-alive packets from the SURB balancer.
    ///
    /// This is useful when the client plans to change the SURB balancer target dynamically.
    ///
    /// Default is true.
    #[default(true)]
    pub surb_target_notify: bool,

    /// Maximum number of concurrent sessions allowed.
    ///
    /// Default is 10_000.
    #[default(10_000)]
    pub maximum_sessions: usize,

    /// How many packets can be buffered if the [`HoprSession`] input socket is not fast enough.
    ///
    /// Controls the capacity of the internal `crossfire` channel used for each session slot.
    ///
    /// Default is 10 000.
    #[default(10000)]
    pub session_forward_capacity: usize,

    /// Configuration of the PIX protocol for the Exit nodes.
    pub pix_config: IncomingSessionPixConfig,

    /// Maximum number of SSA commitments this node, acting as an Entry, will accept in a single
    /// [`SsaServerCommitmentMessage`].
    ///
    /// This is the Entry's protection against a misbehaving Exit, not a preference: every accepted
    /// entry costs a full `new_ssa_commitment`, thousands of outbound `SsaCommit` packets and its own
    /// on-chain deposit, so without a cap one inbound packet amplifies into minutes of CPU, a large
    /// packet burst, and as many simultaneous deposits as the wire format admits. An over-cap request
    /// is rejected in full, before any commitment is generated or any `ReadyToDeposit` is emitted, and
    /// the Exit is told with a `SessionError` — see `refuse_ssa_request`.
    ///
    /// It must be at least the `ssas_per_request` of every Exit this node connects to — see
    /// [`SupervisorConfig::ssas_per_request`](crate::SupervisorConfig::ssas_per_request) for why a mismatch
    /// loses every Session.
    ///
    /// Clamped to `1..=`[`MAX_SSA_BATCH_SIZE`] in [`SessionManager::new`].
    ///
    /// Defaults to [`DEFAULT_MAX_SSAS_PER_SSA_REQUEST`] (2).
    #[default(DEFAULT_MAX_SSAS_PER_SSA_REQUEST)]
    pub max_ssas_per_ssa_request: usize,
}

// Type-erased sink used by the `SessionManager` to notify about newly incoming sessions.
// The errors produced by the underlying sink are remapped into `SessionManagerError`.
type BoxSink<T> = Pin<Box<dyn Sink<T, Error = SessionManagerError> + Send>>;

type SessionNotifiers = (
    Arc<hopr_utils::runtime::prelude::Mutex<BoxSink<IncomingSession>>>,
    crossfire::MTx<crossfire::mpsc::Array<(SessionId, ClosureReason)>>,
);

// Sink for processing Start protocol messages.
// Must be within Arc to be shared across SessionManager clones.
// The inner OnceLock is set once in `start()` and read in `dispatch_message`.
type StartProtocolMsgSink = Arc<OnceLock<crossfire::MTx<crossfire::mpsc::Array<(HoprPseudonym, HoprStartProtocol)>>>>;

/// PIX protocol toolbox to enable [`SessionManager`] to use PIX protocol.
#[derive(Clone)]
pub struct PixToolbox {
    share_generator: Arc<SsaShareGenerator<HoprPixSpec>>,
    share_processor: Arc<SsaReconstructor<HoprPixSpec>>,
    pix_events: crossfire::MTx<crossfire::mpsc::Array<HoprSessionOutPixEvent>>,
}

impl PixToolbox {
    pub fn new(
        share_generator: Arc<SsaShareGenerator<HoprPixSpec>>,
        share_processor: Arc<SsaReconstructor<HoprPixSpec>>,
    ) -> (Self, impl futures::Stream<Item = HoprSessionOutPixEvent>) {
        let (pix_events, pix_events_rx) = crossfire::mpsc::bounded_blocking_async::<HoprSessionOutPixEvent>(1024);
        (
            Self {
                share_generator,
                share_processor,
                pix_events,
            },
            pix_events_rx.into_stream(),
        )
    }
}

/// Manages lifecycles of Sessions.
///
/// Once the manager is [started](SessionManager::start), the [`SessionManager::dispatch_message`]
/// should be called for each [`ApplicationData`] received by the node.
/// This way, the `SessionManager` takes care of proper Start sub-protocol message processing
/// and correct dispatch of Session-related packets to individual existing Sessions.
///
/// Secondly, the manager can initiate new outgoing sessions via [`SessionManager::new_session`],
/// probe sessions using [`SessionManager::ping_session`]
/// and list them via [`SessionManager::active_sessions`].
///
/// Since the `SessionManager` operates over the HOPR protocol,
/// the message transport `S` is required.
/// Such transport must also be `Clone`, since it will be cloned into all the created [`HoprSession`] objects.
///
/// ## SURB balancing
///
/// The manager also can take care of automatic [SURB balancing](SurbBalancerConfig) per Session.
///
/// With each packet sent from the session initiator over to the receiving party, zero to 2 SURBs might be delivered.
/// When the receiving party wants to send reply packets back, it must consume 1 SURB per packet. This
/// means that if the difference between the SURBs delivered and SURBs consumed is negative, the receiving party
/// might soon run out of SURBs. If SURBs run out, the reply packets will be dropped, causing likely quality of
/// service degradation.
///
/// In an attempt to counter this effect, there are two co-existing automated modes of SURB balancing:
/// *local SURB balancing* and *remote SURB balancing*.
///
/// ### Local SURB balancing
///
/// Local SURB balancing is performed on the sessions that were initiated by another party (and are
/// therefore incoming to us).
/// The local SURB balancing mechanism continuously evaluates the rate of SURB consumption and retrieval,
/// and if SURBs are running out, the packet egress shaping takes effect. This by itself does not
/// avoid the depletion of SURBs but slows it down in the hope that the initiating party can deliver
/// more SURBs over time. This might happen either organically by sending effective payloads that
/// allow non-zero number of SURBs in the packet, or non-organically by delivering KeepAlive messages
/// via *remote SURB balancing*.
///
/// The egress shaping is done automatically, unless the Session initiator sets the [`Capability::NoRateControl`]
/// flag during Session initiation.
///
/// ### Remote SURB balancing
///
/// Remote SURB balancing is performed by the Session initiator. The SURB balancer estimates the number of SURBs
/// delivered to the other party, and also the number of SURBs consumed by seeing the amount of traffic received
/// in replies.
/// When enabled, a desired target level of SURBs at the Session counterparty is set. According to measured
/// inflow and outflow of SURBs to/from the counterparty, the production of non-organic SURBs is started
/// via keep-alive messages (sent to counterparty) and is controlled to maintain that target level.
///
/// In other words, the Session initiator tries to compensate for the usage of SURBs by the counterparty by
/// sending new ones via the keep-alive messages.
///
/// This mechanism is configurable via the `surb_management` field in [`SessionClientConfig`].
///
/// ### Possible scenarios
///
/// There are 4 different scenarios of local vs. remote SURB balancing configuration, but
/// an equilibrium (= matching the SURB production and consumption) is most likely to be reached
/// only when both are configured (the ideal case below):
///
/// #### 1. Ideal local and remote SURB balancing
///
/// 1. The Session recipient (Exit) set the `initial_return_session_egress_rate`, `max_surb_buffer_duration` and
///    `maximum_surb_buffer_size` values in the [`SessionManagerConfig`].
/// 2. The Session initiator (Entry) sets the [`target_surb_buffer_size`](SurbBalancerConfig) which matches the
///    [`maximum_surb_buffer_size`](SessionManagerConfig) of the counterparty.
/// 3. The Session initiator (Entry) does *NOT* set the [`Capability::NoRateControl`] capability flag when opening
///    Session.
/// 4. The Session initiator (Entry) sets [`max_surbs_per_sec`](SurbBalancerConfig) slightly higher than the
///    `maximum_surb_buffer_size / max_surb_buffer_duration` value configured at the counterparty.
///
/// In this situation, the maximum Session egress from Exit to the Entry is given by the
/// `maximum_surb_buffer_size / max_surb_buffer_duration` ratio. If there is enough bandwidth,
/// the (remote) SURB balancer sending SURBs to the Exit will stabilize roughly at this rate of SURBs/sec,
/// and the whole system will be in equilibrium during the Session's lifetime (under ideal network conditions).
///
/// #### 2. Remote SURB balancing only
///
/// 1. The Session initiator (Entry) *DOES* set the [`Capability::NoRateControl`] capability flag when opening Session.
/// 2. The Session initiator (Entry) sets `max_surbs_per_sec` and `target_surb_buffer_size` values in
///    [`SurbBalancerConfig`]
///
/// In this one-sided situation, the Entry node floods the Exit node with SURBs,
/// only based on its estimated consumption of SURBs at the Exit. The Exit's egress is not
/// rate-limited at all. If the Exit runs out of SURBs at any point in time, it will simply drop egress packets.
///
/// This configuration could potentially only lead to an equilibrium
/// when the `SurbBalancer` at the Entry can react fast enough to Exit's demand.
///
/// #### 3. Local SURB balancing only
///
/// 1. The Session recipient (Exit) set the `initial_return_session_egress_rate`, `max_surb_buffer_duration` and
///    `maximum_surb_buffer_size` values in the [`SessionManagerConfig`].
/// 2. The Session initiator (Entry) does *NOT* set the [`Capability::NoRateControl`] capability flag when opening
///    Session.
/// 3. The Session initiator (Entry) does *NOT* set the [`SurbBalancerConfig`] at all when opening Session.
///
/// In this one-sided situation, the Entry node does not provide any additional SURBs at all (except the
/// ones that are naturally carried by the egress packets which have space to hold SURBs). It relies
/// only on the Session egress limiting of the Exit node.
/// The Exit will limit the egress roughly to the rate of natural SURB occurrence in the ingress.
///
/// This configuration could potentially only lead to an equilibrium when uploading non-full packets
/// (ones that can carry at least a single SURB), and the Exit's egress is limiting itself to such a rate.
/// If Exit's egress reaches low values due to SURB scarcity, the upper layer protocols over Session might break.
///
/// #### 4. No SURB balancing on each side
///
/// 1. The Session initiator (Entry) *DOES* set the [`Capability::NoRateControl`] capability flag when opening Session.
/// 2. The Session initiator (Entry) does *NOT* set the [`SurbBalancerConfig`] at all when opening Session.
///
/// In this situation, no additional SURBs are being produced by the Entry and no Session egress rate-limiting
/// takes place at the Exit.
///
/// This configuration can only lead to an equilibrium when Entry sends non-full packets (ones that carry
/// at least a single SURB) and the Exit is consuming the SURBs (Session egress) at a slower or equal rate.
/// Such configuration is very fragile, as any disturbances in the SURB flow might lead to a packet drop
/// at the Exit's egress.
///
/// ### SURB decay
///
/// In a hypothetical scenario of a non-zero packet loss, the Session initiator (Entry) might send a
/// certain number of SURBs to the Session recipient (Exit), but only a portion of it is actually delivered.
/// The Entry has no way of knowing that and assumes that everything has been delivered.
/// A similar problem happens when the Exit uses SURBs to construct return packets, but only a portion
/// of those packets is actually delivered to the Entry. At this point, the Entry also subtracts
/// fewer SURBs from its SURB estimate at the Exit.
///
/// In both situations, the Entry thinks there are more SURBs available at the Exit than there really are.
///
/// To compensate for a potential packet loss, the Entry's estimation of Exit's SURB buffer is regularly
/// diminished by a percentage of the `target_surb_buffer_size`, even if no incoming traffic from the
/// Exit is detected.
///
/// This behavior can be controlled via the `surb_decay` field of [`SurbBalancerConfig`].
///
/// ### SURB balance and target notification
///
/// The Session recipient (Exit) can notify the Session initiator (Entry) periodically about its estimated
/// number of SURBs for the Session. This can help the Entry to adjust its approximation of that level so
/// that its Local SURB balancer can better intervene.
/// This can be set using the `surb_balance_notify_period` field of [`SessionManagerConfig`] for the Exit.
///
/// Likewise, the Entry can inform the Exit about its desired SURB buffer target so that the Exit
/// can better accommodate its Remote SURB balancing.
/// This can be set using the `surb_target_notify` field of the [`SessionManagerConfig`] of each new Session.
///
/// Both mechanisms leverage the Keep Alive message to report the respective values.
///
/// ## PIX (Protocol for Incentivization of eXits) Protocol Flow
///
/// When a Session is opened with the [`Capability::UsePIX`] flag, the following protocol
/// runs between the Entry (initiator) and Exit (recipient) to provide on-chain payment
/// guarantees for the data relayed through the Session.
///
/// ### 1. PIX Parameter Negotiation (Session Initiation)
///
/// During [`SessionManager::new_session`], the Entry encodes its PIX SSA (Session Stealth
/// Address) parameters — a [`PixParams`] quadruple of `polys_per_ssa`, `shares_per_poly`,
/// `surplus_shares` and the curve `suite` — into the upper 32 bits of the
/// `StartSession.additional_data` field, via [`PixParams::into_additional_data`]. The first two
/// describe how many polynomials and shares each SSA will use; the third is how many extra shares
/// per polynomial the Entry emits to absorb losses. Those three define the data quota per SSA, which
/// is `polys × (threshold + surplus) × PAYLOAD_SIZE` — the surplus is priced in rather than free,
/// since a cycle emits it whether or not any share is lost (see `pix_params_to_quota`).
///
/// The fourth is not a dimension and does not enter the quota: it names the elliptic curve the
/// Entry's build instantiates PIX over, which fixes the width of every curve-sized field later in
/// the handshake. It is fixed at build time on both sides and is therefore not negotiated — the Exit
/// refuses anything but its own, below.
///
/// What is announced is built from the installed [`SsaShareGenerator`]'s
/// [`SsaGeneratorConfig`](hopr_protocol_pix::SsaGeneratorConfig), never from the caller: the
/// generator is what produces the shares that go on the wire, so advertising anything else would
/// let the Session proceed while emitting shares the Exit cannot reconstruct. The caller's
/// `pix_ssa_quota` is an assertion about this node's own PIX configuration, and a disagreement is
/// refused so a caller whose belief is stale fails loudly rather than silently getting a different
/// per-SSA quota — and so differently sized deposits — than it sized for. That check runs before
/// the initiation challenge slot is reserved, so repeated misconfigurations cannot exhaust
/// challenge slots.
///
/// On the Exit side, `check_pix_params` validates these parameters against:
/// - The protocol ranges, which [`PixParams::try_from_additional_data`] enforces as it unpacks.
/// - The configured [`IncomingSessionPixConfig::quota_range`] (by default derived from the default PIX dimensions: ≈162
///   MiB–649 MiB per SSA).
/// - Optionally, [`IncomingSessionPixConfig::enforce_pix`] rejects Sessions that do not offer PIX.
/// - The Exit only checks the *product* `polys × (threshold + surplus)`, not the individual values, so the Entry can
///   split it to suit its computing power. The computation is easily parallelizable in the number of polynomials, but
///   not in threshold. The surplus is inside that product, so redundancy is bought rather than taken: a cycle emits
///   `threshold + surplus` shares per polynomial come what may, and the deposit covers all of them.
///
/// If parameters are rejected, a [`StartErrorReason::UnacceptablePixParams`] error is returned.
///
/// ### 2. Exit SSA Request (`SsaRequest` → Entry)
///
/// Once the PIX parameters are accepted, the Exit spawns a `SessionPixSupervisor`, whose opening
/// `RequestSsa` action has `send_ssa_request` create a new SSA commitment via the server-side
/// [`SsaReconstructor`]. This produces an *Exit commitment* (a group element) that is sent back to
/// the Entry as a [`SsaServerCommitmentMessage`].
///
/// One action, and therefore one message, can carry a whole batch:
/// [`SupervisorConfig::ssas_per_request`](crate::SupervisorConfig::ssas_per_request) SSAs at
/// contiguous indices, sharing the single `params` field, since every SSA in a Session uses the same
/// negotiated dimensions. The Entry caps what it will accept at
/// [`SessionManagerConfig::max_ssas_per_ssa_request`], and rejects an over-cap request in full while
/// replying with an `UnacceptablePixParams` [`StartErrorType`], so the Exit does not have to infer the
/// refusal from a deadline. The default is a batch of one, which is byte-for-byte the unbatched
/// exchange.
///
/// From here the supervisor owns the cycle's deadlines: `max_ssa_delivery_time` for the Entry's
/// commitment, then `max_deposit_wait` for the funds, then the recovery deadlines. Missing any of
/// them closes the Session with `ClosureReason::PixFailure`. The first two are multiplied by the batch
/// size, because a batch asks the Entry for that many commitment sets and that many deposits before
/// any of them can fairly be called late.
///
/// ### 3. Entry SSA Commitment (`SsaCommit` → Exit)
///
/// Upon receiving the [`SsaServerCommitmentMessage`], the Entry's `handle_ssa_request`
/// generates a *client commitment* using the shared [`SsaShareGenerator`] (which is also
/// used by the packet pipeline to embed PIX shares into return-path SURBs). The client
/// commitment is combined with the Exit commitment to derive the on-chain deposit address
/// via [`HoprPixSpec::group_to_deposit_address`].
///
/// The Entry then sends one or more [`SsaClientCommitmentMessage`]s back to the Exit and
/// emits a [`HoprSessionOutPixEvent::ReadyToDeposit`] to the upper layer, signaling that
/// funds can be deposited at the computed address.
///
/// ### 4. Deposit Awaiting (Exit Side)
///
/// The Exit receives the client commitment messages in `handle_ssa_commit`, inserts the
/// coefficient commitments into the [`SsaReconstructor`], and extracts the deposit address.
/// It emits [`HoprSessionOutPixEvent::DepositNeeded`] to the upper layer with the
/// [`AgreedSsaQuota`] and a channel to confirm the deposit.
///
/// A verifiable commitment is what starts the deposit clock, so the supervisor is told
/// (`CommitmentVerified`) before anything can be reported against it. A `PixDepositObserver` task
/// then forwards every confirmation arriving on that channel as `DepositConfirmed`: top-ups
/// accumulate, and the supervisor is what decides when enough has landed. The observer carries no
/// timeout of its own — `max_deposit_wait` is the deadline, and a second authority racing the
/// first is exactly what the supervisor exists to prevent. If the upper layer drops the sender
/// without ever confirming, the observer reports that (`DepositObserverClosed`) rather than
/// letting the deadline run out on a deposit that is never coming.
///
/// Once the deposit suffices, the SSA moves to *recovering* and its recovery deadlines start. The
/// first sufficient deposit on a Session also releases the egress gate, which is what lets the
/// Session be served at all; later cycles inherit that release.
///
/// ### 5. SSA Collection, Recovery and Pipelining
///
/// As the Entry sends return-path SURBs during the Session, each SURB can carry a PIX
/// share generated from the client's polynomial set. The Exit's [`SsaReconstructor`]
/// collects these shares and reports how far the cycle has got via
/// [`HoprSessionInPixEvent::RecoveryProgress`]; `dispatch_pix_event` forwards those snapshots to
/// the supervisor, which uses them both to reset `max_recovery_idle` and to keep the gate serving.
///
/// When the reconstructor reaches the *early recovery threshold* (≈85%), an
/// [`HoprSessionInPixEvent::SsaAlmostRecovered`] event fires and the supervisor answers with a
/// `RequestSsa` action for the next index — pipelining the costly commitment exchange with the
/// tail of the share collection for the current SSA. If the current cycle is still awaiting its
/// own commitment or deposit, the request is deferred until that clears, so a Session never has
/// two unfunded cycles outstanding.
///
/// Once fully recovered, [`HoprSessionInPixEvent::SsaRecovered`] fires, allowing the Exit to
/// unlock and redeem the deposited funds. The supervisor tombstones the cycle and emits
/// `RetireSsa`, which drops that cycle's [`SsaCommitmentGuard`] and aborts its deposit observer;
/// the next SSA is requested here if `SsaAlmostRecovered` has not already done so. Observers are
/// keyed by SSA index, so retiring one cycle never cancels a pipelined successor's.
///
/// ### 6. Unverifiable Shares
///
/// Shares are not checked individually. Once a polynomial has collected `threshold` of them,
/// the reconstructor interpolates its constant term and compares it against the commitment; if
/// they disagree, at least one of those shares did not come from the committed polynomial and an
/// [`HoprSessionInPixEvent::UnverifiableShares`] event fires, carrying the reconstructor's running
/// total for the SSA rather than a delta.
/// [`SupervisorConfig::max_unverifiable_shares_per_ssa`] is 0 by default, so the first such report
/// closes the Session — the cycle is already unrecoverable, and closing immediately caps what a
/// malicious Entry is served at `threshold` packets.
/// [`SupervisorConfig::max_unverifiable_shares_per_session`] bounds the same across cycles, so a
/// steady trickle of one failure per SSA still escalates; at the default per-SSA limit of zero it
/// never gets the chance to fire.
///
/// ### Configuring PIX at the Exit
///
/// The Exit configures PIX via [`IncomingSessionPixConfig`] within [`SessionManagerConfig`].
///
/// The [`PixToolbox`] (holding the [`SsaShareGenerator`] and [`SsaReconstructor`]) must
/// be provided via [`SessionManager::start`] for PIX to function.
pub struct SessionManager<S> {
    // Keeps track of Session initiations requests on the Client side.
    session_initiations: SessionInitiationCache,
    session_notifiers: Arc<OnceLock<SessionNotifiers>>,
    start_protocol_tx: StartProtocolMsgSink,
    /// Authoritative session count for admission control.
    /// Incremented atomically inside `allocate_session_slot` before the cache insertion,
    /// and decremented at every removal path (explicit close, eviction, guard rollback).
    active_sessions: Arc<std::sync::atomic::AtomicUsize>,
    sessions: moka::sync::Cache<SessionId, SessionSlot>,
    msg_sender: Arc<OnceLock<S>>,
    pix_toolbox: Arc<OnceLock<PixToolbox>>,
    cfg: SessionManagerConfig,
    /// Per-SessionId waiters notified when a new session slot is allocated. Lets message
    /// handlers that arrive before the slot insertion completes (e.g. SsaRequest vs
    /// SessionEstablished) await the slot once instead of busy-looping with sleeps.
    /// Keyed by SessionId so that only waiters for the relevant session are woken.
    slot_allocated: Arc<Mutex<HashMap<SessionId, Vec<oneshot::Sender<()>>>>>,
    /// Serialises `handle_ssa_request` per pseudonym.
    ///
    /// Start messages are processed under `for_each_concurrent`, so without this several
    /// `SsaRequest`s for one pseudonym run at once — and each reads the successor gate's admission
    /// state before any of them has advanced it. Every racer passes, and the Entry commits to (and
    /// funds) as many batches as the Exit cared to send, which is exactly what the gate exists to
    /// stop. The generator's own monotonic index is the backstop that keeps this from being a
    /// correctness hole, but it only rejects *equal or lower* indices: an Exit numbering its batches
    /// upwards races past it.
    ///
    /// An async lock rather than a blocking one because the guarded region awaits — commitment
    /// generation goes to a blocking pool, and publication awaits the transport.
    ///
    /// A `moka` cache rather than a map, for its idle eviction: a `HashMap` keyed by pseudonym would
    /// otherwise retain one entry per Session for the life of the process.
    ssa_request_locks: moka::future::Cache<HoprPseudonym, Arc<futures::lock::Mutex<()>>>,
    /// Live reconstructor-cycle bytes this node has committed to, summed over every PIX Session.
    ///
    /// The counterpart of `active_sessions`, and the reason that one is not sufficient: a Session is
    /// admitted on a slot count, but what it costs the reconstructor is set by the dimensions its
    /// peer offered. Charged in `handle_incoming_session_initiation` and returned by
    /// [`CycleBudgetReservation`]'s `Drop`, so no removal path has to remember to decrement it.
    live_cycle_bytes: Arc<std::sync::atomic::AtomicU64>,
}

impl<S> Clone for SessionManager<S> {
    fn clone(&self) -> Self {
        Self {
            session_initiations: self.session_initiations.clone(),
            session_notifiers: self.session_notifiers.clone(),
            start_protocol_tx: self.start_protocol_tx.clone(),
            active_sessions: self.active_sessions.clone(),
            live_cycle_bytes: self.live_cycle_bytes.clone(),
            sessions: self.sessions.clone(),
            cfg: self.cfg.clone(),
            msg_sender: self.msg_sender.clone(),
            pix_toolbox: self.pix_toolbox.clone(),
            slot_allocated: Arc::clone(&self.slot_allocated),
            ssa_request_locks: self.ssa_request_locks.clone(),
        }
    }
}

fn session_config(cfg: &SessionManagerConfig, capabilities: Capabilities) -> HoprSessionConfig {
    session_config_with(cfg, capabilities, None)
}

/// As [`session_config`], with the initiating session's own head-of-line bound.
///
/// `None` inherits the node default; `Some(0)` disables the bound for this session; `Some(n)` sets
/// it. The zero-disables convention matches the node setting, so the per-session
/// and node-wide knobs cannot mean opposite things by the same value.
///
/// Sessions accepted from a peer pass `None`: the bound governs how *we* reassemble what arrives,
/// so it is ours to choose, not the initiator's to impose on us.
fn session_config_with(
    cfg: &SessionManagerConfig,
    capabilities: Capabilities,
    max_frames_behind_gap: Option<usize>,
) -> HoprSessionConfig {
    // Only a session that cannot recover the missing frame should abandon it early. With
    // retransmission the gap is a request away, so waiting is productive and cutting it short
    // would discard data that was on its way back; without it the wait is for something that is
    // never coming, and every frame behind the gap is held for nothing.
    let can_retransmit =
        capabilities.contains(Capability::RetransmissionAck) || capabilities.contains(Capability::RetransmissionNack);

    // The session's own value when it stated one, the node's otherwise. `Some(0)` disables the
    // bound at either level -- a threshold of zero would abandon every gap before a single frame
    // arrived behind it, which nobody wants and which the sequencer would silently clamp to one.
    let bound = match max_frames_behind_gap.or(cfg.max_frames_behind_gap) {
        Some(0) | None => None,
        Some(n) => Some(n),
    };

    HoprSessionConfig {
        capabilities,
        frame_mtu: cfg.frame_mtu,
        frame_timeout: cfg.max_frame_timeout,
        max_buffered_segments: cfg.max_buffered_segments,
        max_frames_behind_gap: (!can_retransmit).then_some(bound).flatten(),
    }
}

#[cfg(feature = "telemetry")]
fn initialize_session_telemetry(
    session_id: SessionId,
    cfg: &SessionManagerConfig,
    capabilities: Capabilities,
    surb_estimator: Option<&AtomicSurbFlowEstimator>,
    surb_mgmt: Option<&Arc<BalancerStateValues>>,
) {
    initialize_session_metrics(session_id, session_config(cfg, capabilities));
    set_session_state(&session_id, SessionLifecycleState::Active);
    if let (Some(estimator), Some(mgmt)) = (surb_estimator, surb_mgmt) {
        set_session_balancer_data(&session_id, estimator.clone(), mgmt.clone());
    }
}

async fn send_via_msg_sender<S, D>(
    msg_sender: &mut S,
    routing: DestinationRouting,
    data: D,
    error_context: &'static str,
) -> errors::Result<()>
where
    S: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Unpin,
    S::Error: std::error::Error + Send + Sync + Clone + 'static,
    D: TryInto<ApplicationData>,
    D::Error: std::error::Error + Send + Sync + 'static,
{
    let app_data: ApplicationData = data.try_into().map_err(SessionManagerError::other)?;
    msg_sender
        .send((routing, ApplicationDataOut::with_no_packet_info(app_data)))
        .timeout(futures_time::time::Duration::from(EXTERNAL_SEND_TIMEOUT))
        .await
        .map_err(|_| {
            error!("timeout sending {error_context}");
            TransportSessionError::Timeout
        })?
        .map_err(|error| {
            error!(%error, "failed to send {error_context}");
            SessionManagerError::other(error)
        })?;
    Ok(())
}

impl<S> SessionManager<S>
where
    S: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
    S::Error: std::error::Error + Send + Sync + Clone + 'static,
{
    /// Creates a new instance given the [`config`](SessionManagerConfig).
    pub fn new(mut cfg: SessionManagerConfig) -> Self {
        let maximum_sessions = cfg.maximum_sessions;
        cfg.surb_balance_notify_period = cfg
            .surb_balance_notify_period
            .map(|p| p.max(MIN_SURB_BUFFER_NOTIFICATION_PERIOD));
        cfg.minimum_surb_buffer_duration = cfg.minimum_surb_buffer_duration.max(MIN_SURB_BUFFER_DURATION);

        // Ensure the Frame MTU is at least the size of the Session segment MTU payload
        cfg.frame_mtu = cfg.frame_mtu.max(SESSION_MTU);
        cfg.max_frame_timeout = cfg.max_frame_timeout.max(MIN_FRAME_TIMEOUT);

        // Both SSA batch knobs are range-validated in `HoprProtocolConfig`, but nothing in this crate
        // calls `validate()` — clamp here so a programmatically built config cannot ask for a batch of
        // zero (no request would ever be sent) or one large enough to blow past what
        // `MAX_SSA_BATCH_SIZE` exists to bound.
        cfg.max_ssas_per_ssa_request = cfg.max_ssas_per_ssa_request.clamp(1, MAX_SSA_BATCH_SIZE);
        cfg.pix_config.supervision.ssas_per_request =
            cfg.pix_config.supervision.ssas_per_request.clamp(1, MAX_SSA_BATCH_SIZE);

        // Every supervisor duration, for the same reason and with more riding on it. A duration the
        // monotonic clock cannot represent makes `Instant::checked_add` return `None`, and every phase
        // reads an absent deadline as *no deadline* — so an over-large value does not produce a distant
        // deadline, it silently disables the rule. `validate_pix_supervision` rejects such a config, but
        // only a node built from a config file goes through it.
        //
        // Representability is necessary and not sufficient, though: `max_recovery_idle` also has to
        // stay under the reconstructor's `unused_verifier_lifetime`, or the supervisor waits on a
        // cycle whose state was reclaimed hours earlier. Clamping only to the monotonic-clock cap left
        // exactly that — `Duration::MAX` became 24 h against a 30-minute default lifetime — so the
        // normalized config was representable and still invalid.
        //
        // The reconstructor is not installed until `start`, so the *default* configuration is what
        // this can normalize against. `start` re-checks against the one actually installed, which is
        // where a caller pairing a non-default reconstructor is caught.
        //
        // The two batch-scaled deadlines are clamped by the scaled value, since that is what is armed;
        // dividing by the batch size (already clamped to at least 1 above) is what keeps the product
        // under the cap.
        {
            use hopr_protocol_pix::SsaReconstructorConfig;

            let sup = &mut cfg.pix_config.supervision;
            let cap = crate::supervision::MAX_SUPERVISOR_DURATION;
            let per_cycle_cap = cap / sup.ssas_per_request as u32;
            sup.max_ssa_delivery_time = sup.max_ssa_delivery_time.min(per_cycle_cap);
            sup.max_deposit_wait = sup.max_deposit_wait.min(per_cycle_cap);
            sup.max_recovery_time = sup.max_recovery_time.min(cap);
            sup.tombstone_retention_window = sup.tombstone_retention_window.min(cap);

            // Strictly under the lifetime, which is what `validate_pix_supervision` requires; a
            // saturating subtraction keeps a pathologically short lifetime from wrapping. The lower
            // bound is not clamped: raising a too-small value would be inventing a policy rather than
            // bounding one, and it is what `validate_pix_supervision` reports.
            let idle_cap = SsaReconstructorConfig::default()
                .unused_verifier_lifetime
                .saturating_sub(Duration::from_secs(1))
                .min(cap);
            sup.max_recovery_idle = sup.max_recovery_idle.min(idle_cap);
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_ACTIVE_SESSIONS.set(0.0);

        let active_sessions: Arc<std::sync::atomic::AtomicUsize> = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let active_sessions_for_listener = active_sessions.clone();

        let msg_sender = Arc::new(OnceLock::new());
        let initiation_timeout =
            2 * initiation_timeout_max_one_way(cfg.initiation_timeout_base, RoutingOptions::MAX_INTERMEDIATE_HOPS);
        let pix_toolbox: Arc<OnceLock<PixToolbox>> = Arc::new(OnceLock::new());
        Self {
            msg_sender: msg_sender.clone(),
            session_initiations: moka::sync::Cache::builder()
                .max_capacity(maximum_sessions as u64)
                .time_to_live(initiation_timeout)
                .build(),
            sessions: moka::sync::Cache::builder()
                .max_capacity(maximum_sessions as u64)
                .time_to_idle(cfg.idle_timeout)
                .eviction_listener(
                    move |session_id: Arc<SessionId>, entry: SessionSlot, reason| match &reason {
                        moka::notification::RemovalCause::Expired | moka::notification::RemovalCause::Size => {
                            trace!(?session_id, ?reason, "session evicted from the cache");
                            // Reconstructor state is released by `close_session` aborting the PIX
                            // action driver: the driver owns an `SsaCommitmentGuard` per live cycle,
                            // and dropping its future drops them. That bounds the release to the
                            // cycles actually in flight, where enumerating every index this Session
                            // had ever used was unbounded in its lifetime.
                            active_sessions_for_listener.fetch_sub(1, Ordering::Relaxed);
                            close_session(*session_id.as_ref(), entry, ClosureReason::Eviction);
                        }
                        _ => {}
                    },
                )
                .build(),
            pix_toolbox,
            session_notifiers: Arc::new(OnceLock::new()),
            start_protocol_tx: Arc::new(OnceLock::new()),
            active_sessions,
            live_cycle_bytes: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            cfg,
            slot_allocated: Arc::new(Mutex::new(HashMap::new())),
            // Idle rather than live TTL, and generous: the entry must outlive the gap between two
            // successive `SsaRequest`s of a Session, which is a whole SSA cycle — ~72 min at the
            // deployed dimensions and the documented rate cap. Matches the generator's own
            // per-pseudonym cache, which is what the guarded region reads.
            ssa_request_locks: moka::future::Cache::builder()
                .time_to_idle(Duration::from_secs(1800))
                .build(),
        }
    }

    /// Starts the instance with the given `msg_sender` `Sink`
    /// and a channel `new_session_notifier` used to notify when a new incoming session is opened to us.
    ///
    /// Optionally, the PIX processor and event sink can be provided for handling PIX protocol.
    /// If not specified, the `SessionManager` will not handle PIX protocol.
    ///
    /// This method must be called prior to any calls to [`SessionManager::new_session`] or
    /// [`SessionManager::dispatch_message`].
    pub fn start<T>(
        &self,
        msg_sender: S,
        new_session_notifier: T,
        pix: Option<PixToolbox>,
    ) -> errors::Result<Vec<AbortHandle>>
    where
        T: futures::Sink<IncomingSession> + Send + 'static,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        // Every fallible check runs before the first `OnceLock::set` below.
        //
        // `start` publishes the manager's started state through those locks, and a `OnceLock` cannot
        // be un-set. Validating after `msg_sender` had been filled left a manager that reports
        // `is_started() == false` — no workers, no notifier — and yet refuses every retry with
        // `AlreadyStarted`: neither running nor recoverable, out of what is an ordinary configuration
        // error the caller could have corrected and retried on the same instance.
        if let Some(pix) = pix.as_ref() {
            // The authoritative cross-component check, and the first moment it can be made: the
            // supervisor's deadlines are meaningless without the reconstructor lifetimes they race
            // against, and the reconstructor arrives here rather than at construction.
            //
            // `SessionManager::new` normalizes against the *default* reconstructor config, which is
            // what makes the common programmatic case correct without an API break. A caller that
            // pairs a supervisor config with a non-default reconstructor is caught here instead — as
            // an error rather than a clamp, because at this point both halves were chosen
            // deliberately and silently overriding one of them would be the wrong answer.
            crate::supervision::validate_pix_supervision(
                &self.cfg.pix_config.supervision,
                pix.share_processor.config(),
            )?;
        }

        self.msg_sender
            .set(msg_sender)
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        if let Some(pix) = pix {
            self.pix_toolbox
                .set(pix)
                .map_err(|_| SessionManagerError::AlreadyStarted)?;
        }

        // Re-map the user-provided sink errors to `SessionManagerError` and erase the concrete
        //  type so that the `SessionManager` does not need to be generic over it. This also avoids
        // having to spawn a separate task to forward items between channels: senders simply lock
        // the sink and send directly.
        let new_session_notifier: BoxSink<IncomingSession> =
            Box::pin(new_session_notifier.sink_map_err(SessionManagerError::other));
        let new_session_notifier = Arc::new(hopr_utils::runtime::prelude::Mutex::new(new_session_notifier));

        let (session_close_tx, session_close_rx) =
            crossfire::mpsc::bounded_blocking_async(self.cfg.maximum_sessions + 10);
        self.session_notifiers
            .set((new_session_notifier, session_close_tx))
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        let (start_protocol_tx, start_protocol_rx) =
            crossfire::mpsc::bounded_blocking_async(start_protocol_channel_capacity(&self.cfg));
        let _ = self.start_protocol_tx.set(start_protocol_tx);

        let myself = self.clone();
        let closure_diag = hopr_utils::runtime::diagnostics::ConcurrentDiagnostics::new(
            "session_close_for_each_concurrent",
            module_path!(),
            file!(),
            line!(),
        );
        let ah_closure_notifications = hopr_utils::spawn_as_abortable_named!(
            "session_close_notifications",
            session_close_rx.into_stream().for_each_concurrent(
                self.cfg.maximum_sessions + 10,
                move |(session_id, closure_reason)| {
                    let myself = myself.clone();
                    let closure_diag = closure_diag.clone();
                    closure_diag.wrap(|| {
                        // These notifications come from the Sessions themselves once
                        // an empty read is encountered, which means the closure was done by the
                        // other party.
                        if let Some(session_data) = myself.sessions.remove(&session_id) {
                            // Reconstructor state is released by `close_session` aborting the PIX
                            // action driver, whose commitment guards retire on drop.
                            myself.active_sessions.fetch_sub(1, Ordering::Relaxed);
                            close_session(session_id, session_data, closure_reason);
                        } else {
                            // Do not treat this as an error
                            debug!(
                                ?session_id,
                                ?closure_reason,
                                "could not find session id to close, maybe the session is already closed"
                            );
                        }
                        futures::future::ready(())
                    })
                }
            )
        );

        // This is necessary to evict expired entries from the caches if
        // no session-related operations happen at all.
        // This ensures the dangling expired sessions are properly closed
        // and their closure is timely notified to the other party.
        let myself = self.clone();
        let ah_session_expiration = hopr_utils::spawn_as_abortable!(async move {
            let jitter = hopr_api::types::crypto_random::random_float_in_range(1.0..1.5);
            let timeout = 2 * initiation_timeout_max_one_way(
                myself.cfg.initiation_timeout_base,
                RoutingOptions::MAX_INTERMEDIATE_HOPS,
            )
            .min(myself.cfg.idle_timeout)
            .mul_f64(jitter)
                / 2;
            futures_time::stream::interval(timeout.into())
                .for_each(|_| async {
                    trace!("executing session cache evictions");
                    myself.sessions.run_pending_tasks();
                    myself.session_initiations.run_pending_tasks();
                })
                .await;
        });

        // Begin processing of Start protocol messages
        let myself = self.clone();
        let ah_start_protocol = hopr_utils::spawn_as_abortable_named!(
            "session_start_protocol_processor",
            start_protocol_rx.into_stream().for_each_concurrent(
                Some(self.cfg.maximum_sessions + 10),
                move |(pseudonym, protocol_msg)| {
                    let myself = myself.clone();
                    async move {
                        let result = match protocol_msg {
                            HoprStartProtocol::StartSession(session_req) => {
                                myself.handle_incoming_session_initiation(pseudonym, session_req).await
                            }
                            HoprStartProtocol::SessionEstablished(est) => myself.handle_session_established(est).await,
                            HoprStartProtocol::SessionError(error_type) => {
                                myself.handle_session_error(error_type).await
                            }
                            HoprStartProtocol::KeepAlive(msg) => myself.handle_keep_alive(msg).await,
                            HoprStartProtocol::SsaCommit(client_commit_msg) => {
                                myself.handle_ssa_commit(pseudonym, client_commit_msg).await
                            }
                            HoprStartProtocol::SsaRequest(server_commit_msg) => {
                                myself.handle_ssa_request(pseudonym, server_commit_msg).await
                            }
                        };

                        if let Err(error) = result {
                            error!(%error, "failed to process Start protocol message");
                        }
                    }
                }
            )
        );

        Ok(vec![ah_closure_notifications, ah_session_expiration, ah_start_protocol])
    }

    /// Check if [`start`](SessionManager::start) has been called and the instance is running.
    pub fn is_started(&self) -> bool {
        self.session_notifiers.get().is_some()
    }

    /// Atomically allocates a new [`SessionSlot`] for `session_id` and returns an RAII
    /// [`SessionSlotGuard`] for it.
    ///
    /// Establishing a session involves several fallible steps *after* the slot has been
    /// inserted. The returned guard rolls the slot back - tearing the partially
    /// established session down via [`close_session`] - unless it is
    /// [committed](SessionSlotGuard::commit).
    ///
    /// The active-sessions gauge is incremented here, atomically with the insertion and
    /// the guard creation, precisely so that it is always paired with the guard's
    /// rollback decrement (performed through [`close_session`]). This keeps the gauge
    /// accurate: it is never decremented for a slot that was not counted in the first
    /// place, and every counted slot is decremented exactly once when it leaves the cache.
    ///
    /// Returns `None` if a slot for `session_id` already exists; in that case nothing is
    /// inserted, the gauge is left untouched, and no guard is produced. The atomic `entry`
    /// API guarantees that only one concurrent caller can claim the slot for a given
    /// pseudonym (avoiding a TOCTOU race), which also rules out loopback sessions onto
    /// ourselves.
    ///
    /// Capacity is enforced by an atomic counter incremented *before* the cache insertion,
    /// making it impossible for two concurrent callers (with different session IDs) to both
    /// succeed when the cache is already at `maximum_sessions`.
    fn allocate_session_slot(&self, session_id: SessionId, slot: SessionSlot) -> Option<SessionSlotGuard<'_>> {
        // Try to claim a session slot before touching the cache. `fetch_update` atomically
        // increments only if the value is strictly below the limit, preventing two concurrent
        // callers from both succeeding when already at capacity.
        let counter = &self.active_sessions;
        #[allow(clippy::incompatible_msrv)]
        let did_reserve = counter
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                (n < self.cfg.maximum_sessions).then_some(n + 1)
            })
            .is_ok();

        if !did_reserve {
            return None;
        }

        let result =
            self.sessions
                .entry(session_id)
                .and_compute_with(|entry: Option<moka::Entry<SessionId, SessionSlot>>| {
                    if entry.is_none() {
                        moka::ops::compute::Op::Put(slot)
                    } else {
                        // Duplicate key — release the reservation so the counter stays accurate.
                        counter.fetch_sub(1, Ordering::Relaxed);
                        moka::ops::compute::Op::Nop
                    }
                });

        match result {
            moka::ops::compute::CompResult::Inserted(_) => {
                // Notify any waiting message handler (e.g. handle_ssa_request) that the slot
                // is now available. Drain and signal the senders registered for this SessionId.
                if let Some(waiters) = self
                    .slot_allocated
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(&session_id)
                {
                    for w in waiters {
                        let _ = w.send(());
                    }
                }
                // take_guard borrows self, so the guard stores the counter clone separately.
                Some(SessionSlotGuard::new(&self.sessions, session_id, counter.clone()))
            }
            _ => None,
        }
    }

    /// Initiates a new outgoing Session to `destination` with the given configuration.
    ///
    /// If the Session's counterparty does not respond within
    /// the [configured](SessionManagerConfig) period,
    /// this method returns [`TransportSessionError::Timeout`].
    ///
    /// It will also fail if the instance has not been [started](SessionManager::start).
    pub async fn new_session(
        &self,
        destination: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
    ) -> errors::Result<HoprSession> {
        self.sessions.run_pending_tasks();
        if self.cfg.maximum_sessions <= self.active_sessions.load(Ordering::Relaxed) {
            return Err(SessionManagerError::TooManySessions.into());
        }

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        let (tx_initiation_done, rx_initiation_done): (
            crossfire::MTx<crossfire::mpsc::One<_>>,
            crossfire::AsyncRx<crossfire::mpsc::One<_>>,
        ) = crossfire::mpsc::build(crossfire::mpsc::One::new());

        let current_ssa_state = Arc::new(OnceLock::new());

        let mut additional_data = 0_u64;

        // SURB balancer target announcement is encoded in the lower 32-bits of additional_data
        if !cfg.capabilities.contains(Capability::NoRateControl) {
            additional_data |= cfg
                .surb_management
                .map(|c| c.target_surb_buffer_size)
                .unwrap_or(
                    self.cfg.initial_return_session_egress_rate as u64
                        * self
                            .cfg
                            .minimum_surb_buffer_duration
                            .max(MIN_SURB_BUFFER_DURATION)
                            .as_secs(),
                )
                .min(u32::MAX as u64);
        }

        // PIX quota parameter announcement is encoded in the upper 32-bits of additional_data.
        // Run these validations BEFORE reserving the initiation challenge slot so that a
        // repeated invalid request cannot exhaust all challenge slots.
        if cfg.capabilities.contains(Capability::UsePIX) {
            // PIX requires at least 1 intermediate hop on the return path so that PIX
            // shares can be encrypted with the first relayer's ticket-challenge solution
            // (`HalfKey`) and delivered via return-path SURBs. With 0 intermediate hops
            // (a direct Exit→Entry SURB), there is no relayer to provide the challenge
            // solution, so shares are never embedded — the ongoing PIX share delivery
            // mechanism is dead and the Exit's quota is never replenished.
            if cfg.return_path_options.count_hops() == 0 {
                return Err(SessionManagerError::Other(anyhow!(
                    "UsePIX requires at least 1 intermediate hop on the return path, got 0"
                ))
                .into());
            }

            let requested = cfg
                .pix_ssa_quota
                .ok_or_else(|| SessionManagerError::Other(anyhow!("UsePIX requested without PIX SSA quota")))?;

            // Validate that PIX toolbox is available before advertising UsePIX
            let pix_toolbox = self
                .pix_toolbox
                .get()
                .ok_or_else(|| SessionManagerError::Other(anyhow!("UsePIX requested but no PIX toolbox installed")))?;

            // The installed generator is what actually produces the shares that go on the wire, so
            // it — not the caller — is the source of the announced parameters. The requested value
            // is an assertion about this node's own PIX configuration, checked so that a caller
            // whose belief is stale fails loudly instead of silently getting different dimensions
            // (and so a different per-SSA quota, and so differently sized deposits) than it sized
            // for. Advertising the caller's value instead would let the Session proceed while
            // producing shares the Exit cannot reconstruct.
            // The same source for the dimensions and for the curve suite: `HoprPixSpec` is what the
            // installed generator is instantiated over, so the announced suite cannot disagree with
            // the one that will actually produce the shares.
            let gen_cfg = pix_toolbox.share_generator.config();
            let params = PixParams::try_from_config::<HoprPixSpec>(gen_cfg)
                .map_err(|error| SessionManagerError::Other(anyhow!("invalid PIX dimensions: {error}")))?;
            if requested != params {
                return Err(SessionManagerError::Unacceptable(format!(
                    "requested PIX parameters {requested} do not match installed generator ({params})"
                ))
                .into());
            }

            let _ = current_ssa_state.set(SessionSsaState::new(params));
            additional_data = params.into_additional_data(additional_data as u32);
        }

        let (challenge, _) = insert_into_next_slot(
            &self.session_initiations,
            |ch| {
                if let Some(challenge) = ch {
                    ((challenge + 1) % hopr_api::types::crypto_random::MAX_RANDOM_INTEGER).max(MIN_CHALLENGE)
                } else {
                    hopr_api::types::crypto_random::random_integer(MIN_CHALLENGE, None)
                }
            },
            |_| tx_initiation_done,
            Some(self.cfg.maximum_sessions as u64),
        )
        .ok_or(SessionManagerError::NoChallengeSlots)?; // almost impossible with u64

        // Prepare the session initiation message in the Start protocol
        trace!(challenge, ?cfg, "initiating session with config");
        let start_session_msg = HoprStartProtocol::StartSession(StartInitiation {
            challenge,
            target,
            capabilities: HoprSessionCapabilities(cfg.capabilities),
            additional_data,
        });

        let pseudonym = cfg.pseudonym.unwrap_or(HoprPseudonym::random());
        let forward_routing = DestinationRouting::Forward {
            destination: Box::new(destination.into()),
            pseudonym: Some(pseudonym), // Session must use a fixed pseudonym already
            forward_options: cfg.forward_path_options.clone(),
            return_options: cfg.return_path_options.clone().into(),
        };

        // Send the Session initiation message
        info!(challenge, %pseudonym, %destination, "new session request");
        send_via_msg_sender(
            &mut msg_sender,
            forward_routing.clone(),
            start_session_msg,
            "session request message",
        )
        .await
        .map_err(|error| {
            self.session_initiations.remove(&challenge);
            TransportSessionError::packet_sending(error)
        })?;

        // The timeout is given by the number of hops requested
        let initiation_timeout: futures_time::time::Duration = initiation_timeout_max_one_way(
            self.cfg.initiation_timeout_base,
            cfg.forward_path_options.count_hops() + cfg.return_path_options.count_hops() + 2,
        )
        .into();

        // Await session establishment response from the Exit node or timeout

        trace!(challenge, "awaiting session establishment");
        match rx_initiation_done
            .into_stream()
            .try_next()
            .timeout(initiation_timeout)
            .await
        {
            Ok(Ok(Some(est))) => {
                // Session has been established, construct it
                let session_id = est.session_id;
                debug!(challenge = est.orig_challenge, ?session_id, "started a new session");

                let (session_tx, session_rx) =
                    crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(self.cfg.session_forward_capacity);
                let (session_rx, session_rx_ah) = hopr_utils::runtime::DropAbortable::new(session_rx.into_stream());

                let mut abort_handles = AbortableList::default();
                abort_handles.insert(SessionHandles::Ingress, session_rx_ah);

                let notifier = self
                    .session_notifiers
                    .get()
                    .map(|(_, notifier)| {
                        let notifier = notifier.clone();
                        Box::new(move |session_id: SessionId, reason: ClosureReason| {
                            let _ = notifier
                                .try_send((session_id, reason))
                                .inspect_err(|error| error!(%session_id, %error, "failed to notify session closure"));
                        })
                    })
                    .ok_or(SessionManagerError::NotStarted)?;

                // NOTE: the Exit node can have different `max_surb_buffer_size`
                // setting on the Session manager, so it does not make sense to cap it here
                // with our maximum value.
                if let Some(balancer_config) = cfg.surb_management {
                    let surb_estimator = AtomicSurbFlowEstimator::default();

                    // Sender responsible for keep-alive and Session data will be counting produced SURBs
                    let surb_estimator_clone = surb_estimator.clone();
                    let full_surb_scoring_sender =
                        msg_sender.with(move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                            let produced = data.estimate_surbs_with_msg() as u64;
                            // Count how many SURBs we sent with each packet
                            surb_estimator_clone
                                .produced
                                .fetch_add(produced, std::sync::atomic::Ordering::Relaxed);
                            #[cfg(feature = "telemetry")]
                            telemetry::record_session_surb_produced(&session_id, produced);
                            futures::future::ok::<_, S::Error>((routing, data))
                        });

                    // For standard Session data we first reduce the number of SURBs we want to produce,
                    // unless requested to always max them out
                    let max_out_organic_surbs = cfg.always_max_out_surbs;
                    let reduced_surb_scoring_sender = full_surb_scoring_sender.clone().with(
                        // NOTE: this is put in-front of the `full_surb_scoring_sender`,
                        // so that its estimate of SURBs gets automatically updated based on
                        // the `max_surbs_in_packets` set here.
                        move |(routing, mut data): (DestinationRouting, ApplicationDataOut)| {
                            if !max_out_organic_surbs {
                                // TODO: make this dynamic to honor the balancer target (#7439)
                                data.packet_info
                                    .get_or_insert_with(|| OutgoingPacketInfo {
                                        max_surbs_in_packet: 1,
                                        ..Default::default()
                                    })
                                    .max_surbs_in_packet = 1;
                            }
                            futures::future::ok::<_, S::Error>((routing, data))
                        },
                    );

                    let surb_mgmt = Arc::new(BalancerStateValues::from(balancer_config));
                    // The counterparty's store is the same bounded ring buffer as ours, so its
                    // capacity bounds what our `produced - consumed` estimate can legitimately
                    // claim it is holding.
                    surb_mgmt.set_counterparty_buffer_capacity(self.cfg.maximum_surb_buffer_size as u64);

                    // Spawn the SURB-bearing keep alive stream towards the Exit
                    let (ka_controller, ka_abort_handle) = utils::spawn_keep_alive_stream(
                        session_id,
                        full_surb_scoring_sender,
                        forward_routing.clone(),
                        if self.cfg.surb_target_notify {
                            SurbNotificationMode::Target
                        } else {
                            SurbNotificationMode::DoNotNotify
                        },
                        surb_mgmt.clone(),
                    );
                    abort_handles.insert(SessionHandles::KeepAlive, ka_abort_handle);

                    // Spawn the SURB balancer, which will decide on the initial SURB rate.
                    debug!(%session_id, ?balancer_config ,"spawning entry SURB balancer");
                    let balancer = SurbBalancer::new(
                        session_id,
                        // The setpoint and output limit is immediately reconfigured by the SurbBalancer
                        PidBalancerController::from_gains(PidControllerGains::from_env_or_default()),
                        surb_estimator.clone(),
                        // Currently, a keep-alive message can bear `HoprPacket::MAX_SURBS_IN_PACKET` SURBs,
                        // so the correction by this factor is applied.
                        SurbControllerWithCorrection(ka_controller, HoprPacket::MAX_SURBS_IN_PACKET as u32),
                        surb_mgmt.clone(),
                    );

                    let (level_stream, balancer_abort_handle) =
                        balancer.start_control_loop(self.cfg.balancer_sampling_interval);
                    abort_handles.insert(SessionHandles::Balancer, balancer_abort_handle);

                    let returned_packets = Arc::new(std::sync::atomic::AtomicU64::new(0));

                    // Insert the slot before the SURB readiness wait so any echo packets that
                    // arrive during pre-loading are accepted rather than dropped as unknown.
                    // Early return from the wait below drops slot_guard uncommitted, which removes
                    // the slot and calls close_session → abort_all() on the spawned tasks.
                    let mut slot_guard = self
                        .allocate_session_slot(
                            session_id,
                            SessionSlot {
                                session_tx,
                                routing_opts: forward_routing.clone(),
                                abort_handles: Arc::new(parking_lot::Mutex::new(abort_handles)),
                                surb_mgmt: surb_mgmt.clone(),
                                surb_estimator: surb_estimator.clone(),
                                current_ssa_state,
                                // Entry side: the Exit is authoritative for the PIX lifecycle, so
                                // there is no supervisor here and nothing gates egress.
                                pix_supervisor: Default::default(),
                                pix_egress_gate: Default::default(),
                                returned_packets: returned_packets.clone(),
                                // Nor does it hold reconstructor state: the live-cycle budget is
                                // charged by the side that reconstructs.
                                cycle_budget: None,
                            },
                        )
                        .ok_or_else(|| {
                            // Session already exists; it means it is most likely a loopback attempt
                            error!(%session_id, "session already exists - loopback attempt");
                            SessionManagerError::Loopback
                        })?;

                    // Prevent the slot from being evicted by time_to_idle while SURBs are
                    // pre-loading. Each `get()` call resets the idle timer; the task is
                    // aborted as soon as the readiness wait resolves.
                    let sessions_keepalive = self.sessions.clone();
                    let touch_period = (self.cfg.idle_timeout / 2).max(self.cfg.min_session_touch_period);
                    let slot_keepalive = hopr_utils::runtime::prelude::spawn(async move {
                        loop {
                            hopr_utils::runtime::prelude::sleep(touch_period).await;
                            let _ = sessions_keepalive.get(&session_id);
                        }
                    });

                    // TODO: consider making this interactive = other party reports the exact level periodically
                    let wait_result = level_stream
                        .skip_while(|current_level| {
                            futures::future::ready(*current_level < balancer_config.target_surb_buffer_size / 2)
                        })
                        .next()
                        .timeout(futures_time::time::Duration::from(SESSION_READINESS_TIMEOUT))
                        .await;
                    slot_keepalive.abort();
                    match wait_result {
                        Ok(Some(surb_level)) => {
                            info!(%session_id, surb_level, "session is ready");
                        }
                        Ok(None) => {
                            return Err(
                                SessionManagerError::other(anyhow!("surb balancer was cancelled prematurely")).into(),
                            );
                        }
                        Err(_) => {
                            warn!(%session_id, "session didn't reach target SURB buffer size in time");
                        }
                    }

                    #[cfg(all(feature = "telemetry", not(test)))]
                    METRIC_NUM_INITIATED_SESSIONS.increment();

                    let surb_estimator_for_rx = surb_estimator.clone();
                    let session = HoprSession::new_with_surb_state(
                        session_id,
                        forward_routing,
                        session_config_with(&self.cfg, cfg.capabilities, cfg.max_frames_behind_gap),
                        (
                            reduced_surb_scoring_sender,
                            session_rx.inspect(move |_| {
                                // Received packets = SURB consumption estimate
                                // The received packets always consume a single SURB.
                                surb_estimator_for_rx
                                    .consumed
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                // Same event, separate counter, on purpose — see `returned_packets`.
                                returned_packets.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                #[cfg(feature = "telemetry")]
                                telemetry::record_session_surb_consumed(&session_id, 1);
                            }),
                        ),
                        Some(notifier),
                        // Entry (sending) side: give flow control the SURB balancer state as its
                        // anti-grief down-only ceiling, and the client's opt-in flow-control config.
                        Some(surb_mgmt.clone()),
                        cfg.flow_control,
                    )?;

                    #[cfg(feature = "telemetry")]
                    initialize_session_telemetry(
                        session_id,
                        &self.cfg,
                        cfg.capabilities,
                        Some(&surb_estimator),
                        Some(&surb_mgmt),
                    );

                    slot_guard.commit();
                    Ok(session)
                } else {
                    warn!(%session_id, "session ready without SURB balancing");

                    // Counted here too, unlike `surb_estimator`: the PIX successor gate reads this,
                    // and a knob that binds on some Sessions and not others is how a deposit gate
                    // silently becomes a gate that never opens.
                    let returned_packets = Arc::new(std::sync::atomic::AtomicU64::new(0));
                    let returned_packets_for_rx = returned_packets.clone();

                    // Insert the slot and obtain a guard that rolls it back if any
                    // subsequent setup step fails.
                    let mut slot_guard = self
                        .allocate_session_slot(
                            session_id,
                            SessionSlot {
                                session_tx,
                                routing_opts: forward_routing.clone(),
                                abort_handles: Arc::new(parking_lot::Mutex::new(abort_handles)),
                                surb_mgmt: Default::default(), // Disabled SURB management
                                surb_estimator: Default::default(), // No SURB estimator needed
                                current_ssa_state,
                                // Entry side: the Exit is authoritative for the PIX lifecycle.
                                pix_supervisor: Default::default(),
                                pix_egress_gate: Default::default(),
                                returned_packets,
                                // Nor does it hold reconstructor state: the live-cycle budget is
                                // charged by the side that reconstructs.
                                cycle_budget: None,
                            },
                        )
                        .ok_or_else(|| {
                            // Session already exists; it means it is most likely a loopback attempt
                            error!(%session_id, "session already exists - loopback attempt");
                            SessionManagerError::Loopback
                        })?;

                    #[cfg(all(feature = "telemetry", not(test)))]
                    METRIC_NUM_INITIATED_SESSIONS.increment();

                    // For standard Session data we first reduce the number of SURBs we want to produce,
                    // unless requested to always max them out
                    let max_out_organic_surbs = cfg.always_max_out_surbs;
                    let reduced_surb_sender =
                        msg_sender.with(move |(routing, mut data): (DestinationRouting, ApplicationDataOut)| {
                            if !max_out_organic_surbs {
                                data.packet_info
                                    .get_or_insert_with(|| OutgoingPacketInfo {
                                        max_surbs_in_packet: 1,
                                        ..Default::default()
                                    })
                                    .max_surbs_in_packet = 1;
                            }
                            futures::future::ok::<_, S::Error>((routing, data))
                        });

                    let session = HoprSession::new(
                        session_id,
                        forward_routing,
                        session_config_with(&self.cfg, cfg.capabilities, cfg.max_frames_behind_gap),
                        (
                            reduced_surb_sender,
                            session_rx.inspect(move |_| {
                                returned_packets_for_rx.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }),
                        ),
                        Some(notifier),
                    )?;

                    #[cfg(feature = "telemetry")]
                    initialize_session_telemetry(session_id, &self.cfg, cfg.capabilities, None, None);

                    slot_guard.commit();
                    Ok(session)
                }
            }
            Ok(Ok(None)) => {
                self.session_initiations.remove(&challenge);
                Err(SessionManagerError::other(anyhow!(
                    "internal error: sender has been closed without completing the session establishment"
                ))
                .into())
            }
            Ok(Err(error)) => {
                // The other side did not allow us to establish a session
                let challenge = match error.identifier {
                    ErrorIdentifier::Challenge(c) => c,
                    ErrorIdentifier::SessionId(_) => {
                        // This arm should only ever receive pre-establishment errors.
                        // Log the id if it matters for debugging.
                        0
                    }
                };
                error!(
                    %challenge, ?error,
                    "the other party rejected the session initiation with error"
                );
                Err(TransportSessionError::Rejected(error.reason))
            }
            Err(_) => {
                // Timeout waiting for a session establishment
                error!(challenge, "session initiation attempt timed out");

                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_RECEIVED_SESSION_ERRS.increment(&["timeout"]);

                self.session_initiations.remove(&challenge);
                Err(TransportSessionError::Timeout)
            }
        }
    }

    /// Sends a keep-alive packet with the given [`SessionId`].
    ///
    /// This currently "fires & forgets" and does not expect nor await any "pong" response.
    pub async fn ping_session(&self, id: &SessionId) -> errors::Result<()> {
        if let Some(session_data) = self.sessions.get(id) {
            trace!(session_id = ?id, "pinging manually session");
            let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;
            send_via_msg_sender(
                &mut msg_sender,
                session_data.routing_opts.clone(),
                HoprStartProtocol::KeepAlive((*id).into()),
                "session ping message",
            )
            .await
            .map_err(TransportSessionError::packet_sending)
        } else {
            Err(SessionManagerError::NonExistingSession.into())
        }
    }

    /// Spawns the task that carries out one Session's supervisor actions.
    ///
    /// The supervisor decides; this executes. Keeping the two apart is what lets the state machine
    /// be pure and exhaustively tested, and it means every side effect a PIX Session can have is
    /// visible in one `match`.
    ///
    /// The task owns an [`SsaCommitmentGuard`] per SSA it has requested, which is what releases
    /// reconstructor state on teardown: aborting this task drops its future, and the guards with it.
    /// That bounds cleanup to the cycles actually in flight — at most two live plus one tombstone —
    /// where enumerating every index a Session had ever used grew without bound.
    fn spawn_pix_action_driver(
        &self,
        session_id: SessionId,
        slot: &SessionSlot,
        action_rx: ActionRx,
        reply_routing: DestinationRouting,
    ) -> hopr_utils::runtime::AbortHandle {
        let myself = self.clone();
        let gate = slot
            .pix_egress_gate
            .get()
            .cloned()
            .expect("the gate is installed before the driver is spawned");

        hopr_utils::spawn_as_abortable!(async move {
            // Dropping a guard releases its SSA, so the vector *is* the retirement mechanism —
            // draining it retires, and so does dropping this future.
            let mut owned_ssas: Vec<SsaCommitmentGuard<HoprPixSpec>> = Vec::new();

            let close_reason = loop {
                let Ok(action) = action_rx.recv().await else {
                    // The worker is gone without having said why, which is a failure of the
                    // supervisor itself rather than of the Session it was watching.
                    break Some(SessionPixCloseReason::SupervisorUnavailable);
                };

                match action {
                    SessionPixAction::RequestSsa { ssa_ids, params } => {
                        let Some(slot) = myself.sessions.get(&session_id) else {
                            // The Session went away underneath us; report the failure so the
                            // supervisor stops waiting on an action that can never complete.
                            myself
                                .report_ssa_request(
                                    &session_id,
                                    None,
                                    SessionPixAction::RequestSsa { ssa_ids, params },
                                    false,
                                )
                                .await;
                            continue;
                        };
                        match myself.send_ssa_request(session_id, &slot, &ssa_ids, params).await {
                            Ok(guards) => {
                                owned_ssas.extend(guards);
                                // One confirmation per index: the supervisor arms each cycle's
                                // commitment deadline on its own `SsaRequestSent`, and they were all
                                // put on the wire by the one send that just succeeded.
                                if let Some(supervisor) = slot.pix_supervisor.get() {
                                    for ssa_id in &ssa_ids {
                                        if supervisor
                                            .send_event(SessionPixEvent::SsaRequestSent(*ssa_id))
                                            .await
                                            .is_err()
                                        {
                                            error!(%session_id, "pix supervisor stopped accepting events");
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(error) => {
                                // Every guard was dropped by the early return inside
                                // `send_ssa_request`, so the whole batch of indices is released — none
                                // of them is left registered for a cycle that will never be asked for.
                                error!(
                                    %session_id, %error, batch_size = ssa_ids.len(),
                                    "failed to send ssa request"
                                );
                                if let Some(supervisor) = slot.pix_supervisor.get()
                                    && supervisor
                                        .send_action_result(
                                            // Only the discriminant and the ids are read on the
                                            // failure path, so the dimensions are echoed back
                                            // rather than zeroed — `PixParams` has no zero value,
                                            // and inventing one would be a lie about what was sent.
                                            SessionPixAction::RequestSsa { ssa_ids, params },
                                            false,
                                        )
                                        .await
                                        .is_err()
                                {
                                    error!(%session_id, "pix supervisor stopped accepting results");
                                }
                            }
                        }
                    }
                    SessionPixAction::ReleaseService => {
                        gate.release_service();
                        #[cfg(feature = "telemetry")]
                        crate::telemetry::set_pix_gate_mode(&session_id, true);
                    }
                    SessionPixAction::ProgressNotification => gate.notify_progress(),
                    SessionPixAction::RetireSsa(ssa_id) => {
                        // Dropping the guard is the retirement.
                        owned_ssas.retain(|guard| guard.ssa_id() != Some(&ssa_id));
                        if let Some(slot) = myself.sessions.get(&session_id) {
                            slot.abort_handles
                                .lock()
                                .abort_one(&SessionHandles::PixDepositObserver(ssa_id.ssa_index().get()));
                        }
                    }
                    SessionPixAction::Close(reason) => break Some(reason),
                }
            };

            let Some(reason) = close_reason else { return };
            error!(%session_id, %reason, "pix supervisor closed the session");

            // Unblock anything parked on the gate before tearing down: the supervisor that would
            // have woken it is the thing that just stopped.
            gate.poison();
            owned_ssas.clear();

            #[cfg(feature = "telemetry")]
            crate::telemetry::record_pix_closure(&reason.to_string());

            // Tell the Entry, so it can drop its side rather than wait out its own timeout. The
            // Session is closed either way, so a send failure here changes nothing.
            myself.notify_pix_failure(session_id, reply_routing).await;

            if let Some(slot) = myself.sessions.remove(&session_id) {
                myself.active_sessions.fetch_sub(1, Ordering::Relaxed);
                close_session(session_id, slot, ClosureReason::PixFailure);
            }
        })
    }

    /// Reports an action outcome to a Session's supervisor, if it still has one.
    async fn report_ssa_request(
        &self,
        session_id: &SessionId,
        slot: Option<&SessionSlot>,
        action: SessionPixAction,
        ok: bool,
    ) {
        let slot = match slot {
            Some(slot) => Some(slot.clone()),
            None => self.sessions.get(session_id),
        };
        if let Some(slot) = slot
            && let Some(supervisor) = slot.pix_supervisor.get()
            && supervisor.send_action_result(action, ok).await.is_err()
        {
            error!(%session_id, "pix supervisor stopped accepting results");
        }
    }

    /// Best-effort `SessionError` for an already-established Session, identified by its `SessionId`.
    ///
    /// Best-effort in the strict sense: the outcome is logged and discarded. Every caller is on a path
    /// that has already decided the Session is over, so there is nothing a failed send could change —
    /// and the peer has a deadline of its own as the backstop either way. What the notice buys is
    /// promptness and attribution: the peer tears down in about a round trip, with the reason in its
    /// log, instead of waiting out a timer that names the timer rather than the cause.
    ///
    /// The peer's `handle_session_error` closes the Session on a `SessionId`-identified error and sends
    /// nothing back, so there is no error exchange to loop.
    async fn notify_session_error(
        &self,
        session_id: SessionId,
        routing: DestinationRouting,
        reason: StartErrorReason,
        context: &'static str,
    ) {
        let Some(mut msg_sender) = self.msg_sender.get().cloned() else {
            warn!(%session_id, %reason, context, "cannot send session error - manager not started");
            return;
        };
        match send_via_msg_sender(
            &mut msg_sender,
            routing,
            HoprStartProtocol::SessionError(StartErrorType {
                identifier: ErrorIdentifier::SessionId(session_id),
                reason,
            }),
            context,
        )
        .await
        {
            Ok(()) => {
                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_SENT_SESSION_ERRS.increment(&[&reason.to_string()]);
            }
            Err(error) => warn!(%session_id, %reason, %error, context, "failed to send session error"),
        }
    }

    /// Best-effort notice to the Entry that this Session died for PIX reasons.
    async fn notify_pix_failure(&self, session_id: SessionId, reply_routing: DestinationRouting) {
        self.notify_session_error(
            session_id,
            reply_routing,
            StartErrorReason::Unknown,
            "session error after pix failure",
        )
        .await;
    }

    /// Registers an Exit commitment for every index in `ssa_indices` and asks the Entry to commit to
    /// the matching SSAs — all in a single [`SsaServerCommitmentMessage`].
    ///
    /// Driven by the supervisor's [`RequestSsa`](SessionPixAction::RequestSsa) action, which is what
    /// allocates the indices and fixes the dimensions — this only carries it out. Taking them as
    /// arguments rather than re-reading them from the slot keeps one source of truth for what was
    /// negotiated: the commitments registered here and the parameters the message advertises cannot
    /// disagree with what the supervisor is timing.
    ///
    /// One message for the whole batch, and one `params` field covering all of it, which is correct:
    /// every SSA in a Session uses the same negotiated dimensions. The Entry enforces its own ceiling
    /// on how many it will accept ([`SessionManagerConfig::max_ssas_per_ssa_request`]) and refuses an
    /// over-cap batch in full, so a batch larger than the peer allows loses the Session — see
    /// [`SupervisorConfig::ssas_per_request`](crate::SupervisorConfig::ssas_per_request).
    ///
    /// Installs no deadline of its own: every timeout that used to be armed here now belongs to the
    /// supervisor, which can see the whole cycle rather than just this one step.
    ///
    /// The returned [`SsaCommitmentGuard`]s own their registrations until the caller transfers or drops
    /// them. Handing them out rather than retaining them here is what makes the failure path safe:
    /// this function has fallible steps after the registrations exist — including partway through the
    /// batch — and an early return from any of them releases every one of them. That matters more for
    /// a batch than for a single SSA: without it one failed send would strand every index it had
    /// registered, and since the supervisor does not reuse an index, the Session could never recover.
    async fn send_ssa_request(
        &self,
        session_id: SessionId,
        slot: &SessionSlot,
        ssa_ids: &[SsaId<HoprPseudonym>],
        params: PixParams,
    ) -> errors::Result<Vec<SsaCommitmentGuard<HoprPixSpec>>> {
        let Some(first_ssa_id) = ssa_ids.first().copied() else {
            return Err(SessionManagerError::Other(anyhow!("ssa request with no indices")).into());
        };

        let pix_toolbox = self.pix_toolbox.get().cloned().ok_or(SessionManagerError::NotStarted)?;
        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        // One blocking task for the whole batch rather than one per SSA: each commitment is a single
        // random scalar and one generator multiplication, so per-task overhead would dominate.
        let ids = ssa_ids.to_vec();
        let (exit_commitments, guards) = hopr_utils::parallelize::cpu::spawn_blocking(
            move || {
                let mut commitments = Vec::with_capacity(ids.len());
                let mut guards = Vec::with_capacity(ids.len());
                for ssa_id in ids {
                    let (commitment, guard) = pix_toolbox
                        .share_processor
                        .new_guarded_exit_commitment(ssa_id, params)?;
                    commitments.push((ssa_id.ssa_index(), HoprPixGroupElement(commitment.to_bytes())));
                    guards.push(guard);
                }
                Ok::<_, hopr_protocol_pix::errors::PixError<HoprPseudonym>>((commitments, guards))
            },
            "server_ssa_commitment",
        )
        .await
        .map_err(SessionManagerError::other)?
        .map_err(SessionManagerError::PixError)?;

        info!(
            %session_id, batch_size = ssa_ids.len(), %params, %first_ssa_id,
            "generated exit commitments for the SSA batch"
        );

        // Construct and send the Exit SSA commitment request message.
        // The parameters were previously verified to be acceptable.
        let data = HoprStartProtocol::SsaRequest(SsaServerCommitmentMessage::new(
            session_id,
            params,
            exit_commitments,
            HoprPixDepositData::default(),
        ));

        send_via_msg_sender(
            &mut msg_sender,
            slot.routing_opts.clone(),
            data,
            "session SSA commitment request message",
        )
        .await
        .map_err(TransportSessionError::packet_sending)?;

        Ok(guards)
    }

    /// Returns the current number of active sessions.
    pub fn num_active_sessions(&self) -> usize {
        self.active_sessions.load(Ordering::Relaxed)
    }

    /// Returns [`SessionIds`](SessionId) of all currently active sessions.
    pub fn active_sessions(&self) -> Vec<SessionId> {
        self.sessions.run_pending_tasks();
        self.sessions.iter().map(|(k, _)| *k).collect()
    }

    /// Explicitly closes the session with the given `id`.
    ///
    /// Removes the entry from the internal session cache, closes the data channel,
    /// and aborts any auxiliary tasks. Returns `true` if a session was found and
    /// closed, `false` otherwise.
    ///
    /// This avoids waiting for the idle timeout (`time_to_idle`) or the LRU
    /// capacity bound to evict the entry, which is the desired behaviour when
    /// the caller (e.g. REST `DELETE /session`) knows the session is finished.
    pub fn close_session(&self, id: &SessionId) -> bool {
        if let Some(slot) = self.sessions.remove(id) {
            self.active_sessions.fetch_sub(1, Ordering::Relaxed);
            // Reconstructor state is released by `close_session` aborting the PIX action driver,
            // whose commitment guards retire on drop.
            close_session(*id, slot, ClosureReason::Eviction);
            true
        } else {
            false
        }
    }

    /// Updates the configuration of the SURB balancer on the given [`SessionId`].
    ///
    /// Returns an error if the Session with the given `id` does not exist, or
    /// if it does not use SURB balancing.
    pub fn update_surb_balancer_config(&self, id: &SessionId, config: SurbBalancerConfig) -> errors::Result<()> {
        let cfg = self
            .sessions
            .get(id)
            .ok_or(SessionManagerError::NonExistingSession)?
            .surb_mgmt;

        // Only update the config if there already was one before
        if !cfg.is_disabled() {
            cfg.update(&config);
            Ok(())
        } else {
            Err(SessionManagerError::other(anyhow!("session does not use SURB balancing")).into())
        }
    }

    /// Retrieves the configuration of SURB balancing for the given Session.
    ///
    /// Returns an error if the Session with the given `id` does not exist.
    pub fn get_surb_balancer_config(&self, id: &SessionId) -> errors::Result<Option<SurbBalancerConfig>> {
        match self.sessions.get(id) {
            Some(session) => Ok(Some(session.surb_mgmt.as_ref())
                .filter(|c| !c.is_disabled())
                .map(|d| d.as_config())),
            None => Err(SessionManagerError::NonExistingSession.into()),
        }
    }

    /// Gets estimations produced/received and consumed SURBs by the Session.
    ///
    /// For an outgoing Session (Entry) the pair is the number of SURBs sent (by us) and used (by the Exit).
    /// For an incoming Session (Exit) the pair is the number of SURBs received (from Entry) and used (by us).
    ///
    /// Returns an error if the Session with the given `id` does not exist.
    pub fn get_surb_level_estimates(&self, id: &SessionId) -> errors::Result<(u64, u64)> {
        match self.sessions.get(id) {
            Some(session) => Ok((
                session
                    .surb_estimator
                    .produced
                    .load(std::sync::atomic::Ordering::Relaxed),
                session
                    .surb_estimator
                    .consumed
                    .load(std::sync::atomic::Ordering::Relaxed),
            )),
            None => Err(SessionManagerError::NonExistingSession.into()),
        }
    }

    /// Forwards a PIX protocol observation to the Session's supervisor.
    ///
    /// The manager no longer interprets these. Deciding what an early-recovery signal or a failed
    /// share means for a Session's lifecycle needs the whole picture — which SSAs are in flight,
    /// what phase each is in, how much service has been consumed — and that lives in the supervisor.
    ///
    /// A `NonExistingSession` error is expected traffic rather than a fault: acknowledgements
    /// outlive their Session by up to the reconstructor's ack window, so events for a just-closed
    /// Session are routine. Callers distinguish it for exactly that reason.
    pub async fn dispatch_pix_event(&self, event: HoprSessionInPixEvent) -> errors::Result<()> {
        let session_id = event.pseudonym();
        let Some(slot) = self.sessions.get(session_id) else {
            debug!(%session_id, "pix event for a session that is no longer registered");
            return Err(SessionManagerError::NonExistingSession.into());
        };

        let Some(supervisor) = slot.pix_supervisor.get() else {
            // Entry side, or a Session that negotiated no PIX: nothing supervises it, so there is
            // nothing an observation could act on.
            trace!(%session_id, "pix event on a session without a supervisor");
            return Ok(());
        };

        // Progress is the one input whose rate is set by traffic rather than by lifecycle
        // transitions, so it is delivered without backpressure — awaiting channel capacity here
        // would put the supervisor's scheduling latency on the acknowledgement path. Safe by
        // construction rather than by tolerance: snapshots carry absolute counters and the state
        // machine keeps the maximum it has seen, so a dropped one is indistinguishable from a late
        // one, and the next one supersedes it.
        if let HoprSessionInPixEvent::RecoveryProgress(progress) = event {
            #[cfg(feature = "telemetry")]
            telemetry::set_pix_recovery_progress(session_id, progress.useful_shares, progress.target_useful_shares);

            if !supervisor.try_send_progress(progress) {
                trace!(%session_id, "dropped a pix progress snapshot on a full supervisor channel");
            }
            return Ok(());
        }

        let sent = match event {
            HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id) => {
                supervisor.send_event(SessionPixEvent::AlmostRecovered(ssa_id)).await
            }
            HoprSessionInPixEvent::SsaRecovered(ssa_id) => {
                supervisor.send_event(SessionPixEvent::Recovered(ssa_id)).await
            }
            HoprSessionInPixEvent::UnverifiableShares { ssa_id, observed_total } => {
                supervisor
                    .send_event(SessionPixEvent::UnverifiableShares { ssa_id, observed_total })
                    .await
            }
            HoprSessionInPixEvent::RecoveryProgress(_) => unreachable!("handled above"),
        };

        if sent.is_err() {
            error!(%session_id, "pix supervisor is no longer accepting events");
        }

        Ok(())
    }

    /// Marks the return path to `destination` as degraded on every Session routed there.
    ///
    /// The Session layer cannot tell a dead return path from a peer with nothing to say -- both
    /// simply stop consuming SURBs -- so the judgement is made where sibling paths can be compared
    /// and delivered here. Sessions that did not opt in ignore the mark; the rest stop trusting
    /// their counterparty buffer estimate for `grace`.
    ///
    /// Returns how many Sessions were marked, which is zero when nothing currently routes there.
    pub fn mark_return_path_degraded(
        &self,
        destination: &hopr_api::types::internal::NodeId,
        grace: std::time::Duration,
    ) -> usize {
        self.sessions
            .iter()
            .filter(|(_, slot)| {
                matches!(&slot.routing_opts, DestinationRouting::Forward { destination: d, .. } if d.as_ref() == destination)
            })
            .map(|(_, slot)| slot.surb_mgmt.mark_return_path_degraded(grace))
            .count()
    }

    /// The main method to be called whenever data are received.
    ///
    /// It tries to recognize the message and correctly dispatches either
    /// the Session protocol or Start protocol messages.
    ///
    /// If the data are not recognized, they are returned as [`DispatchResult::Unrelated`].
    pub fn dispatch_message(
        &self,
        pseudonym: HoprPseudonym,
        in_data: ApplicationDataIn,
    ) -> errors::Result<DispatchResult> {
        if in_data.data.application_tag == HoprStartProtocol::START_PROTOCOL_MESSAGE_TAG {
            // This is a Start protocol message, so we send it to the handler
            trace!("dispatching Start protocol message");
            if let Some(start_protocol_tx) = self.start_protocol_tx.get() {
                start_protocol_tx
                    .try_send((pseudonym, HoprStartProtocol::try_from(in_data.data)?))
                    .map_err(|error| {
                        error!(%error, "failed to send Start protocol message to processing task");
                        SessionManagerError::other(error)
                    })?;
            } else {
                return Err(SessionManagerError::NotStarted.into());
            }

            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_DISPATCHED_MSGS.increment_by(&["processed"], 1);

            return Ok(DispatchResult::Processed);
        } else if in_data.data.application_tag == SESSION_APPLICATION_TAG {
            // This is traffic that belongs to one of the Sessions
            let session_id = pseudonym;

            return if let Some(session_slot) = self.sessions.get(&session_id) {
                trace!(%session_id, "received data for a registered session");

                Ok(session_slot
                    .session_tx
                    .try_send(in_data)
                    .map(|_| {
                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_DISPATCHED_MSGS.increment_by(&["processed"], 1);

                        DispatchResult::Processed
                    })
                    .map_err(|error| {
                        error!(%session_id, %error, "failed to dispatch session data");
                        crate::counters::SESSION_INBOX_DROPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        SessionManagerError::other(error)
                    })?)
            } else {
                error!(%session_id, "received data from an unestablished session");
                crate::counters::SESSION_UNKNOWN_DATA_DROPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(TransportSessionError::UnknownData)
            };
        }

        trace!(tag = %in_data.data.application_tag, "received data not associated with session protocol or any existing session");

        crate::counters::SESSION_UNRELATED_DATA_DISPATCHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_DISPATCHED_MSGS.increment_by(&["unrelated"], 1);

        Ok(DispatchResult::Unrelated(in_data))
    }

    /// Pre-populates the sessions cache with a session slot for benchmarking.
    ///
    /// Intended for benchmarks that need a session to exist before calling
    /// [`SessionManager::dispatch_message`].
    ///
    /// Requires the `"benchmark"` feature.
    #[cfg(feature = "benchmark")]
    pub fn pre_populate_session(&self, session_id: SessionId, routing_opts: DestinationRouting) {
        let (session_tx, _) =
            crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(self.cfg.session_forward_capacity);
        let slot = SessionSlot {
            session_tx,
            routing_opts,
            abort_handles: Default::default(),
            surb_mgmt: Arc::new(BalancerStateValues::default()),
            surb_estimator: Default::default(),
            current_ssa_state: Default::default(),
            pix_supervisor: Default::default(),
            pix_egress_gate: Default::default(),
            returned_packets: Default::default(),
            // Never PIX, so there is no reconstructor state to charge for.
            cycle_budget: None,
        };
        self.sessions.insert(session_id, slot);
        self.slot_allocated
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&session_id);
    }

    /// Like [`pre_populate_session`](SessionManager::pre_populate_session) but also returns the
    /// session channel receiver so the caller can spawn a drain task.
    ///
    /// Requires the `"benchmark"` feature.
    #[cfg(feature = "benchmark")]
    pub fn pre_populate_session_with_receiver(
        &self,
        session_id: SessionId,
        routing_opts: DestinationRouting,
    ) -> crossfire::AsyncRx<crossfire::mpsc::Array<ApplicationDataIn>> {
        let (session_tx, session_rx) =
            crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(self.cfg.session_forward_capacity);
        let slot = SessionSlot {
            session_tx,
            routing_opts,
            abort_handles: Default::default(),
            surb_mgmt: Arc::new(BalancerStateValues::default()),
            surb_estimator: Default::default(),
            current_ssa_state: Default::default(),
            pix_supervisor: Default::default(),
            pix_egress_gate: Default::default(),
            returned_packets: Default::default(),
            // Never PIX, so there is no reconstructor state to charge for.
            cycle_budget: None,
        };
        self.sessions.insert(session_id, slot);
        self.slot_allocated
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&session_id);
        session_rx
    }

    /// Charges `params`' worth of reconstructor state against the node-wide budget.
    ///
    /// Returns `None` if the node has already committed to as much as
    /// [`IncomingSessionPixConfig::max_live_cycle_bytes`] allows — the caller must then refuse the
    /// Session, because nothing later in establishment can give the memory back.
    ///
    /// A CAS loop rather than an unconditional `fetch_add` with a rollback: an add that is
    /// provisionally over the ceiling is briefly visible to every concurrent initiation, and with
    /// enough of them arriving at once the budget would appear exhausted to Sessions that do fit.
    fn reserve_cycle_budget(&self, params: &PixParams) -> Option<Arc<CycleBudgetReservation>> {
        let bytes = cycle_budget_for(params, self.cfg.pix_config.supervision.ssas_per_request);
        let ceiling = self.cfg.pix_config.max_live_cycle_bytes;

        self.live_cycle_bytes
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |held| {
                held.checked_add(bytes).filter(|total| *total <= ceiling)
            })
            .ok()
            .map(|_| {
                Arc::new(CycleBudgetReservation {
                    bytes,
                    outstanding: self.live_cycle_bytes.clone(),
                    released: std::sync::atomic::AtomicBool::new(false),
                })
            })
    }

    /// Checks the PIX parameters offered by the Entry during the Session Initiation.
    ///
    /// Returns the validated parameters, or `None` if the offered parameters were rejected.
    fn check_pix_params(&self, req: &StartInitiation<SessionTarget, HoprSessionCapabilities>) -> Option<PixParams> {
        // TODO: the Exit may decide to use different quota based on the `target` in the StartInitiation message
        if req.capabilities.0.contains(Capability::UsePIX) {
            // Client offered PIX, so validate the offered parameters. Unpacking is what enforces the
            // protocol ranges on the three dimensions, and what rejects a suite identifier no curve
            // claims — leaving only "a known curve, but not ours" for the check below.
            let params = PixParams::try_from_additional_data(req.additional_data)
                .inspect_err(|error| {
                    debug!(
                        challenge = req.challenge,
                        %error,
                        "client offered PIX parameters outside the protocol ranges"
                    )
                })
                .ok()?;

            // The Exit decides the curve, and it decides it by refusing anything else: nothing here
            // is negotiated, because there is nothing to negotiate — the suite is fixed at build
            // time on both sides. Checked before the quota because it is the cheaper question and
            // because a suite mismatch makes the dimensions meaningless anyway, and checked at all
            // because every later PIX field is sized by it. Refusing here means the Exit's own
            // commitments, the first curve-sized bytes in the exchange, are never sent to a peer
            // that would read their boundaries in the wrong place.
            if params.suite() != LOCAL_PIX_SUITE {
                warn!(
                    challenge = req.challenge,
                    offered = %params.suite(),
                    ours = %LOCAL_PIX_SUITE,
                    "refusing a client offering a PIX curve suite this node was not built for"
                );
                return None;
            }

            let quota_per_ssa = pix_params_to_quota(&params);
            debug!(
                challenge = req.challenge,
                %params,
                acceptable_range = ?self.cfg.pix_config.quota_range,
                offered_quota_mb_per_ssa = quota_per_ssa as f64 / (1024.0 * 1024.0),
                "client offered MB SSA quota"
            );

            // The compared quota covers the whole cycle, surplus included, so the range bounds what
            // this Exit will actually serve rather than a fraction of it. See `pix_params_to_quota`.
            self.cfg
                .pix_config
                .quota_range
                .contains(&quota_per_ssa)
                .then_some(params)
        } else if self.cfg.pix_config.enforce_pix {
            // Client didn't offer PIX, but PIX is enforced
            None
        } else {
            // Client didn't offer PIX, and PIX is not enforced, so set default values
            // which are not going to be used.
            Some(DEFAULT_PIX_PARAMS)
        }
    }

    #[tracing::instrument(level = "debug", skip(self, session_req))]
    async fn handle_incoming_session_initiation(
        &self,
        pseudonym: HoprPseudonym,
        session_req: StartInitiation<SessionTarget, HoprSessionCapabilities>,
    ) -> errors::Result<()> {
        trace!(challenge = session_req.challenge, "received session initiation request");

        debug!("got new session request, searching for a free session slot");

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        // Reply routing uses SURBs only with the pseudonym of this Session's ID
        let reply_routing = DestinationRouting::Return(pseudonym.into());

        // Reject UsePIX if this node is not configured with a PixToolbox
        // (e.g. relay nodes that do not participate in PIX processing).
        if self.pix_toolbox.get().is_none() && session_req.capabilities.0.contains(Capability::UsePIX) {
            error!(
                challenge = session_req.challenge,
                "client offered PIX but this node has no PIX support installed"
            );
            let data = HoprStartProtocol::SessionError(StartErrorType {
                identifier: ErrorIdentifier::Challenge(session_req.challenge),
                reason: StartErrorReason::UnacceptablePixParams,
            });
            send_via_msg_sender(
                &mut msg_sender,
                reply_routing,
                data,
                "session error due to missing PIX support",
            )
            .await?;
            return Ok(());
        }

        // Verify if the client offered the right parameters for PIX
        let Some(client_params) = self.check_pix_params(&session_req) else {
            error!(
                challenge = session_req.challenge,
                "client offered unacceptable PIX parameters"
            );

            // Notify the sender that the session could not be established
            let reason = StartErrorReason::UnacceptablePixParams;
            let data = HoprStartProtocol::SessionError(StartErrorType {
                identifier: ErrorIdentifier::Challenge(session_req.challenge),
                reason,
            });
            send_via_msg_sender(
                &mut msg_sender,
                reply_routing,
                data,
                "session error message due to unacceptable PIX parameters",
            )
            .await?;

            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_SENT_SESSION_ERRS.increment(&[&reason.to_string()]);
            return Ok(());
        };

        info!(params = %client_params, "client offered acceptable PIX parameters");

        // Charge this Session's reconstructor state against the node-wide budget before anything is
        // allocated for it. Only a PIX Session holds cycle state, so only a PIX Session is charged —
        // `check_pix_params` hands back nominal parameters for a peer that offered none, and
        // reserving on those would bill Sessions that never reconstruct anything.
        //
        // Refused here rather than at the successor request, which is the other place the ceiling
        // could be applied: by then the Entry has funded a cycle and refusing costs it that deposit,
        // whereas a Session refused now is one the peer can retry elsewhere at no charge.
        let cycle_budget = if session_req.capabilities.0.contains(Capability::UsePIX) {
            let Some(reservation) = self.reserve_cycle_budget(&client_params) else {
                warn!(
                    challenge = session_req.challenge,
                    requested = cycle_budget_for(&client_params, self.cfg.pix_config.supervision.ssas_per_request),
                    outstanding = self.live_cycle_bytes.load(Ordering::Relaxed),
                    ceiling = self.cfg.pix_config.max_live_cycle_bytes,
                    "refusing a PIX session: the node's live reconstructor-cycle budget is exhausted"
                );

                let reason = StartErrorReason::NoSlotsAvailable;
                let data = HoprStartProtocol::SessionError(StartErrorType {
                    identifier: ErrorIdentifier::Challenge(session_req.challenge),
                    reason,
                });
                send_via_msg_sender(
                    &mut msg_sender,
                    reply_routing,
                    data,
                    "session error message due to an exhausted live-cycle budget",
                )
                .await?;

                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_SENT_SESSION_ERRS.increment(&[&reason.to_string()]);
                return Ok(());
            };
            Some(reservation)
        } else {
            None
        };

        let (new_session_notifier, close_session_notifier) = self
            .session_notifiers
            .get()
            .cloned()
            .ok_or(SessionManagerError::NotStarted)?;

        // Use constant application tag for all sessions
        self.sessions.run_pending_tasks();

        let session_id = pseudonym;

        let (session_tx, session_rx) =
            crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(self.cfg.session_forward_capacity);
        let (session_rx, session_rx_ah) = hopr_utils::runtime::DropAbortable::new(session_rx.into_stream());

        let slot = SessionSlot {
            session_tx,
            routing_opts: reply_routing.clone(),
            abort_handles: Default::default(),
            surb_mgmt: Default::default(),
            surb_estimator: Default::default(),
            current_ssa_state: Default::default(),
            pix_supervisor: Default::default(),
            pix_egress_gate: Default::default(),
            returned_packets: Default::default(),
            cycle_budget,
        };
        slot.abort_handles.lock().insert(SessionHandles::Ingress, session_rx_ah);

        // Insert the slot and get a guard. Any failure from here on rolls the slot
        // back, otherwise it would block this pseudonym until idle eviction. The atomic
        // insert (inside the helper) also prevents a TOCTOU race, so only one concurrent
        // request can claim the slot for a given pseudonym.
        let Some(mut slot_guard) = self.allocate_session_slot(session_id, slot.clone()) else {
            // No slots available for this pseudonym
            error!("no slots available for this pseudonym");
            let reason = StartErrorReason::NoSlotsAvailable;
            let data = HoprStartProtocol::SessionError(StartErrorType {
                identifier: ErrorIdentifier::Challenge(session_req.challenge),
                reason,
            });

            send_via_msg_sender(
                &mut msg_sender,
                reply_routing.clone(),
                data,
                "session error message due to lack of slots",
            )
            .await?;

            #[cfg(all(feature = "telemetry", not(test)))]
            METRIC_SENT_SESSION_ERRS.increment(&[&reason.to_string()]);

            return Ok(());
        };

        debug!(?session_req, "assigned a new session");

        // Stand the supervisor up before the `HoprSession` is constructed, so the egress adapters
        // below see a populated gate. Installing it afterwards would leave a window in which packets
        // are served ungated — small, but exactly the window an unfunded Entry would aim for.
        //
        // Its first `RequestSsa` action is deliberately *not* carried out here: the action driver
        // spawned after publication does it, which is what keeps `SessionEstablished` ahead of
        // `SsaRequest` on the wire.
        let pix: Option<ActionRx> = if self.pix_toolbox.get().is_some()
            && session_req.capabilities.0.contains(Capability::UsePIX)
        {
            // We use the same dimensions the client offered.
            slot.current_ssa_state
                .set(SessionSsaState::new(client_params))
                .map_err(|_| SessionManagerError::other(anyhow::anyhow!("session pix state must be uninitialized")))?;

            let (handle, action_rx) = spawn_supervisor_worker(
                self.cfg.pix_config.supervisor_config(),
                client_params,
                session_id,
                std::time::Instant::now(),
            );

            let _ = slot.pix_egress_gate.set(handle.gate.clone());
            let _ = slot.pix_supervisor.set(handle);
            Some(action_rx)
        } else {
            None
        };

        let closure_notifier = Box::new(move |session_id: SessionId, reason: ClosureReason| {
            if let Err(error) = close_session_notifier.try_send((session_id, reason)) {
                error!(%session_id, %error, %reason, "failed to notify session closure");
            }
        });

        let session = if !session_req.capabilities.0.contains(Capability::NoRateControl) {
            // Because of SURB scarcity, control the egress rate of incoming sessions
            let egress_rate_control =
                RateController::new(self.cfg.initial_return_session_egress_rate, Duration::from_secs(1));

            // The Session request carries a "hint" as additional data telling what
            // the Session initiator has configured as its target buffer size in the Balancer.
            // The lower 32 bits contain the SURB target; the upper 32 bits carry PIX
            // parameters and must be masked out.
            let surb_target = (session_req.additional_data & u32::MAX as u64) as u32;
            let target_surb_buffer_size = if surb_target > 0 {
                (surb_target as u64).min(self.cfg.maximum_surb_buffer_size as u64)
            } else {
                self.cfg.initial_return_session_egress_rate as u64
                    * self
                        .cfg
                        .minimum_surb_buffer_duration
                        .max(MIN_SURB_BUFFER_DURATION)
                        .as_secs()
            };

            let surb_estimator_clone = slot.surb_estimator.clone();
            // Resolved once, here, rather than per packet: the gate was installed before this point,
            // so the egress path never has to look inside the `OnceLock` again.
            let egress_gate = slot.pix_egress_gate.get().cloned();
            let session = HoprSession::new(
                session_id,
                reply_routing.clone(),
                session_config(&self.cfg, session_req.capabilities.into()),
                (
                    // Sent packets = SURB consumption estimate
                    msg_sender
                        .clone()
                        .sink_map_err(std::io::Error::other)
                        .with(move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                            // Each outgoing packet consumes one SURB
                            surb_estimator_clone
                                .consumed
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            #[cfg(feature = "telemetry")]
                            telemetry::record_session_surb_consumed(&session_id, 1);
                            acquire_egress_permit(egress_gate.clone(), routing, data)
                        })
                        .rate_limit_with_controller(&egress_rate_control)
                        .buffer((2 * target_surb_buffer_size) as usize),
                    // Received packets = SURB retrieval estimate
                    session_rx.inspect(move |data| {
                        let produced = data.num_surbs_with_msg() as u64;
                        // Count the number of SURBs delivered with each incoming packet
                        surb_estimator_clone
                            .produced
                            .fetch_add(produced, std::sync::atomic::Ordering::Relaxed);
                        #[cfg(feature = "telemetry")]
                        telemetry::record_session_surb_produced(&session_id, produced);
                    }),
                ),
                Some(closure_notifier),
            )?;

            // The SURB balancer will start intervening by rate-limiting the
            // egress of the Session, once the estimated number of SURBs drops below
            // the target defined here. Otherwise, the maximum egress is allowed.
            let balancer_config = SurbBalancerConfig {
                target_surb_buffer_size,
                // At maximum egress, the SURB buffer drains in `minimum_surb_buffer_duration` seconds
                max_surbs_per_sec: target_surb_buffer_size / self.cfg.minimum_surb_buffer_duration.as_secs(),
                // No SURB decay at the Exit, since we know almost exactly how many SURBs
                // were received
                surb_decay: None,
                sustain_on_return_path_loss: false,
            };

            slot.surb_mgmt.update(&balancer_config);
            slot.surb_mgmt
                .set_counterparty_buffer_capacity(self.cfg.maximum_surb_buffer_size as u64);

            // Spawn the SURB balancer only once we know we have registered the
            // abort handle with the pre-allocated Session slot
            debug!(%session_id, ?balancer_config ,"spawning exit SURB balancer");
            let balancer = SurbBalancer::new(
                session_id,
                SimpleBalancerController::default(),
                slot.surb_estimator.clone(),
                SurbControllerWithCorrection(egress_rate_control, 1), // 1 SURB per egress packet
                slot.surb_mgmt.clone(),
            );

            // Assign the SURB balancer and abort handles to the already allocated Session slot
            let (_, balancer_abort_handle) = balancer.start_control_loop(self.cfg.balancer_sampling_interval);
            slot.abort_handles
                .lock()
                .insert(SessionHandles::Balancer, balancer_abort_handle);

            // Spawn a keep-alive stream notifying about the SURB buffer level towards the Entry
            if let Some(period) = self.cfg.surb_balance_notify_period {
                let surb_estimator_clone = slot.surb_estimator.clone();
                let (ka_controller, ka_abort_handle) = utils::spawn_keep_alive_stream(
                    session_id,
                    // Deliberately not passed through the PIX egress gate. Keep-alives carry no
                    // payload and exist to report the SURB buffer level; gating them would let an
                    // exhausted predeposit budget silence the signal the Entry needs to keep the
                    // Session fundable, turning a stall into a teardown.
                    //
                    // Sent Keep-Alive packets also contribute to SURB consumption
                    msg_sender
                        .clone()
                        .with(move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                            // Each sent keepalive consumes 1 SURB
                            surb_estimator_clone
                                .consumed
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            #[cfg(feature = "telemetry")]
                            telemetry::record_session_surb_consumed(&session_id, 1);
                            futures::future::ok::<_, S::Error>((routing, data))
                        }),
                    slot.routing_opts.clone(),
                    SurbNotificationMode::Level(slot.surb_estimator.clone()),
                    slot.surb_mgmt.clone(),
                );

                // Start keepalive stream towards the Entry with a predefined period
                hopr_utils::runtime::prelude::spawn(async move {
                    // Delay the stream execution by one period
                    hopr_utils::runtime::prelude::sleep(period).await;
                    ka_controller.set_rate_per_unit(1, period);
                });

                slot.abort_handles
                    .lock()
                    .insert(SessionHandles::KeepAlive, ka_abort_handle);

                debug!(%session_id, ?period, "started SURB level-notifying keep-alive stream");
            }

            session
        } else {
            // `NoRateControl`: no SURB balancer, but the PIX gate still applies. A Session that
            // opts out of rate control is exactly the one that could drain the most service before
            // funding, so leaving this path ungated would make the predeposit budget optional.
            let egress_gate = slot.pix_egress_gate.get().cloned();
            HoprSession::new(
                session_id,
                reply_routing.clone(),
                session_config(&self.cfg, session_req.capabilities.into()),
                (
                    msg_sender.clone().sink_map_err(std::io::Error::other).with(
                        move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                            acquire_egress_permit(egress_gate.clone(), routing, data)
                        },
                    ),
                    session_rx,
                ),
                Some(closure_notifier),
            )?
        };

        // Extract useful information about the session from the Start protocol message
        let incoming_session = IncomingSession {
            id: session_id,
            session,
            target: session_req.target,
        };

        // Notify that a new incoming session has been created. Lock the sink and send
        // directly into it, so no extra forwarding task between channels is needed.
        match async {
            let mut guard = new_session_notifier.lock().await;
            guard.send(incoming_session).await
        }
        .timeout(futures_time::time::Duration::from(EXTERNAL_SEND_TIMEOUT))
        .await
        {
            Err(_) => {
                error!(%session_id, "timeout to notify about new incoming session");
                return Err(TransportSessionError::Timeout);
            }
            Ok(Err(error)) => {
                error!(%session_id, %error, "failed to notify about new incoming session");
                return Err(SessionManagerError::other(error).into());
            }
            _ => {}
        };

        trace!(?session_id, "session notification sent");

        // Notify the sender that the session has been established.
        // Set our peer ID in the session ID sent back to them.
        let data = HoprStartProtocol::SessionEstablished(StartEstablished {
            orig_challenge: session_req.challenge,
            session_id,
        });

        send_via_msg_sender(
            &mut msg_sender,
            reply_routing.clone(),
            data,
            "session establishment message",
        )
        .await?;

        #[cfg(feature = "telemetry")]
        initialize_session_telemetry(
            session_id,
            &self.cfg,
            session_req.capabilities.0,
            Some(&slot.surb_estimator),
            Some(&slot.surb_mgmt),
        );

        // The session is published, so the supervisor's actions can now be carried out. The first of
        // them is the initial `RequestSsa`, which is why this runs after `SessionEstablished` has
        // gone out: the Entry must see the two in that order.
        if let Some(action_rx) = pix {
            let driver = self.spawn_pix_action_driver(session_id, &slot, action_rx, reply_routing.clone());
            slot.abort_handles
                .lock()
                .insert(SessionHandles::PixActionDriver, driver);
        }

        info!(%session_id, "new session established");

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_NUM_ESTABLISHED_SESSIONS.increment();

        slot_guard.commit();

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, est))]
    async fn handle_session_established(&self, est: StartEstablished<SessionId>) -> errors::Result<()> {
        debug!(
            session_id = ?est.session_id,
            "received session establishment confirmation"
        );
        let challenge = est.orig_challenge;
        let session_id = est.session_id;

        if let Some(tx_est) = self.session_initiations.remove(&challenge) {
            if let Err(error) = tx_est.try_send(Ok(est)) {
                error!(%challenge, %session_id, %error, "failed to send session establishment confirmation");
                return Err(SessionManagerError::other(error).into());
            }
            debug!(?session_id, challenge, "session establishment complete");
        } else {
            error!(%session_id, challenge, "unknown session establishment attempt or expired");
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn handle_session_error(&self, error_type: StartErrorType<SessionId>) -> errors::Result<()> {
        let reason = error_type.reason;
        match error_type.identifier {
            ErrorIdentifier::Challenge(challenge) => {
                trace!(%challenge, error = ?error_type.reason, "session initiation error received");
                if let Some(tx_est) = self.session_initiations.remove(&challenge) {
                    if let Err(error) = tx_est.try_send(Err(error_type)) {
                        error!(%error, "could not send session error message");
                        return Err(SessionManagerError::other(error).into());
                    }
                    error!(%challenge, "session establishment error received");
                } else {
                    error!(
                        %challenge,
                        "session establishment attempt expired before error could be delivered"
                    );
                }
            }
            ErrorIdentifier::SessionId(session_id) => {
                error!(
                    %session_id, %reason,
                    "received post-establishment session error — closing session"
                );
                // Best-effort close; the session may have already been removed.
                self.close_session(&session_id);
            }
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_RECEIVED_SESSION_ERRS.increment(&[&reason.to_string()]);

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, msg))]
    async fn handle_keep_alive(&self, msg: KeepAliveMessage<SessionId>) -> errors::Result<()> {
        let session_id = msg.session_id;
        if let Some(session_slot) = self.sessions.get(&session_id) {
            trace!(?session_id, "received keep-alive message");
            match &session_slot.routing_opts {
                // Session is outgoing - keep-alive was received from the Exit
                DestinationRouting::Forward { .. } => {
                    if msg.flags.contains(KeepAliveFlag::BalancerState)
                        && !session_slot.surb_mgmt.is_disabled()
                        && session_slot.surb_mgmt.buffer_level() != msg.additional_data
                    {
                        // Update the buffer level as sent to us from the Exit
                        session_slot
                            .surb_mgmt
                            .buffer_level
                            .store(msg.additional_data, std::sync::atomic::Ordering::Relaxed);
                        debug!(%session_id, surb_level = msg.additional_data, "keep-alive updated SURB buffer size from the Exit");
                    }

                    // Increase the number of consumed SURBs in the estimator
                    session_slot
                        .surb_estimator
                        .consumed
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    #[cfg(feature = "telemetry")]
                    telemetry::record_session_surb_consumed(&session_id, 1);
                }
                // Session is incoming - keep-alive was received from the Entry
                DestinationRouting::Return(_) => {
                    // Allow updating SURB balancer target based on the received Keep-Alive message
                    if msg.flags.contains(KeepAliveFlag::BalancerTarget)
                        && msg.additional_data > 0
                        && !session_slot.surb_mgmt.is_disabled()
                        && session_slot.surb_mgmt.controller_bounds().target() != msg.additional_data
                    {
                        // Update the target buffer size as sent to us from the Entry
                        session_slot
                            .surb_mgmt
                            .target_surb_buffer_size
                            .store(msg.additional_data, std::sync::atomic::Ordering::Relaxed);
                        // Update maximum SURBs per second based on the new target
                        session_slot.surb_mgmt.max_surbs_per_sec.store(
                            msg.additional_data / self.cfg.minimum_surb_buffer_duration.as_secs(),
                            std::sync::atomic::Ordering::Relaxed,
                        );
                        debug!(%session_id, target_surb_buffer_size = msg.additional_data, "keep-alive updated SURB balancer target buffer size from the Entry");
                    }

                    // Increase the number of received SURBs in the estimator.
                    // Typically, 2 SURBs per Keep-Alive message
                    let produced = KeepAliveMessage::<SessionId>::MIN_SURBS_PER_MESSAGE as u64;
                    session_slot
                        .surb_estimator
                        .produced
                        .fetch_add(produced, std::sync::atomic::Ordering::Relaxed);
                    #[cfg(feature = "telemetry")]
                    telemetry::record_session_surb_produced(&session_id, produced);
                }
            }
        } else {
            debug!(%session_id, "received keep-alive request for an unknown session");
        }

        Ok(())
    }

    /// Handled by the Exit, when Entry replies with PIX commitment
    #[tracing::instrument(level = "debug", skip(self, msg))]
    async fn handle_ssa_commit(
        &self,
        pseudonym: HoprPseudonym,
        msg: SsaClientCommitmentMessage<SessionId, HoprPixGroupElement, HoprPixCommitmentProof>,
    ) -> errors::Result<()> {
        let Some(pix_toolbox) = self.pix_toolbox.get().cloned() else {
            return Err(SessionManagerError::UnsupportedMessage.into());
        };

        let session_id = msg.session_id;

        if pseudonym != session_id {
            error!(%pseudonym, %msg.session_id, "received SSA client commitment for a different session");
            return Err(SessionManagerError::NonExistingSession.into());
        }

        let Some(session_slot) = self.sessions.get(&session_id) else {
            return Err(SessionManagerError::NonExistingSession.into());
        };

        // See if we haven't received an SSA commitment for a Session that we did not register as PIX-capable
        let Some(quota_per_ssa) = session_slot.current_ssa_state.get().map(|s| s.quota_per_ssa()) else {
            return Err(SessionManagerError::Other(anyhow::anyhow!("no SSA state for session {session_id}")).into());
        };

        let ssa_id = SsaId::new(pseudonym, msg.ssa_index);

        // Decode the accompanying proof of knowledge, if the message carries one. A malformed proof
        // is rejected here rather than being passed on as absent, so that it cannot be mistaken for
        // a peer that simply did not send one.
        let commitment_proof = msg
            .commitment_proof
            .map(|proof| proof.try_into_pix_proof())
            .transpose()
            .map_err(SessionManagerError::other)?;

        // Insert the newly received coefficients into the SSA Reconstructor
        let pix_toolbox_clone = pix_toolbox.clone();
        let ssa_client_commitment_state = hopr_utils::parallelize::cpu::spawn_blocking(
            move || {
                pix_toolbox_clone
                    .share_processor
                    .insert_coefficient_commitments(
                        ssa_id,
                        msg.coefficient_index,
                        commitment_proof,
                        msg.coefficient_commitments.into_iter().map(|(k, v)| (k, v.0)),
                    )
                    .map_err(SessionManagerError::PixError)
            },
            "ssa commitment reconstructor",
        )
        .await
        .map_err(|_| {
            SessionManagerError::Other(anyhow::anyhow!(
                "failed to insert SSA coefficients into the SSA reconstructor"
            ))
        })??;

        // A verifiable commitment is what starts the deposit clock, so tell the supervisor before
        // the observer below can report anything against it.
        if ssa_client_commitment_state.is_verifiable
            && let Some(supervisor) = session_slot.pix_supervisor.get()
            && supervisor
                .send_event(SessionPixEvent::CommitmentVerified {
                    ssa_id,
                    // The negotiated quota is denominated in bytes, not balance, so this Exit states
                    // no per-SSA amount and accepts whatever `SupervisorConfig::min_deposit` allows.
                    expected_deposit: None,
                })
                .await
                .is_err()
        {
            error!(%session_id, %ssa_id, "pix supervisor is no longer accepting events");
        }

        if ssa_client_commitment_state.deposit_address_first_encountered
            && let Some(deposit_address) = ssa_client_commitment_state.ssa_deposit_address
        {
            // Inside the guard on purpose: every other `SsaCommit` of a cycle takes the other branch,
            // so allocating this before the `if` built and dropped a channel per message.
            let (deposit_done_tx, deposit_done_rx) = futures::channel::mpsc::channel(10);
            // Report deposits for as long as they arrive rather than waiting for the first one and
            // stopping: top-ups accumulate towards `min_deposit`, and the supervisor is what decides
            // when enough has landed. It also owns the deadline, so the observer carries none — a
            // timeout here would have been a second authority racing the first.
            let supervisor = session_slot.pix_supervisor.get().cloned();
            session_slot.abort_handles.lock().insert(
                SessionHandles::PixDepositObserver(ssa_id.ssa_index().get()),
                hopr_utils::spawn_as_abortable!(async move {
                    let Some(supervisor) = supervisor else {
                        // No supervisor: a Session that negotiated PIX always has one, so this is
                        // only reachable if it died between the check above and here.
                        return;
                    };
                    let mut confirmations = deposit_done_rx.filter(|((evt_pseudonym, evt_index), _)| {
                        futures::future::ready(evt_index == &ssa_id.ssa_index() && evt_pseudonym == ssa_id.pseudonym())
                    });
                    while let Some((_, amount)) = confirmations.next().await {
                        info!(%session_id, %ssa_id, %amount, "ssa deposit confirmed");
                        if supervisor
                            .send_event(SessionPixEvent::DepositConfirmed { ssa_id, amount })
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    // The upper layer dropped the sender without ever confirming, so no deposit is
                    // coming. Saying so is better than letting the supervisor wait out its deadline.
                    warn!(%session_id, %ssa_id, "deposit channel closed without confirmation");
                    let _ = supervisor
                        .send_event(SessionPixEvent::DepositObserverClosed(ssa_id))
                        .await;
                }),
            );

            // Notify upstream that deposit is needed
            pix_toolbox
                .pix_events
                .try_send(HoprSessionOutPixEvent::DepositNeeded(
                    AgreedSsaQuota {
                        ssa_id,
                        deposit_address,
                        quota_per_ssa,
                    },
                    deposit_done_tx,
                ))
                .map_err(|_| {
                    SessionManagerError::other(anyhow::anyhow!("failed to send pix event for needed deposit"))
                })?;
            info!(%ssa_id, %deposit_address, quota_per_ssa, "retrieved first client SSA commitment and deposit address");
        }

        Ok(())
    }

    /// Tells the Exit that its [`SsaServerCommitmentMessage`] was refused, and tears down this half
    /// of the Session.
    ///
    /// Without the notice the refusal is invisible to the Exit, and it has no way to recover from it.
    /// Its supervisor registered an Exit commitment per requested index and armed each cycle's
    /// commitment deadline the moment the request went out; it will never receive an `SsaCommit`, and
    /// nothing can make it re-ask, because a new `RequestSsa` only comes from share-recovery events and
    /// no shares exist for a cycle the Entry never committed to. So it serves the Session
    /// unincentivized for the whole batch-scaled `max_ssa_delivery_time` and then closes it as
    /// `CommitmentTimeout` — a reason that names the clock rather than the refusal, on the one node
    /// whose operator can act on it.
    ///
    /// Closing this half too is what stops the Entry sitting on a Session that can never make PIX
    /// progress: a refusal is terminal either way, since the Exit re-derives every request from state
    /// that cannot drift within a Session, so a later one would be refused identically. Left alone the
    /// slot would survive until the idle timeout.
    ///
    /// No new capability is handed to an attacker by closing on a refusal: an `SsaRequest` only reaches
    /// here Sphinx-authenticated and with `pseudonym == session_id`, so only the Exit can produce one —
    /// and the Exit can already close the Session whenever it likes.
    async fn refuse_ssa_request(&self, session_id: SessionId, routing: DestinationRouting) {
        self.notify_session_error(
            session_id,
            routing,
            StartErrorReason::UnacceptablePixParams,
            "session error due to a refused SSA request",
        )
        .await;

        if self.close_session(&session_id) {
            error!(%session_id, "closed session after refusing the Exit's SSA request");
        }
    }

    /// Handled by the Entry, when the Exit sends PIX initiation request
    #[tracing::instrument(level = "debug", skip(self, msg))]
    async fn handle_ssa_request(
        &self,
        pseudonym: HoprPseudonym,
        msg: SsaServerCommitmentMessage<SessionId, HoprPixGroupElement, HoprPixDepositData>,
    ) -> errors::Result<()> {
        let Some(pix_toolbox) = self.pix_toolbox.get().cloned() else {
            return Err(SessionManagerError::UnsupportedMessage.into());
        };

        if pseudonym != msg.session_id {
            error!(%pseudonym, %msg.session_id, "received SSA server commitment for a different session");
            return Err(SessionManagerError::NonExistingSession.into());
        }

        // The SsaRequest can arrive before new_session() or handle_incoming_session_initiation
        // has finished allocating the session slot, since both SessionEstablished and SsaRequest
        // are sent by the Exit back-to-back and processed concurrently by the Start protocol handler.
        // Instead of busy-looping, await the allocation notification for this specific SessionId.
        let session_slot = {
            use std::collections::hash_map::Entry;
            let session_id = msg.session_id;
            let waiter_deadline = Instant::now() + Duration::from_secs(30);
            loop {
                if Instant::now() >= waiter_deadline {
                    error!(%session_id, "session slot waiter deadline exceeded");
                    return Err(SessionManagerError::NonExistingSession.into());
                }
                // Optimistic cache check
                if let Some(slot) = self.sessions.get(&session_id) {
                    break slot;
                }

                // Register a waiter under the lock, then recheck the cache to avoid the
                // TOCTOU race between the initial cache check and waiter registration.
                let (tx, rx) = oneshot::channel::<()>();
                {
                    let mut map = self.slot_allocated.lock().unwrap_or_else(|e| e.into_inner());
                    // Recheck while holding the lock: the slot may have been inserted
                    // between the optimistic check and now. If so, don't register.
                    if self.sessions.get(&session_id).is_some() {
                        drop(map);
                        continue;
                    }
                    map.entry(session_id).or_default().push(tx);
                }

                let timeout = futures_time::time::Duration::from(Duration::from_millis(1000));
                match rx.timeout(timeout).await {
                    Ok(Ok(())) => {
                        // Notified — recheck the cache
                        continue;
                    }
                    _ => {
                        // Timeout or the sender was dropped (cancelled). Clean up our
                        // waiter entry to avoid unbounded accumulation in the map.
                        let mut map = self.slot_allocated.lock().unwrap_or_else(|e| e.into_inner());
                        if let Entry::Occupied(mut e) = map.entry(session_id) {
                            e.get_mut().retain(|w| !w.is_canceled());
                            if e.get().is_empty() {
                                e.remove();
                            }
                        }
                        drop(map);

                        // Final cache recheck before giving up — the slot might have
                        // been inserted while we were cleaning up.
                        if let Some(slot) = self.sessions.get(&session_id) {
                            break slot;
                        }

                        error!(%session_id, "session slot not found after awaiting allocation");
                        return Err(SessionManagerError::NonExistingSession.into());
                    }
                }
            }
        };

        debug!(
            num_server_commitments = msg.commitments.len(),
            "received Exit SSA commitments"
        );

        // Cap how many SSAs a single request may ask us to commit to.
        //
        // The wire format alone permits `MAX_SSAS_PER_REQUEST` (27) commitments per message, and
        // every accepted entry costs a full `new_ssa_commitment` (hundreds of thousands of EC
        // commitments) plus thousands of outbound `SsaCommit` packets, and emits its own
        // `ReadyToDeposit` — i.e. its own on-chain deposit. Without a cap, a single packet from a
        // misbehaving Exit amplifies into minutes of Entry CPU, a large packet burst, and up to 27
        // simultaneous deposits bounded only by the per-deposit allocation limit.
        //
        // Rejecting the whole message rather than the surplus is deliberate: a truncated batch would
        // leave the Exit holding reconstructor cycles for indices it will never receive commitments
        // for, which its own kill switch then has to clean up.
        let max_ssas_per_request = self.cfg.max_ssas_per_ssa_request;
        if msg.commitments.len() > max_ssas_per_request {
            let error = SessionManagerError::Unacceptable(format!(
                "Exit requested {} SSA commitments in a single request, at most {max_ssas_per_request} allowed",
                msg.commitments.len()
            ));
            self.refuse_ssa_request(msg.session_id, session_slot.routing_opts.clone())
                .await;
            return Err(error.into());
        }

        let Some(our_params) = session_slot.current_ssa_state.get().map(|s| s.params) else {
            return Err(
                SessionManagerError::Other(anyhow::anyhow!("no SSA state for session {}", msg.session_id)).into(),
            );
        };

        // The Entry enforces that the Exit's SSA parameters match exactly the ones we offered in the
        // Session Initiation message.  Negotiation (accepting an Exit-chosen quota within our
        // bounds) is not implemented, so any mismatch is rejected.
        //
        // The whole quadruple is compared rather than the scalar quota it implies.  Quota equality
        // was once argued to be sufficient — the Exit does not pick the dimensions independently, so
        // a matching product implied matching `(polys, shares)` from any Exit running unmodified
        // code.  The quota now prices three of the four, so it is no longer blind to the surplus,
        // but it is still a product: it cannot tell `polys x threshold` from a transposition of the
        // two, and those are not interchangeable to the protocol.  Comparing the params is both
        // stricter and simpler, and costs nothing now that all four travel together.
        //
        // The fourth, the curve suite, is why the Entry needs no separate curve check of its own:
        // the Exit refused a foreign suite before it sent this message, and an Exit that echoed a
        // different one back than the Entry offered fails right here.
        //
        // Malformed params never reach this comparison as a mismatch: `dimensions()` fails first,
        // and that failure takes the same refusal path, so the Exit is told either way rather than
        // being left waiting on a dropped packet.
        let server_params = match msg.dimensions() {
            Ok(params) => params,
            Err(error) => {
                self.refuse_ssa_request(msg.session_id, session_slot.routing_opts.clone())
                    .await;
                return Err(
                    SessionManagerError::Unacceptable(format!("Exit sent malformed PIX parameters: {error}")).into(),
                );
            }
        };
        if our_params != server_params {
            let error = SessionManagerError::Unacceptable(format!(
                "Exit sent unacceptable PIX parameters {server_params} (ours are {our_params})"
            ));
            self.refuse_ssa_request(msg.session_id, session_slot.routing_opts.clone())
                .await;
            return Err(error.into());
        }
        let quota_per_ssa = pix_params_to_quota(&our_params);

        // Everything from here to the end of the batch is serialised per pseudonym. Start messages run
        // under `for_each_concurrent`, so the successor gate below would otherwise be read by several
        // requests before any of them advanced it, and every one would pass. See `ssa_request_locks`.
        let request_lock = self
            .ssa_request_locks
            .get_with(pseudonym, async { Arc::new(futures::lock::Mutex::new(())) })
            .await;
        let _request_guard = request_lock.lock().await;

        // Successor gate, the Entry's half of the one in `SessionPixSupervisor`. A correct Exit asks
        // for the next batch when the *last* cycle of the current one is nearly recovered, by which
        // point we have long been emitting that cycle's shares. An Exit that asks earlier is asking
        // for deposits it cannot have earned, so nothing here commits and nothing is deposited.
        //
        // Two conditions, and the second is what the first alone cannot give. Emission must have
        // reached the last committed cycle — which, with the window clamped to one cycle, means every
        // earlier one is exhausted — *and* it must be far enough into that cycle. Checking only the
        // index admitted a successor batch on the last cycle's very first share, i.e. ~0 % of the way
        // through the batch rather than the ~86 % at which a conforming Exit asks. That is nearly a
        // whole cycle of unearned deposits, on a gate whose entire purpose is to prevent them.
        //
        // The boundary is derived rather than guessed — see `min_emission_for_early_recovery`, which
        // accounts for the Exit counting *polynomials* and for emission running in lockstep windows
        // that spend their whole surplus before the next window starts. A flat fraction of the cycle
        // gets this badly wrong in the unsafe direction: 0.85/1.5 is 57 %, against a true boundary of
        // 86.4 % at the deployed dimensions.
        //
        // Being early is refused rather than fatal, and deliberately not via `refuse_ssa_request`,
        // which closes the Session: that message is dropped and the Session left running. A correct
        // Exit never lands there; one that does has its own deadline for the cycles it already
        // allocated, and it can reach them without us having paid for anything. Lost generator state
        // is the one arm that *is* fatal, for the reason given at it.
        //
        // First index rather than each: `commitments` is a `BTreeMap`, so the lowest index leads, and
        // checking it before the loop is also what stops a batch that fails partway from having
        // already emitted `ReadyToDeposit` for its earlier members.
        //
        // Computed at the protocol floor rather than at our own reconstructor's threshold. The value
        // that decides when a correct Exit asks is *its* setting, which does not travel on the wire;
        // gating on ours refused a peer configured lower, silently, with no retry path — two valid
        // configurations that could not talk. See `MIN_EARLY_RECOVERY_THRESHOLD`, which every Exit is
        // held to by `validate_pix_supervision` and which is therefore the earliest any conforming
        // peer can ask.
        let min_emitted = hopr_protocol_pix::min_emission_for_early_recovery(
            &our_params,
            hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD,
        );
        match pix_toolbox.share_generator.emission_progress(&pseudonym) {
            Some(progress) if !progress.is_serving_last_committed() || progress.front_emitted < min_emitted => {
                let asked_first = msg.commitments.keys().next().copied();
                let error = SessionManagerError::Unacceptable(format!(
                    "Exit asked for SSAs from {asked_first:?} while emission has reached {:?} ({} of {min_emitted} \
                     shares needed for an early-recovery signal) of the batch committed up to {}",
                    progress.highest_emitted, progress.front_emitted, progress.last_committed
                ));
                warn!(session_id = %msg.session_id, %error, "refused an early SSA request");
                return Err(error.into());
            }
            // No generator state, but this Session has committed before: the entry was discarded
            // under us and the gate above has nothing left to measure.
            //
            // Absent state is otherwise the ordinary opening batch, which is why the gate treats it as
            // admissible — and that is the whole of the hole this closes. The generator keeps its
            // per-pseudonym entry in a cache with an idle retention, refreshed by share *emission*; a
            // Session kept alive on KeepAlives alone while the Entry sends nothing outlives it. An Exit
            // that arranges exactly that gets the successor gate deleted rather than merely relaxed —
            // no emission boundary, and no monotonic index either, since the discarded entry took the
            // high-watermark with it — and can then farm a fresh deposit per retention period.
            //
            // Terminal rather than a dropped message. The polynomials for the committed cycles went
            // with the entry, so `next_share` yields nothing and no share of them will ever be emitted:
            // whatever the Exit deposited against them is already unrecoverable and this half of the
            // Session can make no further PIX progress. Refusing quietly would leave the Exit to
            // discover that by timing out on commitments we have decided never to send.
            None if session_slot
                .current_ssa_state
                .get()
                .and_then(SessionSsaState::committed_watermark)
                .is_some() =>
            {
                let error = SessionManagerError::Unacceptable(format!(
                    "generator state for {pseudonym} was discarded while the Session was live; the cycles committed \
                     up to {:?} can no longer be served",
                    session_slot
                        .current_ssa_state
                        .get()
                        .and_then(SessionSsaState::committed_watermark)
                ));
                warn!(session_id = %msg.session_id, %error, "refused an SSA request against lost generator state");
                self.refuse_ssa_request(msg.session_id, session_slot.routing_opts.clone())
                    .await;
                return Err(error.into());
            }
            _ => {}
        }

        // Second half of the successor gate, and the one that measures the Exit rather than us.
        //
        // Emission above is this node's own work: it counts shares handed to the local packet
        // pipeline by `create_surb_for_path`, a consumption that is not even rolled back when the
        // rest of the packet build fails. An Exit that requests, is funded, and then returns nothing
        // still walks that counter forward for as long as we keep sending, so on its own it prices
        // deposits against work we did to ourselves.
        //
        // What is checked here is service that actually arrived. A share is encrypted with the first
        // relayer's challenge solution and rides a return SURB, so the Exit can only decrypt it by
        // *using* that SURB — one returned packet is one SURB consumed is one share unlocked. The
        // Exit cannot inflate this without unlocking exactly as many shares, which advances the
        // recovery it is claiming to have made. Nothing it reports is trusted.
        //
        // Skipped before the first commitment: the opening batch is the one nothing has been paid
        // for yet, and there is no service to have been rendered against it.
        if let Some(state) = session_slot.current_ssa_state.get()
            && let Some(watermark) = state.committed_watermark()
        {
            let shares_per_cycle = our_params.polys_per_ssa() as u64 * our_params.emitted_shares_per_poly() as u64;
            let target = (watermark.get() as u64 - 1) * shares_per_cycle + min_emitted;

            // Discounted by exactly the loss the surplus insures against. The Exit unlocks a share
            // when the *first relayer* acknowledges, which is upstream of us, so every packet lost
            // after that point is progress it legitimately has and we cannot see. Demanding the
            // undiscounted figure would refuse conforming Exits on any lossy path; an Exit losing
            // more than the surplus covers could not have reconstructed the cycle anyway.
            //
            // `u128` for the multiplication only — at the extremes of the accepted ranges the
            // numerator overflows `u64`. The quotient cannot, since the ratio is at most one.
            let required = (target as u128 * our_params.shares_per_poly() as u128
                / our_params.emitted_shares_per_poly() as u128) as u64;

            let served = |slot: &SessionSlot| {
                state.served_since_first_commit(slot.returned_packets.load(std::sync::atomic::Ordering::Relaxed))
            };
            let mut observed = served(&session_slot);

            if observed < required {
                // A conforming Exit asks the instant its reconstructor crosses the threshold, and the
                // request travels the same mixed path as the packets that earned it — so it can
                // overtake the last few of them. Refusing that is not a refusal at all: `RequestSsa`
                // is emitted once per index and never retried, so the Exit sits in
                // `AwaitingCommitment` until `max_ssa_delivery_time` and then closes the Session.
                // A short wait costs nothing and saves the Session.
                //
                // Entered only for a near miss, and that is what keeps the wait from being a lever:
                // this handler runs under a bounded `for_each_concurrent` and holds the
                // per-pseudonym request lock, so an Exit must already have returned all but one
                // emission window of what it owes to occupy either. Anything further out is refused
                // on the spot, at no cost to us.
                //
                // One window rather than a fraction of `required`: it is the unit emission advances
                // in, so a shortfall below it is genuinely in-flight, and a percentage would round to
                // zero at exactly the small dimensions where this matters most.
                let near_miss = observed > 0 && required - observed <= hopr_protocol_pix::SHARE_EMISSION_WINDOW as u64;
                if near_miss {
                    let deadline = Instant::now() + SSA_SUCCESSOR_SERVICE_WAIT;
                    while observed < required && Instant::now() < deadline {
                        hopr_utils::runtime::prelude::sleep(SSA_SUCCESSOR_SERVICE_POLL).await;
                        observed = served(&session_slot);
                    }
                }
            }

            if observed < required {
                let error = SessionManagerError::Unacceptable(format!(
                    "Exit asked for SSAs having returned {observed} of the {required} packets its batch committed up \
                     to {watermark} has been paid for"
                ));
                warn!(session_id = %msg.session_id, %error, "refused an under-served SSA request");
                return Err(error.into());
            }
        }

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;
        let session_id = msg.session_id;

        // The server can theoretically send multiple SSA commitments asking us to make the equal
        // number of client commitments and deposits, and the batch is all-or-nothing: either every
        // member gets a commitment and a `ReadyToDeposit`, or none does.
        //
        // Hence three phases rather than one loop. Interleaving them meant a batch whose *second*
        // exit commitment failed to decode had already sent the first member's `SsaCommit` burst and
        // emitted its `ReadyToDeposit` — an on-chain deposit instruction — before the failure was
        // reached. The Exit, whose own request was rejected as a whole, has no cycle to spend it on.
        //
        // The server is authoritative in giving the ssa_index; the client only verifies that it is
        // strictly monotonic. That monotonicity is enforced inside `new_ssa_commitment` in phase two,
        // which rejects any `ssa_index` that is `<=` the last one generated for this pseudonym with
        // `PixError::InvalidInput` (see `SsaShareGenerator::new_ssa_commitment`). Because that call
        // happens in a phase before anything is published, a stale, duplicate, or out-of-order
        // `SsaRequest` cannot cause a deposit — the whole message is rejected first. The
        // per-pseudonym baseline lives in the generator's `polynomials` cache (30-min idle TTL,
        // refreshed on every use), so it persists for the life of an active session. Gaps (an index
        // strictly greater than the last, but not the immediate successor) are allowed by design,
        // since the Exit may advance by more than one SSA at a time.

        // Phase 1 — validate. Decoding is the only step that consumes attacker-supplied bytes:
        // `try_into_pix_group` decompresses the point and rejects anything outside the prime-order
        // subgroup. Doing every one of them up front is what makes a malformed later member cost
        // nothing, and it is cheap relative to phase two.
        let mut validated = Vec::with_capacity(msg.commitments.len());
        for (ssa_index, exit_commitment) in msg.commitments {
            trace!(ssa_index, "received Exit SSA commitment");
            match exit_commitment.try_into_pix_group() {
                Ok(point) => validated.push((ssa_index, point)),
                Err(error) => {
                    // Terminal, like every other unacceptable-parameter case here. A peer that cannot
                    // produce a valid group element is not going to produce one on a retry, and the
                    // Exit would otherwise wait out `max_ssa_delivery_time` on commitments that are
                    // never coming.
                    let error = SessionManagerError::Unacceptable(format!(
                        "Exit sent an undecodable SSA commitment for index {ssa_index}: {error}"
                    ));
                    self.refuse_ssa_request(session_id, session_slot.routing_opts.clone())
                        .await;
                    return Err(error.into());
                }
            }
        }

        // Phase 2 — stage. Generates every client commitment and derives every deposit address,
        // publishing none of them. This is the expensive phase, and it still mutates the generator:
        // `new_ssa_commitment` appends to the per-pseudonym polynomial queue. Failing here therefore
        // leaves queued polynomials the Exit never learns of — wasted work rather than a leaked
        // deposit instruction, and the generator's own monotonic index keeps them from being mistaken
        // for a later cycle's.
        let mut staged = Vec::with_capacity(validated.len());
        for (ssa_index, exit_point) in validated {
            // Use the global `pix_toolbox.share_generator` to generate the client
            // commitment. The generator is shared with the packet pipeline's
            // `next_share`, so polynomials created here will be used when the
            // pipeline embeds PIX shares into return-path SURBs.
            //
            // The generator dimension (polys × threshold) must match what the
            // Exit's reconstructor expects — both are set from the session's
            // negotiated PIX params (pix_global_config on Entry → SsaRequest
            // params on Exit).  If the client sends commitments that exceed the
            // Exit's expected dimensions, the Exit rejects them as InvalidInput.
            let pix_toolbox_clone = pix_toolbox.clone();
            let client_commitment = hopr_utils::parallelize::cpu::spawn_blocking(
                move || {
                    pix_toolbox_clone
                        .share_generator
                        .new_ssa_commitment(&pseudonym, ssa_index)
                },
                "client_ssa_commitment",
            )
            .await
            .map_err(SessionManagerError::other)?
            .map_err(SessionManagerError::PixError)?;

            // The generator has been advanced for this index, so the Session records it now rather
            // than after the publish phase. Recording it early is the safe direction: the watermark
            // only ever makes the successor gate stricter, and a batch abandoned between here and
            // publication has still moved the generator.
            if let Some(state) = session_slot.current_ssa_state.get() {
                state.note_committed(
                    ssa_index,
                    session_slot.returned_packets.load(std::sync::atomic::Ordering::Relaxed),
                );
            }

            // Construct the full SSA by adding the client and exit commitments, getting the deposit address
            let full_ssa = client_commitment.ssa_commitment + exit_point;
            let deposit_address = HoprPixSpec::group_to_deposit_address(full_ssa).ok_or(SessionManagerError::other(
                anyhow::anyhow!("failed to convert SSA to deposit address"),
            ))?;

            // Split the SSA client commitment into Start protocol commitment messages
            let commitment_msgs = SsaClientCommitmentMessage::new_multiple(session_id, client_commitment)
                .map_err(SessionManagerError::other)?;
            debug!(%ssa_index, count = commitment_msgs.len(), "generated client SSA commitment messages");

            staged.push((ssa_index, deposit_address, commitment_msgs));
        }

        // Phase 3 — publish. Nothing below can fail on the *content* of the request; only the
        // transport can, and a transport that has stopped accepting messages fails the Session
        // regardless of where in the batch it happens.
        for (ssa_index, deposit_address, commitment_msgs) in staged {
            // Send each commitment message into the message sender
            for commitment_msg in commitment_msgs {
                send_via_msg_sender(
                    &mut msg_sender,
                    session_slot.routing_opts.clone(),
                    HoprStartProtocol::SsaCommit(commitment_msg),
                    "client SSA commitment message",
                )
                .await?;
            }

            debug!(%ssa_index, "all Entry SSA commitment messages were sent out");

            // Notify the new SSA deposit address *after* all commitment messages have been
            // sent out successfully, so the deposit cannot begin before the Exit has the
            // complete commitment to reconstruct the deposit key.
            pix_toolbox
                .pix_events
                .try_send(HoprSessionOutPixEvent::ReadyToDeposit(AgreedSsaQuota {
                    ssa_id: SsaId::new(pseudonym, ssa_index),
                    deposit_address,
                    quota_per_ssa,
                }))
                .map_err(|_| SessionManagerError::other(anyhow::anyhow!("failed to notify new deposit ssa")))?;
            info!(%ssa_index, %deposit_address, quota_per_ssa, "generated client SSA commitment and deposit address");
        }

        trace!(quota_per_ssa, "Exit commitment message has been fully processed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, anyhow};
    use futures::{AsyncWriteExt, channel::mpsc::UnboundedSender, future::BoxFuture, pin_mut};
    use hopr_api::types::{
        crypto::{keypairs::ChainKeypair, prelude::Keypair},
        crypto_random::Randomizable,
        internal::routing::SurbMatcher,
        primitive::prelude::Address,
    };
    use hopr_protocol_pix::{MAX_POLY_THRESHOLD, SsaGeneratorConfig, SsaIndex, SsaReconstructorConfig};
    use hopr_protocol_start::{StartProtocol, StartProtocolDiscriminants};
    use hopr_utils::network_types::prelude::SealedHost;
    use moka::future::FutureExt;
    use tokio::time::timeout;

    use super::*;
    use crate::{Capabilities, balancer::SurbBalancerConfig, types::SessionTarget};

    /// `StartInitiation::additional_data` as an Entry offering these dimensions would send it.
    ///
    /// Tests go through the same packing production does. They used to write the shifts out by
    /// hand, which meant a change to the layout altered what every one of them was asserting
    /// without altering a single line of them — plain `u64` literals type-check against anything.
    fn pix_additional_data(polys_per_ssa: u16, shares_per_poly: u8, surplus_shares: u8) -> u64 {
        PixParams::try_new(polys_per_ssa, shares_per_poly, surplus_shares, LOCAL_PIX_SUITE)
            .expect("test dimensions must be valid")
            .into_additional_data(0)
    }

    /// The default test dimensions: the smallest legal split, with the surplus the test generators
    /// below are configured with.
    fn small_pix_params() -> PixParams {
        PixParams::try_new(2, 2, TEST_SURPLUS_SHARES, LOCAL_PIX_SUITE).expect("test dimensions must be valid")
    }

    /// [`small_pix_params`] as an Entry offering them would pack them into `additional_data`.
    fn small_pix_additional_data() -> u64 {
        small_pix_params().into_additional_data(0)
    }

    /// Surplus used by the small test `SsaGeneratorConfig`s below. Non-zero and different from
    /// [`SsaGeneratorConfig::default`], so a value that failed to cross the wire is visible.
    const TEST_SURPLUS_SHARES: u8 = 1;

    #[test]
    fn session_config_forwards_max_buffered_segments() {
        assert_eq!(
            SessionManagerConfig::default().max_buffered_segments,
            0,
            "default must leave the transport unbuffered"
        );

        for segments in [0, 64] {
            let cfg = SessionManagerConfig {
                max_buffered_segments: segments,
                ..Default::default()
            };
            assert_eq!(
                session_config(&cfg, Capabilities::empty()).max_buffered_segments,
                segments
            );
        }
    }

    /// The head-of-line bound must reach a session that cannot recover a missing frame, and must
    /// not reach one that can.
    ///
    /// Both halves matter. Without the first, a session with no retransmission holds every frame
    /// behind a gap for the full frame timeout, waiting for something that is never coming —
    /// measured on a cluster as 98.5 % of bytes arriving over the wire and 0.60 % reaching the
    /// application. Without the second, a session that *would* have retransmitted the gap instead
    /// abandons it, turning recoverable frames into loss.
    #[test]
    fn session_config_should_bound_the_gap_only_without_retransmission() {
        assert_eq!(
            SessionManagerConfig::default().max_frames_behind_gap,
            Some(256),
            "the default must bound the gap, or the stall stays in place unless opted out of"
        );

        let cfg = SessionManagerConfig {
            max_frames_behind_gap: Some(8),
            ..Default::default()
        };

        for reliable in [Capability::RetransmissionAck, Capability::RetransmissionNack] {
            assert_eq!(
                session_config(&cfg, reliable.into()).max_frames_behind_gap,
                None,
                "{reliable:?} can recover the gap, so the wait is productive and must be left alone"
            );
        }

        for unreliable in [Capabilities::empty(), Capability::Segmentation.into()] {
            assert_eq!(
                session_config(&cfg, unreliable).max_frames_behind_gap,
                Some(8),
                "without retransmission the gap must be bounded"
            );
        }
    }

    /// The right value tracks throughput x latency spread, which is a property of the *session*,
    /// not of the node: a bulk data session and a control session on the same node have entirely
    /// different reordering depths. A caller that knows its own traffic must be able to say so.
    #[test]
    fn a_session_should_be_able_to_override_the_nodes_gap_bound() {
        let node = SessionManagerConfig {
            max_frames_behind_gap: Some(256),
            ..Default::default()
        };

        assert_eq!(
            session_config_with(&node, Capabilities::empty(), Some(16)).max_frames_behind_gap,
            Some(16),
            "the session's own value must win over the node default"
        );
        assert_eq!(
            session_config_with(&node, Capabilities::empty(), None).max_frames_behind_gap,
            Some(256),
            "saying nothing must inherit the node default"
        );
        assert_eq!(
            session_config_with(&node, Capabilities::empty(), Some(0)).max_frames_behind_gap,
            None,
            "zero disables the bound for this session, matching the env knob's semantics"
        );
    }

    /// A per-session override must not be able to re-enable the bound where waiting is productive.
    #[test]
    fn a_session_override_should_not_reach_a_session_that_can_retransmit() {
        let node = SessionManagerConfig::default();
        assert_eq!(
            session_config_with(&node, Capability::RetransmissionAck.into(), Some(4)).max_frames_behind_gap,
            None,
            "retransmission can recover the gap, so no caller should be able to cut the wait short"
        );
    }

    /// Disabling has to be reachable, since the bound trades reordering tolerance for latency and
    /// the right value is deployment-specific. `None` restores the previous behaviour exactly.
    #[test]
    fn session_config_should_allow_the_gap_bound_to_be_disabled() {
        let cfg = SessionManagerConfig {
            max_frames_behind_gap: None,
            ..Default::default()
        };
        assert_eq!(session_config(&cfg, Capabilities::empty()).max_frames_behind_gap, None);
    }

    #[test]
    fn a_zero_gap_bound_should_disable_it_at_the_node_level_too() {
        // `0` means "not for me" wherever it is written. Read literally it would be the strictest
        // possible bound -- abandon the gap before a single frame arrives behind it -- so the two
        // levels would mean opposite things by the same value.
        let cfg = SessionManagerConfig {
            max_frames_behind_gap: Some(0),
            ..Default::default()
        };
        assert_eq!(session_config(&cfg, Capabilities::empty()).max_frames_behind_gap, None);
    }

    #[async_trait::async_trait]
    trait SendMsg {
        async fn send_message(
            &self,
            routing: DestinationRouting,
            data: ApplicationDataOut,
        ) -> crate::errors::Result<()>;
    }

    mockall::mock! {
        MsgSender {}
        impl SendMsg for MsgSender {
            fn send_message<'a, 'b>(&'a self, routing: DestinationRouting, data: ApplicationDataOut)
            -> BoxFuture<'b, crate::errors::Result<()>> where 'a: 'b, Self: Sync + 'b;
        }
    }

    fn mock_packet_planning(
        sender: MockMsgSender,
    ) -> (
        UnboundedSender<(DestinationRouting, ApplicationDataOut)>,
        tokio::task::JoinHandle<anyhow::Result<()>>,
    ) {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let handle = tokio::task::spawn(async move {
            pin_mut!(rx);
            while let Some((routing, data)) = rx.next().await {
                sender.send_message(routing, data).await?;
            }
            Ok(())
        });
        (tx, handle)
    }

    fn msg_type(data: &ApplicationDataOut, expected: StartProtocolDiscriminants) -> bool {
        HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
            .map(|d| StartProtocolDiscriminants::from(d) == expected)
            .unwrap_or(false)
    }

    fn start_msg_match(data: &ApplicationDataOut, msg: impl Fn(HoprStartProtocol) -> bool) -> bool {
        HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
            .map(msg)
            .unwrap_or(false)
    }

    /// Waits (bounded) until the manager reports no active sessions.
    ///
    /// The session-slot rollback runs on a spawned task, so its effect is observed
    /// asynchronously; this polls [`SessionManager::active_sessions`] until it drains.
    async fn wait_for_no_active_sessions(
        mgr: &SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>,
    ) -> bool {
        for _ in 0..50 {
            if mgr.active_sessions().is_empty() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        mgr.active_sessions().is_empty()
    }

    const SESSION_FORWARD_CAPACITY: usize = 10000;

    /// Verifies that a session's SURB balancer config can be retrieved and updated via the manager API.
    ///
    /// ## Steps
    /// 1. A session slot is manually inserted into Alice's manager with a known `SurbBalancerConfig`
    ///    (`target_surb_buffer_size: 1000`, `max_surbs_per_sec: 100`).
    /// 2. `get_surb_balancer_config` returns the config, confirming round-trip storage.
    /// 3. `update_surb_balancer_config` is called with a new config (`target: 2000`, `max: 200`).
    /// 4. `get_surb_balancer_config` is called again and the returned config matches the updated values.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_update_surb_balancer_config() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let session_id = alice_pseudonym;
        let balancer_cfg = SurbBalancerConfig {
            target_surb_buffer_size: 1000,
            max_surbs_per_sec: 100,
            ..Default::default()
        };

        let alice_mgr =
            SessionManager::<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>::new(Default::default());

        let (dummy_tx, _) = crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(SESSION_FORWARD_CAPACITY);
        alice_mgr.sessions.insert(
            session_id,
            SessionSlot {
                session_tx: dummy_tx,
                routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(alice_pseudonym)),
                abort_handles: Default::default(),
                surb_mgmt: Arc::new(BalancerStateValues::from(balancer_cfg)),
                surb_estimator: Default::default(),
                current_ssa_state: Default::default(),
                pix_supervisor: Default::default(),
                pix_egress_gate: Default::default(),
                returned_packets: Default::default(),
                cycle_budget: None,
            },
        );

        let actual_cfg = alice_mgr
            .get_surb_balancer_config(&session_id)?
            .ok_or(anyhow!("session must have a surb balancer config"))?;
        assert_eq!(actual_cfg, balancer_cfg);

        let new_cfg = SurbBalancerConfig {
            target_surb_buffer_size: 2000,
            max_surbs_per_sec: 200,
            ..Default::default()
        };
        alice_mgr.update_surb_balancer_config(&session_id, new_cfg)?;

        let actual_cfg = alice_mgr
            .get_surb_balancer_config(&session_id)?
            .ok_or(anyhow!("session must have a surb balancer config"))?;
        assert_eq!(actual_cfg, new_cfg);

        Ok(())
    }

    /// Verifies that a self-initiated session is rejected with `SessionManagerError::Loopback`.
    ///
    /// ## Steps
    /// 1. Alice's manager is started with a mock transport that delivers messages back to itself.
    /// 2. Alice initiates a session toward `bob_peer`; the mock routes her `StartSession` back to her own manager
    ///    (simulating a network loopback).
    /// 3. Alice's manager processes `StartSession` as incoming, auto-responds with `SessionEstablished`, and the mock
    ///    delivers it back to complete the handshake.
    /// 4. `new_session` returns `Err(TransportSessionError::Manager(SessionManagerError::Loopback))`.
    /// 5. Exactly one active session is present — the incoming slot accepted from the self-delivered `StartSession`.
    ///    The rejection fires after slot insertion, not before.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_not_allow_loopback_sessions() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let alice_mgr = SessionManager::new(Default::default());

        let mut sequence = mockall::Sequence::new();
        let mut alice_transport = MockMsgSender::new();

        // Alice sends the StartSession message
        let alice_mgr_clone = alice_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
            })
            .returning(move |_, data| {
                // But the message is again processed by Alice due to Loopback
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        ?;
                    Ok(())
                })
            });

        // Alice sends the SessionEstablished message (as Bob)
        let alice_mgr_clone = alice_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
            })
            .returning(move |_, data| {
                let alice_mgr_clone = alice_mgr_clone.clone();

                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(
                        alice_pseudonym,
                        ApplicationDataIn {
                            data: data.data,
                            packet_info: Default::default(),
                        },
                    )?;
                    Ok(())
                })
            });

        // Start Alice
        let (new_session_tx_alice, new_session_rx_alice) = futures::channel::mpsc::channel(1024);
        let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
        alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?;
        assert!(alice_mgr.is_started());

        let alice_session = alice_mgr
            .new_session(
                bob_peer,
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: None.into(),
                    pseudonym: alice_pseudonym.into(),
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await;

        println!("{alice_session:?}");
        assert!(matches!(
            alice_session,
            Err(TransportSessionError::Manager(SessionManagerError::Loopback))
        ));
        // There is one session in the manager, which is the incoming one that Alice's manager
        // accepted when it received the StartSession message from itself.
        assert_eq!(alice_mgr.num_active_sessions(), 1);

        drop(new_session_rx_alice);

        // Cleanup: close sender and await handle
        alice_sender.close_channel();
        alice_handle.await??;

        Ok(())
    }

    /// Verifies that a session initiation returns `TransportSessionError::Timeout` when the peer
    /// never processes or responds to the `StartSession` message.
    ///
    /// ## Steps
    /// 1. Alice's manager is configured with `initiation_timeout_base: 100ms`. Bob's manager is started but its mock
    ///    transport silently swallows all messages (never dispatches to the manager).
    /// 2. Alice calls `new_session`; her `StartSession` is captured by the mock and silently discarded.
    /// 3. The 100ms timeout expires; `new_session` returns `Err(TransportSessionError::Timeout)`.
    /// 4. `num_active_sessions` is 0, confirming no orphaned slot was left in the cache.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_timeout_new_session_attempt_when_no_response() -> anyhow::Result<()> {
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let cfg = SessionManagerConfig {
            initiation_timeout_base: Duration::from_millis(100),
            ..Default::default()
        };

        let alice_mgr = SessionManager::new(cfg);
        let bob_mgr = SessionManager::new(Default::default());

        let mut sequence = mockall::Sequence::new();
        let mut alice_transport = MockMsgSender::new();
        let bob_transport = MockMsgSender::new();

        // Alice sends the StartSession message, but Bob does not handle it
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
            })
            .returning(|_, _| Box::pin(async { Ok(()) }));

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        let (alice_sender, _alice_handle) = mock_packet_planning(alice_transport);
        alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?;
        assert!(alice_mgr.is_started());

        // Start Bob
        let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
        let (bob_sender, _bob_handle) = mock_packet_planning(bob_transport);
        bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?;
        assert!(bob_mgr.is_started());

        let result = alice_mgr
            .new_session(
                bob_peer,
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: None.into(),
                    pseudonym: None,
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await;

        assert!(matches!(result, Err(TransportSessionError::Timeout)));
        assert_eq!(alice_mgr.num_active_sessions(), 0);

        Ok(())
    }

    /// Verifies that a failed incoming session establishment does not register any telemetry.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with the `telemetry` feature enabled.
    /// 2. The new-session notification channel's receiver is dropped immediately, so notifying about a new incoming
    ///    session will fail.
    /// 3. `handle_incoming_session_initiation` is called with a random pseudonym. The slot is inserted first, then
    ///    notifying about the new session fails (receiver is gone).
    /// 4. `wait_for_no_active_sessions` polls until there are no active sessions, confirming the partially-inserted
    ///    slot was rolled back.
    /// 5. `num_active_sessions` is 0, proving the rollback prevented any telemetry registration for the failed session.
    #[cfg(feature = "telemetry")]
    #[test_log::test(tokio::test)]
    async fn failed_incoming_session_establishment_does_not_register_telemetry() -> anyhow::Result<()> {
        let mgr = SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        drop(new_session_rx);
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let pseudonym = HoprPseudonym::random();
        let result = mgr
            .handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities(Capabilities::empty()),
                    additional_data: 0,
                },
            )
            .await;

        assert!(result.is_err());

        // The slot inserted before the failure must be rolled back, so it neither
        // counts towards `maximum_sessions` nor registers any telemetry.
        assert!(
            wait_for_no_active_sessions(&mgr).await,
            "the partially established session slot was not rolled back"
        );
        assert_eq!(mgr.num_active_sessions(), 0);

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that a session slot is rolled back if session setup fails after the slot is inserted.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started; the new-session notification channel's receiver is dropped, so notifying about
    ///    a new incoming session will fail.
    /// 2. `handle_incoming_session_initiation` is called with a random pseudonym. The slot is inserted into the cache
    ///    first, then notifying about the new session fails (because the receiver is gone).
    /// 3. The call returns an error, and `wait_for_no_active_sessions` confirms the slot was removed.
    /// 4. `num_active_sessions` is 0, proving the rollback removed the slot and freed the pseudonym.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_roll_back_slot_when_incoming_session_setup_fails() -> anyhow::Result<()> {
        let mgr = SessionManager::new(Default::default());

        // Drop the receiver so that notifying about the new incoming session fails
        // *after* the session slot has already been inserted into the cache.
        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        drop(new_session_rx);
        let (sender, handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let pseudonym = HoprPseudonym::random();

        // The setup fails after the slot is inserted (notifying about the new
        // incoming session errors out because the receiver is gone), so the slot
        // must be rolled back instead of lingering until idle eviction.
        let result = mgr
            .handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities::empty(),
                    additional_data: 0,
                },
            )
            .await;
        assert!(result.is_err());

        // An empty active-session set proves the slot was removed and, since
        // sessions are keyed by pseudonym, that the pseudonym is free again.
        assert!(
            wait_for_no_active_sessions(&mgr).await,
            "the partially established session slot was not rolled back"
        );

        // Cleanup
        sender.close_channel();
        handle.await??;

        Ok(())
    }

    /// Collects everything that arrives on `rx` during `window`, returning once it elapses.
    async fn originated_during(
        rx: &mut futures::channel::mpsc::UnboundedReceiver<(DestinationRouting, ApplicationDataOut)>,
        window: Duration,
    ) -> Vec<(DestinationRouting, ApplicationDataOut)> {
        let mut collected = Vec::new();
        let deadline = tokio::time::Instant::now() + window;
        while let Ok(Some(item)) = timeout(
            deadline.saturating_duration_since(tokio::time::Instant::now()),
            rx.next(),
        )
        .await
        {
            collected.push(item);
        }
        collected
    }

    /// The manager floors the keep-alive period at [`MIN_SURB_BUFFER_NOTIFICATION_PERIOD`], so this
    /// is as fast as an Exit keep-alive can be made to run, and it sets the pace of these tests.
    const KEEP_ALIVE_PERIOD: Duration = MIN_SURB_BUFFER_NOTIFICATION_PERIOD;

    type RecordingManager = SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>;
    type Originated = futures::channel::mpsc::UnboundedReceiver<(DestinationRouting, ApplicationDataOut)>;

    /// Brings up an Exit-side session and returns once its keep-alive stream is observably running.
    ///
    /// The "observably running" part is load-bearing for every caller: `nothing was originated` is
    /// equally true of a stream that stopped and one that never started, so a test that does not
    /// first establish the stream is alive proves nothing when it later sees silence.
    async fn exit_session_originating_keep_alives(
        cfg: SessionManagerConfig,
    ) -> anyhow::Result<(RecordingManager, Originated, HoprPseudonym)> {
        let mgr = RecordingManager::new(SessionManagerConfig {
            surb_balance_notify_period: Some(KEEP_ALIVE_PERIOD),
            ..cfg
        });

        let (msg_tx, mut msg_rx) = futures::channel::mpsc::unbounded();
        // Held, not dropped: dropping it would fail the establishment notification instead.
        let (new_session_tx, _new_session_rx) = futures::channel::mpsc::channel(4);
        mgr.start(msg_tx, new_session_tx, None)?;

        let pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                // Empty capabilities keep rate control on, which is what spawns the balancer and
                // the keep-alive stream. `NoRateControl` would skip both and make this vacuous.
                capabilities: HoprSessionCapabilities(Capabilities::empty()),
                additional_data: 0,
            },
        )
        .await?;

        let observed = originated_during(&mut msg_rx, KEEP_ALIVE_PERIOD * 2 + Duration::from_millis(500)).await;
        let keep_alives = observed
            .iter()
            .filter(|(routing, data)| {
                msg_type(data, StartProtocolDiscriminants::KeepAlive)
                    && matches!(routing, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &pseudonym)
            })
            .count();
        anyhow::ensure!(
            keep_alives > 0,
            "no return-routed keep-alive was originated, so this test cannot tell a stopped stream from one that \
             never ran; {} message(s) were observed in total",
            observed.len()
        );

        Ok((mgr, msg_rx, pseudonym))
    }

    /// Asserts `mgr` originates nothing further for `pseudonym`.
    ///
    /// Packets already handed to the sender before the closure are in flight rather than newly
    /// originated, so they are drained first and only what appears afterwards counts.
    async fn assert_no_further_origination(rx: &mut Originated, closure: &str) {
        let _in_flight = originated_during(rx, Duration::from_millis(200)).await;

        let window = KEEP_ALIVE_PERIOD * 3;
        let after = originated_during(rx, window).await;
        assert!(
            after.is_empty(),
            "an Exit session closed by {closure} originated {} further packet(s) over {window:?} — each one is \
             return-routed to a pseudonym whose SURBs are gone, and one such packet is enough to stall all \
             origination on the node",
            after.len()
        );
    }

    /// An Exit session must originate nothing once it has been closed explicitly.
    ///
    /// The Exit's keep-alive stream is a `repeat_with` on a rate limiter: it produces a
    /// return-routed packet every period regardless of whether a SURB exists to carry it. That is
    /// fine while the initiator is present and replenishing, and it is the *supply* side of the
    /// `london-01` outage once the initiator is gone — every such packet is one the routing
    /// resolution stage can never resolve, and one unresolvable packet there stalls all origination
    /// on the node (see `hopr_transport::path::resolve`).
    ///
    /// Teardown is therefore the only thing standing between a departed initiator and an unbounded
    /// supply of unresolvable packets, which is why each closure path has to be shown to stop the
    /// stream rather than merely drop the slot.
    #[test_log::test(tokio::test)]
    async fn an_exit_session_should_originate_nothing_after_an_explicit_close() -> anyhow::Result<()> {
        // Default idle timeout (180 s), so eviction cannot confound what the explicit close proves.
        let (mgr, mut msg_rx, pseudonym) = exit_session_originating_keep_alives(Default::default()).await?;

        assert!(mgr.close_session(&pseudonym), "the session must exist to be closed");

        assert_no_further_origination(&mut msg_rx, "an explicit close").await;
        Ok(())
    }

    /// An Exit session must originate nothing once it has been evicted for being idle.
    ///
    /// This is the path that matters most for a departed initiator: nobody closes that session, so
    /// idle eviction is what ends it, and eviction runs through a Moka listener rather than the
    /// explicit close path. A keep-alive stream that survives eviction would go on originating
    /// unresolvable return packets with no session left to account for them.
    #[test_log::test(tokio::test)]
    async fn an_exit_session_should_originate_nothing_after_idle_eviction() -> anyhow::Result<()> {
        let idle_timeout = KEEP_ALIVE_PERIOD * 3;
        let (mgr, mut msg_rx, _) = exit_session_originating_keep_alives(SessionManagerConfig {
            idle_timeout,
            ..Default::default()
        })
        .await?;

        // Moka evicts lazily, so drive its maintenance rather than waiting for the manager's own
        // (jittered, multi-second) eviction tick: this makes the eviction prompt and deterministic.
        for _ in 0..50 {
            mgr.sessions.run_pending_tasks();
            if mgr.active_sessions().is_empty() {
                break;
            }
            tokio::time::sleep(idle_timeout / 10).await;
        }
        assert!(
            mgr.active_sessions().is_empty(),
            "the idle session was never evicted, so this test cannot say anything about eviction"
        );

        assert_no_further_origination(&mut msg_rx, "idle eviction").await;
        Ok(())
    }

    /// Verifies that established sessions exchange `KeepAlive` messages driven by the SURB balancer,
    /// that config updates propagate via keep-alives, and that SURB usage statistics are collected.
    ///
    /// ## Steps
    /// 1. Alice's manager is started with no `PixToolbox` and a `SurbBalancerConfig` with `target_surb_buffer_size:
    ///    10`. Bob's manager is configured with a 500ms `surb_balance_notify_period`.
    /// 2. Alice initiates a session with the balancer config and PIX quota set; the `StartSession` /
    ///    `SessionEstablished` handshake completes via mock transports.
    /// 3. Both managers report the same `target_surb_buffer_size` via `get_surb_balancer_config` (confirmed from both
    ///    Alice and Bob's perspective).
    /// 4. A 1500ms sleep allows the SURB balancer's periodic keep-alive timer to fire multiple times.
    /// 5. `update_surb_balancer_config` is called to raise the target to 50. After another 1500ms, Bob's manager
    ///    reflects the updated target via `get_surb_balancer_config`, confirming keep-alives communicated the change.
    /// 6. `get_surb_level_estimates` is called on both sides; both report positive sent/received/used counts,
    ///    confirming the balancer collected SURB statistics.
    /// 7. Alice closes the session; `ping_session` returns `NonExistingSession` after a short wait.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_send_keep_alives_via_surb_balancer() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let bob_cfg = SessionManagerConfig {
            surb_balance_notify_period: Some(Duration::from_millis(500)),
            ..Default::default()
        };
        let alice_mgr = SessionManager::new(Default::default());
        let bob_mgr = SessionManager::new(bob_cfg.clone());

        let mut alice_transport = MockMsgSender::new();
        let mut bob_transport = MockMsgSender::new();

        // Alice sends the StartSession message
        let mut open_sequence = mockall::Sequence::new();
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut open_sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
            })
            .returning(move |_, data| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        ?;
                    Ok(())
                })
            });

        // Bob sends the SessionEstablished message
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut open_sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
            })
            .returning(move |_, data| {
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(
                        alice_pseudonym,
                        ApplicationDataIn {
                            data: data.data,
                            packet_info: Default::default(),
                        },
                    )?;
                    Ok(())
                })
            });

        const INITIAL_BALANCER_TARGET: u64 = 10;

        // Alice sends the KeepAlive messages reporting the initial balancer target
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .times(1..)
            //.in_sequence(&mut sequence)
            .withf(move |peer, data| {
                start_msg_match(data, |msg| matches!(msg, StartProtocol::KeepAlive(ka) if ka.flags.contains(KeepAliveFlag::BalancerTarget) && ka.additional_data == INITIAL_BALANCER_TARGET))
                //msg_type(data, StartProtocolDiscriminants::KeepAlive)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
            })
            .returning(move |_, data| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        ?;
                    Ok(())
                })
            });

        const NEXT_BALANCER_TARGET: u64 = 50;

        // Alice sends also the KeepAlive messages reporting the updated balancer target
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .times(1..)
            //.in_sequence(&mut sequence)
            .withf(move |peer, data| {
                start_msg_match(data, |msg| matches!(msg, StartProtocol::KeepAlive(ka) if ka.flags.contains(KeepAliveFlag::BalancerTarget) && ka.additional_data == NEXT_BALANCER_TARGET))
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
            })
            .returning(move |_, data| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        ?;
                    Ok(())
                })
            });

        // Bob sends at least 1 Keep Alive back reporting its SURB estimate
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .times(1..)
            //.in_sequence(&mut open_sequence)
            .withf(move |peer, data| {
                start_msg_match(data, |msg| matches!(msg, StartProtocol::KeepAlive(ka) if ka.flags.contains(KeepAliveFlag::BalancerState) && ka.additional_data > 0))
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if *p == alice_pseudonym)
            })
            .returning(move |_, data| {
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        ?;
                    Ok(())
                })
            });

        // Alice sends the terminating segment to close the Session
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            //.in_sequence(&mut sequence)
            .withf(move |peer, data| {
                hopr_protocol_session::types::SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::try_from(
                    data.data.plain_text.as_ref(),
                )
                .ok()
                .and_then(|m| m.try_as_segment())
                .map(|s| s.is_terminating())
                .unwrap_or(false)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination.as_ref() == &bob_peer.into())
            })
            .returning(move |_, data| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        ?;
                    Ok(())
                })
            });

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
        ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?);
        assert!(alice_mgr.is_started());

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?);
        assert!(bob_mgr.is_started());

        let target = SealedHost::Plain("127.0.0.1:80".parse()?);

        let balancer_cfg = SurbBalancerConfig {
            target_surb_buffer_size: INITIAL_BALANCER_TARGET,
            max_surbs_per_sec: 100,
            ..Default::default()
        };

        pin_mut!(new_session_rx_bob);
        let (alice_session, bob_session) = timeout(
            Duration::from_secs(2),
            futures::future::join(
                alice_mgr.new_session(
                    bob_peer,
                    SessionTarget::TcpStream(target.clone()),
                    SessionClientConfig {
                        pseudonym: alice_pseudonym.into(),
                        capabilities: Capability::Segmentation.into(),
                        surb_management: Some(balancer_cfg),
                        ..Default::default()
                    },
                ),
                new_session_rx_bob.next(),
            ),
        )
        .await?;

        let mut alice_session = alice_session?;
        let bob_session = bob_session.ok_or(anyhow!("bob must get an incoming session"))?;

        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        assert_eq!(
            Some(balancer_cfg),
            alice_mgr.get_surb_balancer_config(alice_session.id())?
        );

        let remote_cfg = bob_mgr
            .get_surb_balancer_config(bob_session.session.id())?
            .ok_or(anyhow!("no remote config at bob"))?;
        assert_eq!(remote_cfg.target_surb_buffer_size, balancer_cfg.target_surb_buffer_size);
        assert_eq!(
            remote_cfg.max_surbs_per_sec,
            remote_cfg.target_surb_buffer_size
                / bob_cfg
                    .minimum_surb_buffer_duration
                    .max(MIN_SURB_BUFFER_DURATION)
                    .as_secs()
        );

        // Let the Surb balancer send enough KeepAlive messages
        tokio::time::sleep(Duration::from_millis(1500)).await;

        let new_balancer_cfg = SurbBalancerConfig {
            target_surb_buffer_size: NEXT_BALANCER_TARGET,
            max_surbs_per_sec: 100,
            ..Default::default()
        };

        // Update to a higher target
        alice_mgr.update_surb_balancer_config(alice_session.id(), new_balancer_cfg)?;

        // Let the Surb balancer send enough KeepAlive messages
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Bob should know about the updated target
        let remote_cfg = bob_mgr
            .get_surb_balancer_config(bob_session.session.id())?
            .ok_or(anyhow!("no remote config at bob"))?;
        assert_eq!(
            remote_cfg.target_surb_buffer_size,
            new_balancer_cfg.target_surb_buffer_size
        );
        assert_eq!(
            remote_cfg.max_surbs_per_sec,
            new_balancer_cfg.target_surb_buffer_size / bob_cfg.minimum_surb_buffer_duration.as_secs()
        );

        let (alice_surb_sent, alice_surb_used) = alice_mgr.get_surb_level_estimates(alice_session.id())?;
        let (bob_surb_recv, bob_surb_used) = bob_mgr.get_surb_level_estimates(bob_session.session.id())?;

        alice_session.close().await?;

        assert!(alice_surb_sent > 0, "alice must've sent surbs");
        assert!(bob_surb_recv > 0, "bob must've received surbs");
        assert!(
            bob_surb_recv <= alice_surb_sent,
            "bob cannot receive more surbs than alice sent"
        );

        assert!(alice_surb_used > 0, "alice must see bob used surbs");
        assert!(bob_surb_used > 0, "bob must've used surbs");
        assert!(
            alice_surb_used <= bob_surb_used,
            "alice cannot see bob used more surbs than bob actually used"
        );

        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(matches!(
            alice_mgr.ping_session(alice_session.id()).await,
            Err(TransportSessionError::Manager(SessionManagerError::NonExistingSession))
        ));

        futures::stream::iter(ahs)
            .for_each(|ah| async move { ah.abort() })
            .await;

        // Cleanup: close senders and await handles
        alice_sender.close_channel();
        bob_sender.close_channel();
        alice_handle.await??;
        bob_handle.await??;

        Ok(())
    }

    /// Verifies that a second incoming session initiation for the same pseudonym is handled gracefully
    /// (returns `Ok`) without creating a duplicate session slot.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport that accepts two outbound messages.
    /// 2. `handle_incoming_session_initiation` is called with pseudonym `X` — succeeds; exactly one active session is
    ///    confirmed.
    /// 3. `handle_incoming_session_initiation` is called again with the same pseudonym `X`. The manager detects the
    ///    conflict and handles it internally by sending a `SessionError` to the peer.
    /// 4. The call still returns `Ok` (error is handled internally); `num_active_sessions` remains 1 with only the
    ///    original pseudonym present.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_reject_duplicate_session_for_same_pseudonym() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        let bob_mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        // Start the manager (required for handling incoming sessions)
        let mut transport = MockMsgSender::new();
        transport
            .expect_send_message()
            .times(2)
            .returning(|_, _| futures::future::ok(()).boxed());

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        // Spawn a task to receive new session notifications
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {
                // Just drain the channel
            }
        });
        let (sender, _handle) = mock_packet_planning(transport);
        bob_mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(bob_mgr.is_started());

        let pseudonym = HoprPseudonym::random();

        // First session initiation - should succeed
        let result = bob_mgr
            .handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities::empty(),
                    additional_data: 0,
                },
            )
            .await;

        assert!(result.is_ok(), "first session initiation should succeed");

        // Verify one session exists
        let active = bob_mgr.active_sessions();
        assert_eq!(active.len(), 1, "should have exactly one active session");
        assert_eq!(bob_mgr.num_active_sessions(), 1);

        // Second session initiation with same pseudonym - should be handled gracefully
        // (returns Ok but sends SessionError to the requester)
        let result = bob_mgr
            .handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities::empty(),
                    additional_data: 0,
                },
            )
            .await;

        // The second initiation returns Ok but handles the duplicate by sending SessionError
        assert!(
            result.is_ok(),
            "second session initiation should return Ok (error is handled internally)"
        );

        // Verify still only one session exists
        let active = bob_mgr.active_sessions();
        assert_eq!(active.len(), 1, "should still have exactly one active session");
        assert_eq!(bob_mgr.num_active_sessions(), 1);

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that pinging a session that does not exist returns `NonExistingSession`.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport.
    /// 2. `ping_session` is called with a completely random (non-existent) session ID.
    /// 3. The call returns an error matching `TransportSessionError::Manager(SessionManagerError::NonExistingSession)`.
    /// 4. `num_active_sessions` is 0, confirming no sessions were created.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_return_error_when_pinging_non_existent_session() -> anyhow::Result<()> {
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
        assert_eq!(mgr.num_active_sessions(), 0);
        let result = mgr.ping_session(&fake_session_id).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::NonExistingSession)
        ));

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that closing a session that does not exist returns `false` (no-op).
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport.
    /// 2. `close_session` is called with a random (non-existent) session ID.
    /// 3. The call returns `false`, indicating no session was closed.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_return_false_when_closing_non_existent_session() -> anyhow::Result<()> {
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
        assert_eq!(mgr.num_active_sessions(), 0);
        let result = mgr.close_session(&fake_session_id);

        assert!(!result, "closing non-existent session should return false");

        Ok(())
    }

    /// Verifies that updating the SURB balancer config for a non-existent session returns an error.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport.
    /// 2. `update_surb_balancer_config` is called with a random session ID.
    /// 3. The call returns an error (no `Ok` variant is expected).
    #[test_log::test(tokio::test)]
    async fn session_manager_should_return_error_when_updating_surb_config_for_non_existent_session()
    -> anyhow::Result<()> {
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
        let result = mgr.update_surb_balancer_config(&fake_session_id, SurbBalancerConfig::default());

        assert!(result.is_err());

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that retrieving the SURB balancer config for a non-existent session returns an error.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport.
    /// 2. `get_surb_balancer_config` is called with a random session ID.
    /// 3. The call returns an error matching `TransportSessionError::Manager(SessionManagerError::NonExistingSession)`.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_return_error_when_getting_surb_config_for_non_existent_session()
    -> anyhow::Result<()> {
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
        let result = mgr.get_surb_balancer_config(&fake_session_id);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::NonExistingSession)
        ));

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that retrieving SURB level estimates for a non-existent session returns an error.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport.
    /// 2. `get_surb_level_estimates` is called with a random session ID.
    /// 3. The call returns an error matching `TransportSessionError::Manager(SessionManagerError::NonExistingSession)`.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_return_error_when_getting_surb_estimates_for_non_existent_session()
    -> anyhow::Result<()> {
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
        let result = mgr.get_surb_level_estimates(&fake_session_id);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::NonExistingSession)
        ));

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies the `HoprStartProtocol::SessionError` match arm (line 689) in the
    /// `session_start_protocol_processor` task by calling `handle_session_error` directly.
    ///
    /// When a `SessionError` message is delivered while a `new_session` call is awaiting,
    /// `handle_session_error` retrieves the pending challenge from `session_initiations`,
    /// sends the error down the channel, and `new_session` propagates it as `Rejected`.
    #[test_log::test(tokio::test)]
    async fn handle_session_error_propagates_peer_rejection_to_pending_new_session() -> anyhow::Result<()> {
        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let mut transport = MockMsgSender::new();
        // new_session sends StartSession (succeeds), then waits for SessionEstablished.
        // We inject the error before it arrives.
        transport
            .expect_send_message()
            .returning(|_, _| futures::future::ok(()).boxed());

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        // Spawn new_session so it is blocked waiting for the session establishment response.
        let mgr_clone = mgr.clone();
        let peer_address: Address = (&ChainKeypair::random()).into();
        let handle = tokio::spawn(async move {
            mgr_clone
                .new_session(
                    peer_address,
                    SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    SessionClientConfig {
                        surb_management: None,
                        ..Default::default()
                    },
                )
                .await
        });

        // Give new_session time to insert the challenge into session_initiations.
        let challenge = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Some((ch, _)) = mgr.session_initiations.iter().next() {
                    break *ch;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("new_session did not insert a challenge into session_initiations")?;

        // Inject the SessionError with the matching challenge before SessionEstablished arrives.
        let error_type = StartErrorType {
            identifier: ErrorIdentifier::Challenge(challenge),
            reason: StartErrorReason::NoSlotsAvailable,
        };
        mgr.handle_session_error(error_type).await?;

        // new_session must propagate the error as Rejected.
        let result = handle.await?;
        match result {
            Ok(_session) => panic!("expected rejection error, got session"),
            Err(e) => {
                assert!(matches!(
                    e,
                    TransportSessionError::Rejected(StartErrorReason::NoSlotsAvailable)
                ));
            }
        }

        sender.close_channel();
        _handle.await??;
        Ok(())
    }

    /// Verifies that an incoming session initiation is rejected (handled internally) when the
    /// manager already has `maximum_sessions` active sessions.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is configured with `maximum_sessions: 1`.
    /// 2. `handle_incoming_session_initiation` is called with pseudonym `X1` — succeeds; one active session confirmed.
    /// 3. `handle_incoming_session_initiation` is called with pseudonym `X2` — the manager detects it is at capacity
    ///    and handles the conflict internally (sends `SessionError` to peer).
    /// 4. The call returns `Ok` (handled internally); `num_active_sessions` remains 1, with only `X1` present — `X2`
    ///    was rejected without creating a slot.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_reject_new_session_when_max_sessions_reached() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        // Create manager with max 1 session
        let cfg = SessionManagerConfig {
            maximum_sessions: 1,
            ..Default::default()
        };
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(cfg);

        let mut transport = MockMsgSender::new();
        transport
            .expect_send_message()
            .times(2)
            .returning(|_, _| futures::future::ok(()).boxed());

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        // First session - should succeed
        let pseudonym1 = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym1,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities::empty(),
                additional_data: 0,
            },
        )
        .await?;

        // Verify one session exists
        assert_eq!(mgr.active_sessions().len(), 1);
        assert_eq!(mgr.num_active_sessions(), 1);

        // Second session - should fail with TooManySessions
        let pseudonym2 = HoprPseudonym::random();
        let _result = mgr
            .handle_incoming_session_initiation(
                pseudonym2,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities::empty(),
                    additional_data: 0,
                },
            )
            .await;

        // The error is handled internally (sends SessionError), so result is Ok
        // But we can verify no new session was added
        assert_eq!(mgr.active_sessions().len(), 1);
        assert_eq!(mgr.num_active_sessions(), 1);

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// The node refuses a PIX Session it has no reconstructor memory left for, and does so before
    /// allocating anything.
    ///
    /// This is the bound `maximum_sessions` cannot express: a session slot is one slot whatever the
    /// peer offered, while the reconstructor state behind it is set by the peer's dimensions. Here
    /// `maximum_sessions` is wide open and the budget is exactly one Session's worth, so nothing but
    /// the budget can be doing the refusing.
    ///
    /// Refused, not queued: nothing later in establishment can give the memory back, and the peer is
    /// free to retry against another Exit.
    #[test_log::test(tokio::test)]
    async fn a_pix_session_over_the_live_cycle_budget_is_refused() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        let params = small_pix_params();
        let one_session = cycle_budget_for(&params, DEFAULT_SSAS_PER_SSA_REQUEST);

        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(SessionManagerConfig {
                maximum_sessions: 100,
                pix_config: IncomingSessionPixConfig {
                    quota_range: 0..=10_000_000_000_000,
                    // Room for exactly one Session at these dimensions.
                    max_live_cycle_bytes: one_session,
                    ..Default::default()
                },
                ..Default::default()
            });

        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: TEST_SURPLUS_SHARES,
        };
        let (pix_toolbox, _pix_events_rx) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mut transport = MockMsgSender::new();
        transport
            .expect_send_message()
            .returning(|_, _| futures::future::ok(()).boxed());
        let (sender, _handle) = mock_packet_planning(transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let offer = |pseudonym| {
            (
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse().unwrap())),
                    capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                    additional_data: small_pix_additional_data(),
                },
            )
        };

        let (p1, req1) = offer(HoprPseudonym::random());
        mgr.handle_incoming_session_initiation(p1, req1).await?;
        assert_eq!(1, mgr.num_active_sessions(), "the first PIX session must be admitted");
        assert_eq!(
            one_session,
            mgr.live_cycle_bytes.load(Ordering::Relaxed),
            "and must have charged exactly one session's worth"
        );

        // The budget is spent, so this one is refused — handled internally as a `SessionError`,
        // hence `Ok` with no new slot.
        let (p2, req2) = offer(HoprPseudonym::random());
        mgr.handle_incoming_session_initiation(p2, req2).await?;
        assert_eq!(
            1,
            mgr.num_active_sessions(),
            "a session over the live-cycle budget must not be admitted"
        );
        assert_eq!(
            one_session,
            mgr.live_cycle_bytes.load(Ordering::Relaxed),
            "and a refused session must not leave a reservation behind"
        );

        // Closing the admitted session returns its share, and the next request fits again.
        assert!(mgr.close_session(&p1));
        assert_eq!(
            0,
            mgr.live_cycle_bytes.load(Ordering::Relaxed),
            "closing a session must return its reservation"
        );

        let (p3, req3) = offer(HoprPseudonym::random());
        mgr.handle_incoming_session_initiation(p3, req3).await?;
        assert_eq!(
            1,
            mgr.num_active_sessions(),
            "the freed budget must admit the next session"
        );

        sender.close_channel();
        _handle.await??;
        Ok(())
    }

    /// A Session is charged for the dimensions its *peer* offered, not for this node's defaults.
    ///
    /// The Exit accepts a range of dimensions, and the reconstructor state behind them differs by
    /// more than an order of magnitude across it. Charging a flat figure would either refuse small
    /// Sessions that fit easily or admit large ones that do not.
    #[test]
    fn the_live_cycle_reservation_scales_with_the_offered_dimensions() -> anyhow::Result<()> {
        let small = PixParams::try_new(1024, 64, 16, LOCAL_PIX_SUITE)?;
        let large = PixParams::try_new(8192, 64, 16, LOCAL_PIX_SUITE)?;

        assert_eq!(
            8 * cycle_budget_for(&small, 1),
            cycle_budget_for(&large, 1),
            "eight times the polynomials must cost eight times the budget"
        );
        assert_eq!(
            3 * cycle_budget_for(&large, 1),
            cycle_budget_for(&large, 3),
            "and a batch of three must cost three times a batch of one"
        );

        // The pipelining factor is in there once, and only once: a batch may have one successor
        // outstanding, not one per member.
        assert_eq!(
            MAX_OVERLAPPING_BATCHES * hopr_protocol_pix::peak_cycle_bytes::<HoprPixSpec>(&large),
            cycle_budget_for(&large, 1)
        );

        // The clamp matches the one `SessionManager::new` applies, so a config that never went
        // through it cannot understate its own reservation.
        assert_eq!(cycle_budget_for(&large, 1), cycle_budget_for(&large, 0));
        assert_eq!(
            cycle_budget_for(&large, MAX_SSA_BATCH_SIZE),
            cycle_budget_for(&large, usize::MAX)
        );

        Ok(())
    }

    /// A non-PIX Session reserves nothing.
    ///
    /// `check_pix_params` hands back nominal parameters for a peer that offered no PIX at all, and
    /// charging on those would bill every plain Session for reconstructor state that will never
    /// exist — silently capping a node that does not run PIX.
    #[test_log::test(tokio::test)]
    async fn a_non_pix_session_does_not_touch_the_live_cycle_budget() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    // Not enough for any PIX session at all; a plain one must be unaffected.
                    max_live_cycle_bytes: 1,
                    ..Default::default()
                },
                ..Default::default()
            });

        let mut transport = MockMsgSender::new();
        transport
            .expect_send_message()
            .returning(|_, _| futures::future::ok(()).boxed());
        let (sender, _handle) = mock_packet_planning(transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(sender.clone(), new_session_tx, None)?;

        mgr.handle_incoming_session_initiation(
            HoprPseudonym::random(),
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities::empty(),
                additional_data: 0,
            },
        )
        .await?;

        assert_eq!(1, mgr.num_active_sessions());
        assert_eq!(0, mgr.live_cycle_bytes.load(Ordering::Relaxed));

        sender.close_channel();
        _handle.await??;
        Ok(())
    }

    /// Verifies the early `TooManySessions` return at the top of `new_session` (line 767).
    /// Unlike `session_manager_should_reject_new_session_when_max_sessions_reached`, which fills
    /// incoming slots and hits the slot-guard path at line 957, this test fills all `maximum_sessions`
    /// slots so that the `if self.cfg.maximum_sessions <= self.sessions.entry_count()` check fires
    /// before any message is sent.
    #[test_log::test(tokio::test)]
    async fn new_session_returns_too_many_sessions_when_cache_is_full() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        let cfg = SessionManagerConfig {
            maximum_sessions: 2,
            idle_timeout: Duration::from_secs(3600),
            ..Default::default()
        };
        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> = SessionManager::new(cfg);

        let mut transport = MockMsgSender::new();
        // Two incoming sessions: first sends SessionEstablished, second sends SessionError (no slots).
        transport
            .expect_send_message()
            .times(2)
            .returning(|_, _| futures::future::ok(()).boxed());

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        // Fill the cache with two incoming sessions (Exits).
        for i in 0..2 {
            let pseudonym = HoprPseudonym::random();
            mgr.handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE + i as u64,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities::empty(),
                    additional_data: 0,
                },
            )
            .await?;
        }
        assert_eq!(mgr.active_sessions().len(), 2);
        assert_eq!(mgr.num_active_sessions(), 2);

        // Third outgoing call hits the early return before sending anything.
        let result = mgr
            .new_session(
                Address::from(&ChainKeypair::random()),
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::TooManySessions)
        ));

        sender.close_channel();
        _handle.await??;
        Ok(())
    }

    /// Verifies that `session_initiations` is cleaned up when `new_session` fails to
    /// send the StartSession message (e.g. the underlying channel is closed).
    #[test_log::test(tokio::test)]
    async fn new_session_removes_challenge_on_send_failure() -> anyhow::Result<()> {
        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        // Create a channel whose receiver is dropped immediately.  When the mock
        // transport tries to `send` over this channel the call will return an error,
        // which propagates up through `send_via_msg_sender` as
        // `TransportSessionError::packet_sending`.
        let (tx, rx) = futures::channel::mpsc::unbounded();
        drop(rx);

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(tx, new_session_tx, None)?;
        assert!(mgr.is_started());

        // Verify that sending fails because the receiver is gone.
        let result = mgr
            .new_session(
                Address::from(&ChainKeypair::random()),
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await;

        assert!(result.is_err());
        assert_eq!(mgr.num_active_sessions(), 0);
        // The challenge must have been removed from `session_initiations` even
        // though the send failed.
        assert_eq!(
            mgr.session_initiations.entry_count(),
            0,
            "session_initiations was not cleaned up after send failure"
        );

        Ok(())
    }

    /// Verifies that `session_initiations` is cleaned up when the session initiation
    /// times out waiting for a response (neither `SessionEstablished` nor
    /// `SessionError` arrives).
    #[test_log::test(tokio::test)]
    async fn new_session_removes_challenge_on_timeout() -> anyhow::Result<()> {
        let cfg = SessionManagerConfig {
            initiation_timeout_base: Duration::from_millis(100),
            ..Default::default()
        };

        let alice_mgr = SessionManager::new(cfg);
        let bob_mgr = SessionManager::new(Default::default());

        let bob_peer: Address = (&ChainKeypair::random()).into();

        let mut alice_transport = MockMsgSender::new();
        let bob_transport = MockMsgSender::new();

        // Alice sends the StartSession message; Bob never responds.
        alice_transport
            .expect_send_message()
            .once()
            .returning(|_, _| futures::future::ok(()).boxed());

        let (alice_sender, _alice_handle) = mock_packet_planning(alice_transport);
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?;
        assert!(alice_mgr.is_started());

        let (bob_sender, _bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
        bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?;
        assert!(bob_mgr.is_started());

        // Record how many entries are in `session_initiations` before the call.
        assert_eq!(alice_mgr.session_initiations.entry_count(), 0);

        let result = alice_mgr
            .new_session(
                bob_peer,
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: None.into(),
                    pseudonym: None,
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await;

        assert!(matches!(result, Err(TransportSessionError::Timeout)));
        assert_eq!(alice_mgr.num_active_sessions(), 0);
        // The pending challenge must have been removed from `session_initiations`
        // after the timeout error propagated.
        assert_eq!(
            alice_mgr.session_initiations.entry_count(),
            0,
            "session_initiations was not cleaned up after timeout"
        );

        Ok(())
    }

    /// Verifies that dispatching data to a session that does not exist returns `UnknownData`.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport.
    /// 2. `dispatch_message` is called with a random pseudonym and an `ApplicationData` carrying the
    ///    `SESSION_APPLICATION_TAG` (a session-scoped tag).
    /// 3. The manager has no matching session, so the call returns `Err(TransportSessionError::UnknownData)`.
    /// 4. `num_active_sessions` is 0, confirming no session was implicitly created.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_return_unknown_data_error_when_dispatching_to_unknown_session() -> anyhow::Result<()>
    {
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        // Send data with session application tag but no session exists
        let pseudonym = HoprPseudonym::random();
        let result = mgr.dispatch_message(
            pseudonym,
            ApplicationDataIn {
                data: ApplicationData::new(SESSION_APPLICATION_TAG, b"test data")?,
                packet_info: Default::default(),
            },
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransportSessionError::UnknownData));
        assert_eq!(mgr.num_active_sessions(), 0);

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that closing an existing session returns `true` and removes the session from the manager.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is started with a mock transport that accepts one outbound message.
    /// 2. `handle_incoming_session_initiation` is called to create a session — one active session confirmed.
    /// 3. `close_session` is called with the session's pseudonym — returns `true`.
    /// 4. `num_active_sessions` is 0, confirming the session was fully removed.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_return_true_when_closing_existing_session() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let mut transport = MockMsgSender::new();
        transport
            .expect_send_message()
            .once()
            .returning(|_, _| futures::future::ok(()).boxed());

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        // Create a session
        let pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities::empty(),
                additional_data: 0,
            },
        )
        .await?;

        // Verify session exists
        assert_eq!(mgr.active_sessions().len(), 1);
        assert_eq!(mgr.num_active_sessions(), 1);

        // Close the session - should return true
        let result = mgr.close_session(&pseudonym);
        assert!(result, "closing existing session should return true");

        // Verify session is closed
        assert_eq!(mgr.active_sessions().len(), 0);
        assert_eq!(mgr.num_active_sessions(), 0);

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that a `KeepAlive` message with the `BalancerState` flag updates the session's
    /// SURB buffer level in the manager.
    ///
    /// ## Steps
    /// 1. A session slot is manually inserted into Alice's manager with a known `SurbBalancerConfig` and an initial
    ///    buffer level of 100.
    /// 2. A `KeepAlive` message with `KeepAliveFlag::BalancerState` and `additional_data: 200` is constructed and
    ///    dispatched to Alice's manager via `dispatch_message`.
    /// 3. The manager processes the keep-alive asynchronously; the test polls until the slot's `buffer_level` reaches
    ///    200 (with a 1-second timeout).
    /// 4. The buffer level is confirmed to be 200, proving the `BalancerState` flag updated it.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_update_buffer_level_on_keep_alive_with_balancer_state_flag() -> anyhow::Result<()> {
        use std::sync::atomic::Ordering;

        let alice_pseudonym = HoprPseudonym::random();
        let session_id = alice_pseudonym;
        let initial_buffer_level = 100u64;
        let new_buffer_level = 200u64;

        let balancer_cfg = SurbBalancerConfig {
            target_surb_buffer_size: 1000,
            max_surbs_per_sec: 100,
            ..Default::default()
        };

        let alice_mgr =
            SessionManager::<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>::new(Default::default());

        let (new_session_tx, _) = futures::channel::mpsc::channel(1024);
        let (mock_sender, _) = futures::channel::mpsc::unbounded();
        let _ahs = alice_mgr.start(mock_sender, new_session_tx, None)?;
        assert!(alice_mgr.is_started());

        let (dummy_tx, _) = crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(SESSION_FORWARD_CAPACITY);
        let peer_address: Address = (&ChainKeypair::random()).into();
        alice_mgr.sessions.insert(
            session_id,
            SessionSlot {
                session_tx: dummy_tx,
                routing_opts: DestinationRouting::Forward {
                    destination: Box::new(peer_address.into()),
                    pseudonym: Some(alice_pseudonym),
                    forward_options: RoutingOptions::Hops(hopr_api::types::primitive::bounded::BoundedSize::MIN),
                    return_options: RoutingOptions::Hops(hopr_api::types::primitive::bounded::BoundedSize::MIN).into(),
                },
                abort_handles: Default::default(),
                surb_mgmt: Arc::new(BalancerStateValues::from(balancer_cfg)),
                surb_estimator: Default::default(),
                current_ssa_state: Default::default(),
                pix_supervisor: Default::default(),
                pix_egress_gate: Default::default(),
                returned_packets: Default::default(),
                cycle_budget: None,
            },
        );

        // Set initial buffer level
        let session_slot = alice_mgr.sessions.get(&session_id).unwrap();
        session_slot
            .surb_mgmt
            .buffer_level
            .store(initial_buffer_level, Ordering::Relaxed);
        drop(session_slot);

        // Verify initial buffer level
        let session_slot = alice_mgr.sessions.get(&session_id).unwrap();
        assert_eq!(session_slot.surb_mgmt.buffer_level(), initial_buffer_level);
        drop(session_slot);

        // Create keep-alive message with BalancerState flag
        let ka = KeepAliveMessage::<SessionId> {
            session_id,
            flags: KeepAliveFlag::BalancerState.into(),
            additional_data: new_buffer_level,
        };
        let app_data: ApplicationData = HoprStartProtocol::KeepAlive(ka).try_into()?;
        let app_data_in = ApplicationDataIn {
            data: app_data,
            packet_info: Default::default(),
        };

        // Dispatch the keep-alive message
        alice_mgr.dispatch_message(alice_pseudonym, app_data_in)?;

        // Poll until the background task has processed the keep-alive
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Some(slot) = alice_mgr.sessions.get(&session_id)
                    && slot.surb_mgmt.buffer_level() == new_buffer_level
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("keep-alive BalancerState update timed out")?;

        // Verify buffer level was updated
        let session_slot = alice_mgr.sessions.get(&session_id).unwrap();
        assert_eq!(
            session_slot.surb_mgmt.buffer_level(),
            new_buffer_level,
            "buffer level should be updated via keep-alive with BalancerState flag"
        );

        Ok(())
    }

    /// Verifies that a `KeepAlive` message with the `BalancerTarget` flag updates the session's
    /// target SURB buffer size in the manager.
    ///
    /// ## Steps
    /// 1. A session slot is manually inserted into Alice's manager with a known `SurbBalancerConfig` and
    ///    `target_surb_buffer_size: 1000`.
    /// 2. A `KeepAlive` message with `KeepAliveFlag::BalancerTarget` and `additional_data: 2000` is constructed and
    ///    dispatched via `dispatch_message`.
    /// 3. The manager processes the keep-alive asynchronously; the test polls until the slot's
    ///    `target_surb_buffer_size` reaches 2000 (with a 1-second timeout).
    /// 4. The target is confirmed to be 2000, proving the `BalancerTarget` flag updated it.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_update_target_on_keep_alive_with_balancer_target_flag() -> anyhow::Result<()> {
        use std::sync::atomic::Ordering;

        let alice_pseudonym = HoprPseudonym::random();
        let session_id = alice_pseudonym;
        let initial_target = 1000u64;
        let new_target = 2000u64;

        let balancer_cfg = SurbBalancerConfig {
            target_surb_buffer_size: initial_target,
            max_surbs_per_sec: 100,
            ..Default::default()
        };

        let alice_mgr =
            SessionManager::<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>::new(Default::default());

        let (new_session_tx, _) = futures::channel::mpsc::channel(1024);
        let (mock_sender, _) = futures::channel::mpsc::unbounded();
        let _ahs = alice_mgr.start(mock_sender, new_session_tx, None)?;
        assert!(alice_mgr.is_started());

        let (dummy_tx, _) = crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(SESSION_FORWARD_CAPACITY);
        alice_mgr.sessions.insert(
            session_id,
            SessionSlot {
                session_tx: dummy_tx,
                routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(alice_pseudonym)),
                abort_handles: Default::default(),
                surb_mgmt: Arc::new(BalancerStateValues::from(balancer_cfg)),
                surb_estimator: Default::default(),
                current_ssa_state: Default::default(),
                pix_supervisor: Default::default(),
                pix_egress_gate: Default::default(),
                returned_packets: Default::default(),
                cycle_budget: None,
            },
        );

        // Verify initial target
        let session_slot = alice_mgr.sessions.get(&session_id).unwrap();
        assert_eq!(
            session_slot.surb_mgmt.controller_bounds().target(),
            initial_target,
            "initial target should be set"
        );
        drop(session_slot);

        // Create keep-alive message with BalancerTarget flag
        let ka = KeepAliveMessage::<SessionId> {
            session_id,
            flags: KeepAliveFlag::BalancerTarget.into(),
            additional_data: new_target,
        };
        let app_data: ApplicationData = HoprStartProtocol::KeepAlive(ka).try_into()?;
        let app_data_in = ApplicationDataIn {
            data: app_data,
            packet_info: Default::default(),
        };

        // Dispatch the keep-alive message
        alice_mgr.dispatch_message(alice_pseudonym, app_data_in)?;

        // Poll until the background task has processed the keep-alive
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Some(slot) = alice_mgr.sessions.get(&session_id)
                    && slot.surb_mgmt.target_surb_buffer_size.load(Ordering::Relaxed) == new_target
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("keep-alive BalancerTarget update timed out")?;

        // Verify target was updated
        let session_slot = alice_mgr.sessions.get(&session_id).unwrap();
        assert_eq!(
            session_slot.surb_mgmt.target_surb_buffer_size.load(Ordering::Relaxed),
            new_target,
            "target buffer size should be updated via keep-alive with BalancerTarget flag"
        );

        Ok(())
    }

    /// Verifies that a session is evicted after the `idle_timeout` fires, without needing an explicit close.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is configured with `maximum_sessions: 1` and `idle_timeout: 100ms`.
    /// 2. `handle_incoming_session_initiation` creates one session — confirmed active.
    /// 3. The test sleeps 200ms (well past the 100ms timeout), then calls `sessions.run_pending_tasks()` to drive the
    ///    eviction timer.
    /// 4. `active_sessions` is empty, confirming the idle session was cleaned up without an explicit close call.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_evict_idle_session_and_call_close_callback() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        let cfg = SessionManagerConfig {
            maximum_sessions: 1,
            idle_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(cfg);

        let mut transport = MockMsgSender::new();
        transport
            .expect_send_message()
            .times(1)
            .returning(|_, _| futures::future::ok(()).boxed());

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        // Create first session
        let pseudonym1 = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym1,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities::empty(),
                additional_data: 0,
            },
        )
        .await?;

        // Verify first session exists
        assert_eq!(mgr.active_sessions().len(), 1);

        // Wait for the session to expire (idle_timeout = 100ms)
        tokio::time::sleep(Duration::from_millis(200)).await;
        mgr.sessions.run_pending_tasks();

        // Verify session was evicted (cache should be empty now)
        assert_eq!(
            mgr.active_sessions().len(),
            0,
            "idle session should be evicted after timeout"
        );

        Ok(())
    }

    /// Verifies that a second incoming session initiation is rejected (not evicted) when the manager
    /// is at `maximum_sessions` capacity with a long `idle_timeout`.
    ///
    /// ## Steps
    /// 1. A `SessionManager` is configured with `maximum_sessions: 1` and `idle_timeout: 3600s` (long enough that
    ///    eviction will not fire during the test).
    /// 2. `handle_incoming_session_initiation` creates session `X1` — confirmed active.
    /// 3. `handle_incoming_session_initiation` is called for session `X2` — the manager detects capacity is reached and
    ///    rejects the initiation internally (sends `SessionError`).
    /// 4. `active_sessions` still contains only `X1`; the first session was not evicted to make room.
    #[test_log::test(tokio::test)]
    async fn session_manager_should_reject_new_session_when_max_sessions_reached_no_eviction() -> anyhow::Result<()> {
        use hopr_utils::network_types::prelude::SealedHost;

        // Create manager with max 1 session
        let cfg = SessionManagerConfig {
            maximum_sessions: 1,
            idle_timeout: Duration::from_secs(3600), // Long timeout so eviction doesn't happen
            ..Default::default()
        };
        let mgr: SessionManager<futures::channel::mpsc::UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(cfg);

        let mut transport = MockMsgSender::new();
        transport
            .expect_send_message()
            .times(2)
            .returning(|_, _| futures::future::ok(()).boxed());

        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        // Create first session
        let pseudonym1 = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym1,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities::empty(),
                additional_data: 0,
            },
        )
        .await?;

        // Verify first session exists
        assert_eq!(mgr.active_sessions().len(), 1);

        // Try to create second session - should be rejected (not evicted)
        let pseudonym2 = HoprPseudonym::random();
        let _result = mgr
            .handle_incoming_session_initiation(
                pseudonym2,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities::empty(),
                    additional_data: 0,
                },
            )
            .await;

        // Should still have exactly 1 session (the first one)
        assert_eq!(
            mgr.active_sessions().len(),
            1,
            "should still have exactly one session - second session should be rejected"
        );

        // The active session should be the first one (second was rejected)
        assert!(
            mgr.active_sessions().contains(&pseudonym1),
            "the first session should still be active"
        );

        // Cleanup: close sender and await handle
        sender.close_channel();
        _handle.await??;

        Ok(())
    }

    /// Verifies that `new_session` rejects `UsePIX` when the return path has 0 intermediate hops.
    ///
    /// PIX shares are encrypted with the first relayer's ticket-challenge solution and carried
    /// in return-path SURBs. With 0 intermediate hops (a direct Exit→Entry SURB), there is no
    /// relayer to provide the challenge solution, so shares are never embedded — the ongoing
    /// PIX share delivery mechanism is dead and the Exit's quota is never replenished.
    #[test_log::test(tokio::test)]
    async fn new_session_rejects_usepix_with_zero_return_hops() -> anyhow::Result<()> {
        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let mut transport = MockMsgSender::new();
        // The error happens before any message is sent, so expect_send_message should NOT fire.
        transport.expect_send_message().times(0);

        let (sender, _handle) = mock_packet_planning(transport);
        let (new_session_tx, _) = futures::channel::mpsc::channel(1);
        mgr.start(sender.clone(), new_session_tx, None)?;
        assert!(mgr.is_started());

        let result = mgr
            .new_session(
                Address::from(&ChainKeypair::random()),
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: Capability::UsePIX.into(),
                    surb_management: None,
                    pix_ssa_quota: Some(PixParams::try_new(2, 2, TEST_SURPLUS_SHARES, LOCAL_PIX_SUITE)?),
                    forward_path_options: RoutingOptions::Hops(1.try_into()?),
                    return_path_options: RoutingOptions::Hops(0.try_into()?),
                    ..Default::default()
                },
            )
            .await;

        let err = result.unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("UsePIX requires at least 1 intermediate hop on the return path"),
            "expected return-path guard error, got: {msg}"
        );

        assert_eq!(mgr.num_active_sessions(), 0);
        // No challenge slot was consumed because the validations run before insert_into_next_slot.
        assert_eq!(
            mgr.session_initiations.entry_count(),
            0,
            "session_initiations must remain empty when UsePIX is rejected"
        );

        sender.close_channel();
        _handle.await??;
        Ok(())
    }

    /// Verifies that `new_session` rejects UsePIX when the requested
    /// `pix_ssa_quota` doesn't match the installed `SsaShareGenerator`'s configured dimensions.
    ///
    /// ## Steps
    /// 1. Create a `PixToolbox` with a generator configured for `(polys=5, shares=3, surplus=5)`.
    /// 2. Start the manager with that toolbox installed.
    /// 3. Call `new_session` requesting `pix_ssa_quota: Some(PixParams::try_new(10, 10, 5))` — every value is within
    ///    protocol bounds but mismatches the generator.
    /// 4. Assert the error identifies the mismatch.
    /// 5. Assert no challenge slot was consumed (validation runs before slot reservation).
    #[test_log::test(tokio::test)]
    async fn new_session_rejects_usepix_when_quota_mismatches_generator() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};

        // The surplus is at the threshold rather than above it: `surplus_must_not_exceed_threshold`
        // began rejecting the latter when the surplus became a billed ratio, and this fixture was
        // left behind at 5-against-3, which made the generator itself unconstructible.
        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 5,
            threshold: 3,
            surplus_shares: 3,
        };
        let (pix_toolbox, _) = PixToolbox::new(
            Arc::new(SsaShareGenerator::new(ssa_gen_config)),
            Arc::new(SsaReconstructor::new(SsaReconstructorConfig::default())),
        );

        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(Default::default());

        let mut transport = MockMsgSender::new();
        // The error happens before any message is sent, so expect NO sends.
        transport.expect_send_message().times(0);

        let (sender, _handle) = mock_packet_planning(transport);
        let (new_session_tx, _) = futures::channel::mpsc::channel(1);
        mgr.start(sender.clone(), new_session_tx, Some(pix_toolbox))?;
        assert!(mgr.is_started());

        let result = mgr
            .new_session(
                Address::from(&ChainKeypair::random()),
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: Capability::UsePIX.into(),
                    surb_management: None,
                    // Every dimension passes protocol bounds but polys=10 != generator's 5
                    pix_ssa_quota: Some(PixParams::try_new(10, 10, 5, LOCAL_PIX_SUITE)?),
                    forward_path_options: RoutingOptions::Hops(1.try_into()?),
                    return_path_options: RoutingOptions::Hops(2.try_into()?),
                    ..Default::default()
                },
            )
            .await;

        let err = result.unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("do not match installed generator"),
            "expected generator mismatch error, got: {msg}"
        );

        // A surplus-only mismatch must be rejected too. It is the value with no other consumer —
        // nothing downstream would notice it being wrong — so if the comparison ever narrows back
        // to the two priced dimensions, this is what catches it.
        let result = mgr
            .new_session(
                Address::from(&ChainKeypair::random()),
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: Capability::UsePIX.into(),
                    surb_management: None,
                    pix_ssa_quota: Some(PixParams::try_new(
                        ssa_gen_config.polynomials_per_ssa,
                        ssa_gen_config.threshold,
                        ssa_gen_config.surplus_shares + 1,
                        LOCAL_PIX_SUITE,
                    )?),
                    forward_path_options: RoutingOptions::Hops(1.try_into()?),
                    return_path_options: RoutingOptions::Hops(2.try_into()?),
                    ..Default::default()
                },
            )
            .await;
        assert!(
            format!("{:?}", result.unwrap_err()).contains("do not match installed generator"),
            "a surplus-only mismatch must be rejected"
        );

        assert_eq!(mgr.num_active_sessions(), 0);
        // No challenge slot was consumed because the validations run before insert_into_next_slot.
        assert_eq!(
            mgr.session_initiations.entry_count(),
            0,
            "session_initiations must remain empty when generator mismatch is rejected"
        );

        sender.close_channel();
        _handle.await??;
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // PIX protocol tests
    // ---------------------------------------------------------------------------

    /// Supervisor durations a programmatic caller supplies must be clamped, not trusted.
    ///
    /// `validate_pix_supervision` rejects these, but nothing in this crate calls `validate` — a
    /// `SessionManagerConfig` assembled in code reaches the supervisor exactly as written. And the
    /// failure is silent in the worst direction: `Instant::checked_add` returns `None` for a duration
    /// the monotonic clock cannot represent, and every phase reads an absent deadline as *no
    /// deadline*, so the over-large value disables the rule rather than relaxing it.
    ///
    /// The two batch-scaled deadlines are clamped by what is actually armed, so a batch of `n` leaves
    /// each of them at a cap `n` times smaller.
    #[test]
    fn programmatic_supervisor_durations_are_clamped_to_a_representable_deadline() {
        const BATCH: usize = 4;
        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    supervision: SupervisorConfig {
                        ssas_per_request: BATCH,
                        max_ssa_delivery_time: Duration::MAX,
                        max_deposit_wait: Duration::MAX,
                        max_recovery_idle: Duration::MAX,
                        max_recovery_time: Duration::MAX,
                        tombstone_retention_window: Duration::MAX,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

        let sup = &mgr.cfg.pix_config.supervision;
        let cap = crate::supervision::MAX_SUPERVISOR_DURATION;
        let per_cycle_cap = cap / BATCH as u32;

        assert_eq!(per_cycle_cap, sup.max_ssa_delivery_time);
        assert_eq!(per_cycle_cap, sup.max_deposit_wait);
        assert_eq!(cap, sup.max_recovery_time);
        assert_eq!(cap, sup.tombstone_retention_window);

        // `max_recovery_idle` is bounded by something tighter than the clock: it must stay strictly
        // under the reconstructor's `unused_verifier_lifetime`, or the supervisor waits on a cycle
        // whose state was reclaimed long before. Normalized against the default reconstructor, which
        // is the one `SessionManager::new` can see; `start` re-checks against the installed one.
        let lifetime = hopr_protocol_pix::SsaReconstructorConfig::default().unused_verifier_lifetime;
        assert!(
            sup.max_recovery_idle < lifetime,
            "clamped max_recovery_idle ({:?}) must stay under the reconstructor lifetime ({lifetime:?})",
            sup.max_recovery_idle
        );
        crate::supervision::validate_pix_supervision(sup, &hopr_protocol_pix::SsaReconstructorConfig::default())
            .expect("a normalized config must satisfy every cross-component invariant, not only representability");

        // Which is exactly the condition that makes every deadline representable.
        let now = std::time::Instant::now();
        for dur in [
            crate::supervision::scaled_deadline(sup.max_ssa_delivery_time, sup.ssas_per_request),
            crate::supervision::scaled_deadline(sup.max_deposit_wait, sup.ssas_per_request),
            sup.max_recovery_idle,
            sup.max_recovery_time,
            sup.tombstone_retention_window,
        ] {
            assert!(
                now.checked_add(dur).is_some(),
                "a clamped duration ({dur:?}) must still produce a deadline"
            );
        }
    }

    /// Normalizing against the default reconstructor is not enough when a caller installs a
    /// different one.
    ///
    /// `SessionManager::new` cannot see the reconstructor — it arrives with the toolbox at `start` —
    /// so its clamp can only assume the defaults. A caller pairing a shorter
    /// `unused_verifier_lifetime` with a supervisor config that was valid against the defaults gets
    /// a supervisor that outlives the state it is waiting on. That must be an error rather than a
    /// silent clamp: at `start` both halves were chosen deliberately, and overriding one of them
    /// would be answering a question the caller did not ask.
    #[test]
    fn start_rejects_a_supervisor_config_the_installed_reconstructor_cannot_support() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};

        // Valid against the defaults — 10 min sits well under the default 30 min lifetime — and
        // therefore untouched by the constructor's clamp.
        let cfg = SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                supervision: SupervisorConfig {
                    max_recovery_idle: Duration::from_secs(600),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        crate::supervision::validate_pix_supervision(&cfg.pix_config.supervision, &SsaReconstructorConfig::default())
            .expect("the fixture must be valid against the default reconstructor");

        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> = SessionManager::new(cfg);

        // But the reconstructor actually installed reclaims a cycle after 60 s.
        let (pix_toolbox, _pix_events) = PixToolbox::new(
            SsaShareGenerator::new(SsaGeneratorConfig::default()).into(),
            SsaReconstructor::new(SsaReconstructorConfig {
                unused_verifier_lifetime: Duration::from_secs(60),
                ..Default::default()
            })
            .into(),
        );

        let (tx, _rx) = futures::channel::mpsc::unbounded();
        let (new_session_tx, _new_session_rx) = futures::channel::mpsc::channel(1);
        let started = mgr.start(tx, new_session_tx, Some(pix_toolbox));

        assert!(
            matches!(started, Err(TransportSessionError::InvalidConfig(_))),
            "starting with a reconstructor that cannot support the supervisor config must fail, got {started:?}"
        );

        Ok(())
    }

    /// A rejected PIX configuration must not consume the manager's one-shot start state.
    ///
    /// Validation is fallible and happens before any worker is spawned, so callers must be able to
    /// correct the toolbox (or start without PIX) and retry the same manager. Otherwise `start`
    /// reports an ordinary configuration error while leaving `is_started() == false`, but the
    /// already-filled message-sender lock makes every retry fail with `AlreadyStarted`.
    ///
    /// On a runtime because the retry now gets far enough to spawn the manager's workers, which is
    /// the whole point — before the fix it returned `AlreadyStarted` before reaching them.
    #[test_log::test(tokio::test)]
    async fn rejected_pix_start_does_not_poison_a_retry() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};

        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    supervision: SupervisorConfig {
                        max_recovery_idle: Duration::from_secs(600),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

        let (invalid_pix, _pix_events) = PixToolbox::new(
            SsaShareGenerator::new(SsaGeneratorConfig::default()).into(),
            SsaReconstructor::new(SsaReconstructorConfig {
                unused_verifier_lifetime: Duration::from_secs(60),
                ..Default::default()
            })
            .into(),
        );
        let (first_tx, _first_rx) = futures::channel::mpsc::unbounded();
        let (first_notifier, _first_notifications) = futures::channel::mpsc::channel(1);
        assert!(matches!(
            mgr.start(first_tx, first_notifier, Some(invalid_pix)),
            Err(TransportSessionError::InvalidConfig(_))
        ));
        assert!(
            !mgr.is_started(),
            "a rejected start must not report the manager as running"
        );

        let (retry_tx, _retry_rx) = futures::channel::mpsc::unbounded();
        let (retry_notifier, _retry_notifications) = futures::channel::mpsc::channel(1);
        let retry = mgr.start(retry_tx, retry_notifier, None);
        assert!(
            retry.is_ok(),
            "a configuration error must leave the manager retryable, got {retry:?}"
        );

        Ok(())
    }

    /// Making every duration representable is not sufficient if normalization leaves a supervisor
    /// waiting after its paired reconstructor has already discarded the cycle. A programmatic
    /// caller using the default reconstructor must receive a configuration that still satisfies the
    /// same cross-component lifetime invariants as a file-loaded configuration.
    #[test]
    fn programmatic_clamping_preserves_reconstructor_lifetime_invariants() {
        let mgr: SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>> =
            SessionManager::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    supervision: SupervisorConfig {
                        max_recovery_idle: Duration::MAX,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

        crate::supervision::validate_pix_supervision(
            &mgr.cfg.pix_config.supervision,
            &SsaReconstructorConfig::default(),
        )
        .expect(
            "normalizing a programmatic config must not leave recovery-idle beyond the reconstructor's state lifetime",
        );
    }

    /// Verifies that an incoming session initiation with a PIX quota outside the acceptable range
    /// is rejected with `StartErrorReason::UnacceptablePixParams`.
    ///
    /// ## Steps
    /// 1. Bob's manager is configured with `pix_config.quota_range: 0..=2048*1024*1024` (accepts quotas up to ~2 GiB).
    /// 2. The test encodes `additional_data` at the maximum legal dimensions, which translates to a quota of ~4 GiB —
    ///    outside the allowed range, while each individual dimension is in range.
    /// 3. `handle_incoming_session_initiation` is called with `Capability::UsePIX` and the out-of-range quota.
    /// 4. Bob's manager sends a `SessionError` back to the peer with reason `UnacceptablePixParams`.
    /// 5. The test receives the error on a one-shot channel and asserts `err.reason == UnacceptablePixParams` and
    ///    `err.challenge == MIN_CHALLENGE`.
    /// 6. `num_active_sessions` is 0, confirming no session slot was created.
    #[test_log::test(tokio::test)]
    async fn incoming_session_with_unacceptable_pix_quota_is_rejected() -> anyhow::Result<()> {
        use std::sync::Arc;

        use hopr_protocol_start::{StartErrorReason, StartInitiation};
        use tokio::sync::oneshot;

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=2048 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(std::sync::Mutex::new(Some(tx)));

        bob_transport.expect_send_message().returning(move |_, data| {
            let tx = tx.clone();
            Box::pin(async move {
                if let Ok(HoprStartProtocol::SessionError(err)) =
                    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
                    && let Some(tx) = tx.lock().unwrap().take()
                {
                    let _ = tx.send(err);
                }
                Ok(())
            })
        });

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, _) = futures::channel::mpsc::channel(1);
        mgr.start(bob_sender.clone(), new_session_tx, None)?;

        let alice_pseudonym = HoprPseudonym::random();

        // The largest dimensions the protocol admits: quota = 16192 * 255 * 1038 ≈ 3.99 GiB, well
        // outside the acceptable range of 0..=2048*1024*1024. Both dimensions are individually
        // legal, so this exercises the quota check rather than the range check that precedes it.
        let additional_data = pix_additional_data(MAX_POLYS_PER_SSA, MAX_POLY_THRESHOLD, 0);

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data,
            },
        )
        .await?;

        let err = rx.await.context("send_message was never called")?;
        assert_eq!(err.reason, StartErrorReason::UnacceptablePixParams);
        assert_eq!(err.identifier, ErrorIdentifier::Challenge(MIN_CHALLENGE));
        assert_eq!(mgr.num_active_sessions(), 0);

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Verifies that an incoming session initiation that does not declare `UsePIX` capability is
    /// rejected when PIX is enforced on the responder.
    ///
    /// ## Steps
    /// 1. Bob's manager is configured with `pix_config.enforce_pix: true`, requiring all incoming sessions to opt into
    ///    PIX.
    /// 2. The incoming initiation carries `Capability::Segmentation` only (no `UsePIX`).
    /// 3. `handle_incoming_session_initiation` is called; Bob's manager detects the missing `UsePIX` capability and
    ///    sends a `SessionError` with `UnacceptablePixParams`.
    /// 4. The test receives the error and asserts `err.reason == UnacceptablePixParams`.
    /// 5. `num_active_sessions` is 0, confirming no session slot was created.
    #[test_log::test(tokio::test)]
    async fn incoming_session_without_usepix_is_rejected_when_pix_enforced() -> anyhow::Result<()> {
        use std::sync::Arc;

        use hopr_protocol_start::{StartErrorReason, StartInitiation};
        use tokio::sync::oneshot;

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                enforce_pix: true,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(std::sync::Mutex::new(Some(tx)));

        bob_transport.expect_send_message().returning(move |_, data| {
            let tx = tx.clone();
            Box::pin(async move {
                if let Ok(HoprStartProtocol::SessionError(err)) =
                    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
                    && let Some(tx) = tx.lock().unwrap().take()
                {
                    let _ = tx.send(err);
                }
                Ok(())
            })
        });

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, _) = futures::channel::mpsc::channel(1);
        mgr.start(bob_sender.clone(), new_session_tx, None)?;

        let alice_pseudonym = HoprPseudonym::random();

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::Segmentation.into()),
                additional_data: 0,
            },
        )
        .await?;

        let err = rx.await.context("send_message was never called")?;
        assert_eq!(err.reason, StartErrorReason::UnacceptablePixParams);
        assert_eq!(err.identifier, ErrorIdentifier::Challenge(MIN_CHALLENGE));
        assert_eq!(mgr.num_active_sessions(), 0);

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// A node with no `PixToolbox` must refuse an incoming session that asks for `UsePIX`, rather
    /// than establish one it cannot run the PIX state machine for.
    ///
    /// This is the guard at the top of `handle_incoming_session_initiation`, and it was untested:
    /// the integration test named for it never negotiated PIX at all, so the absent toolbox was not
    /// the operative cause of anything it observed.
    ///
    /// The offered dimensions have to be ones `check_pix_params` *accepts*, and the test asserts
    /// that before exercising the handler. The fallthrough from a rejected `check_pix_params` emits
    /// the identical `StartErrorReason::UnacceptablePixParams` under the identical
    /// `ErrorIdentifier::Challenge` and creates no session either, so with unacceptable dimensions
    /// nothing here could tell the two refusals apart — the guard could be deleted outright and
    /// this would still pass.
    #[test_log::test(tokio::test)]
    async fn incoming_usepix_session_is_rejected_when_no_pix_toolbox_is_installed() -> anyhow::Result<()> {
        use std::sync::Arc;

        use hopr_protocol_start::{StartErrorReason, StartInitiation};
        use tokio::sync::oneshot;

        // PIX not enforced, and the default `quota_range` ends exactly on the quota the default
        // dimensions imply, so the offer below sits inside it.
        let mgr = SessionManager::new(SessionManagerConfig::default());

        let mut bob_transport = MockMsgSender::new();
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(std::sync::Mutex::new(Some(tx)));

        bob_transport.expect_send_message().returning(move |_, data| {
            let tx = tx.clone();
            Box::pin(async move {
                if let Ok(HoprStartProtocol::SessionError(err)) =
                    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
                    && let Some(tx) = tx.lock().unwrap().take()
                {
                    let _ = tx.send(err);
                }
                Ok(())
            })
        });

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, _) = futures::channel::mpsc::channel(1);
        // No toolbox — the third argument is what a relay that does not participate in PIX gets.
        mgr.start(bob_sender.clone(), new_session_tx, None)?;

        let req = StartInitiation {
            challenge: MIN_CHALLENGE,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            capabilities: HoprSessionCapabilities(Capability::Segmentation | Capability::UsePIX),
            additional_data: DEFAULT_PIX_PARAMS.into_additional_data(0),
        };

        // What makes the missing toolbox the sole remaining cause of the refusal below.
        assert!(
            mgr.check_pix_params(&req).is_some(),
            "the offered dimensions must be acceptable, or the refusal cannot be attributed to the guard"
        );

        mgr.handle_incoming_session_initiation(HoprPseudonym::random(), req)
            .await?;

        let err = rx.await.context("send_message was never called")?;
        assert_eq!(err.reason, StartErrorReason::UnacceptablePixParams);
        assert_eq!(err.identifier, ErrorIdentifier::Challenge(MIN_CHALLENGE));
        assert_eq!(mgr.num_active_sessions(), 0, "no slot may be created for a refusal");

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Verifies that the exit/responder (Bob) rejects an `SsaCommit` for a session that has no PIX
    /// state — i.e., the SSA commit is delivered with a session ID that Bob does not hold.
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` and a PIX quota config. Alice's session initiation is processed
    ///    normally via `handle_incoming_session_initiation`, establishing a session with PIX state.
    /// 2. `handle_ssa_commit` is called with a completely different (random) session ID — one that Bob's manager does
    ///    not have.
    /// 3. The call returns an error matching `TransportSessionError::Manager(SessionManagerError::NonExistingSession)`,
    ///    confirming the exit rejects commits for unknown sessions.
    #[test_log::test(tokio::test)]
    async fn exit_rejects_ssa_commit_when_session_has_no_pix_state() -> anyhow::Result<()> {
        use std::collections::HashMap;

        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        };
        let ssa_rec_config = SsaReconstructorConfig::default();

        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(ssa_rec_config).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        // handle_incoming_session_initiation sends SessionEstablished + SsaRequest.
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        let result = mgr
            .handle_ssa_commit(
                HoprPseudonym::random(),
                SsaClientCommitmentMessage {
                    session_id: alice_pseudonym,
                    ssa_index: SsaIndex::MIN,
                    coefficient_index: 0,
                    commitment_proof: None,
                    coefficient_commitments: HashMap::new(),
                },
            )
            .await;

        bob_sender.close_channel();
        bob_handle.await??;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::NonExistingSession)
        ));

        Ok(())
    }

    /// Verifies that a session is closed on the very first `UnverifiableShares` PIX event.
    ///
    /// An event now means a whole polynomial failed to open its commitment, which already dooms
    /// the cycle — so [`SupervisorConfig::max_unverifiable_shares_per_ssa`] is 0 and there is
    /// nothing to tolerate. See that field for the reasoning.
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` and a PIX quota config. Alice's session initiation is processed
    ///    via `handle_incoming_session_initiation`.
    /// 2. One `UnverifiableShares` event is dispatched for the session's `SsaId`.
    /// 3. The session is closed: `active_sessions` is empty and `num_active_sessions` is 0, confirming the supervisor
    ///    closed it.
    #[test_log::test(tokio::test)]
    async fn session_is_closed_on_the_first_unverifiable_share() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        };

        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        // handle_incoming_session_initiation sends SessionEstablished + SsaRequest.
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        assert_eq!(mgr.num_active_sessions(), 1, "the session must start out open");

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShares {
            ssa_id,
            observed_total: 1,
        })
        .await?;

        // The supervisor decides, and its `Close` reaches the driver over a channel, so the close
        // is observed rather than assumed.
        tokio::time::timeout(Duration::from_secs(1), async {
            while mgr.num_active_sessions() > 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("session must be closed by the first unverifiable share")?;

        assert!(mgr.active_sessions().is_empty());

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    // Four tests lived here, all asserting how many `SsaRequest` messages an `SsaAlmostRecovered` or
    // `SsaRecovered` event produced: `exit_requests_new_ssa_after_almost_recovered_event`,
    // `exit_ignores_stale_ssa_almost_recovered_event`,
    // `exit_handles_concurrent_almost_and_full_recovery_for_same_ssa` and
    // `exit_requests_new_ssa_on_recovery_when_not_already_pipelined`.
    //
    // That decision is the supervisor's now, and each of them has a direct counterpart against the
    // state machine: `almost_recovered_while_recovering_requests_next_once`,
    // `almost_recovered_while_awaiting_deposit_defers_request`,
    // `recovered_with_prior_early_event_does_not_fallback` and
    // `recovered_without_prior_early_event_falls_back_to_request`. Reaching the same logic through
    // an event channel, an action channel and a spawned driver only added scheduling noise — and
    // three of the four asserted a pipelining that no longer happens on an unfunded SSA, which is
    // the deliberate change: the Exit does not commit to another SSA before the current one is paid
    // for.
    //
    // What those tests did cover that the state machine cannot is the wiring itself, in both
    // directions. Actions reaching the wire is `the_opening_ssa_request_follows_session_established`
    // below; events reaching the supervisor is `session_is_closed_on_the_first_unverifiable_share`.

    /// Verifies that the supervisor's opening `RequestSsa` reaches the wire, and does so *after* the
    /// `SessionEstablished` that publishes the Session.
    ///
    /// The ordering is why the supervisor is spawned where it is. An `SsaRequest` arriving first
    /// would reference a Session the Entry has not been told exists.
    #[test_log::test(tokio::test)]
    async fn the_opening_ssa_request_follows_session_established() -> anyhow::Result<()> {
        use std::sync::Arc;

        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(SsaGeneratorConfig {
                polynomials_per_ssa: 2,
                threshold: 2,
                surplus_shares: 1,
            })
            .into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        // Records the kind of every outbound Start message, in order.
        let sent = Arc::new(std::sync::Mutex::new(Vec::<&'static str>::new()));
        let sent_clone = sent.clone();
        let mut bob_transport = MockMsgSender::new();
        bob_transport.expect_send_message().returning(move |_, data| {
            let sent_clone = sent_clone.clone();
            Box::pin(async move {
                match HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text) {
                    Ok(HoprStartProtocol::SessionEstablished(_)) => sent_clone.lock().unwrap().push("established"),
                    Ok(HoprStartProtocol::SsaRequest(req)) => {
                        assert_eq!(
                            req.dimensions().expect("SsaRequest params must be in range"),
                            PixParams::try_new(2, 2, TEST_SURPLUS_SHARES, LOCAL_PIX_SUITE).expect("valid"),
                            "SsaRequest must carry the negotiated dimensions, surplus included"
                        );
                        assert_eq!(
                            req.commitments.keys().copied().collect::<Vec<_>>(),
                            [SsaIndex::MIN],
                            "the opening request must commit to the first SSA index"
                        );
                        sent_clone.lock().unwrap().push("ssa_request");
                    }
                    _ => {}
                }
                Ok(())
            })
        });

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        tokio::time::timeout(Duration::from_secs(1), async {
            while sent.lock().unwrap().len() < 2 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("the supervisor's opening SsaRequest never reached the wire")?;

        assert_eq!(
            sent.lock().unwrap().as_slice(),
            ["established", "ssa_request"],
            "the Entry must learn the Session exists before it is asked to commit to an SSA"
        );

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    // `pipelined_ssa_preserves_earlier_deposit_deadline` lived here. It asserted that
    // `PixKillSwitch(1)` and `PixKillSwitch(2)` were independent entries in the `AbortableList`,
    // which is no longer a thing the manager can observe: per-SSA deadlines belong to the
    // supervisor. The invariant moved with them, to
    // `supervision::supervisor::tests::pipelining_a_second_ssa_leaves_the_first_ones_deadlines_alone`,
    // where it can compare the deadline instants rather than just the presence of two handles.

    /// Verifies that an explicit `close_session` leaves no reconstructor state behind for an SSA
    /// cycle that was still in flight.
    ///
    /// Retirement used to be an explicit sweep over every index the Session had ever used; it is
    /// now a consequence of aborting the action driver, which drops the guards it holds. This
    /// exercises one cycle because that is all a mock transport can get in flight — pipelining a
    /// second needs a funded first, and funding arrives from the chain. The multi-cycle case is
    /// covered where the guards are: `protocols/pix`'s guard tests and the supervisor's own suite.
    #[test_log::test(tokio::test)]
    async fn close_session_retires_in_flight_ssa_cycles() -> anyhow::Result<()> {
        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(SsaGeneratorConfig {
                polynomials_per_ssa: 2,
                threshold: 2,
                surplus_shares: 1,
            })
            .into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );
        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });
        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        // Grab a reference to the reconstructor before close_session consumes the slot.
        let pix_toolbox_ref = mgr.pix_toolbox.get().unwrap().clone();
        let share_processor = pix_toolbox_ref.share_processor;

        let ssa1 = SsaId::new(alice_pseudonym, SsaIndex::MIN);

        // Precondition: the first cycle's builder exists, so the assertion below proves actual
        // retirement rather than the absence of a builder that was never created. The supervisor's
        // opening `RequestSsa` reaches the driver asynchronously, so this waits rather than assumes.
        tokio::time::timeout(Duration::from_secs(1), async {
            while !share_processor.contains_builder(&ssa1) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("the opening SSA request never registered a builder")?;

        mgr.close_session(&alice_pseudonym);

        // Aborting the driver drops the `SsaCommitmentGuard`s it owns, and dropping a guard retires
        // its SSA. The abort takes effect at the next scheduling point, hence the wait.
        tokio::time::timeout(Duration::from_secs(1), async {
            while share_processor.contains_builder(&ssa1) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("close_session left the in-flight SSA's reconstructor state behind")?;

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Verifies that the entry/initiator (Alice) rejects a `SsaRequest` from the exit when the
    /// proposed SSA quota does not match what Alice offered in `pix_ssa_quota`.
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` and a generous PIX quota config. Alice's session initiation is
    ///    processed with `additional_data = (polynomials=2, shares=2)`.
    /// 2. `handle_ssa_request` is called with a mismatched quota: `(server_polynomials=10, server_shares=10)` while
    ///    Alice offered `(2, 2)`.
    /// 3. The call returns an error matching `TransportSessionError::Manager(SessionManagerError::Unacceptable(_))`,
    ///    confirming the quota mismatch was detected and rejected.
    #[test_log::test(tokio::test)]
    async fn entry_rejects_ssa_request_with_mismatched_quota() -> anyhow::Result<()> {
        use std::collections::BTreeMap;

        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        };

        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        // handle_incoming_session_initiation sends SessionEstablished + SsaRequest.
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        let session_id = alice_pseudonym;

        // Server sends dimensions of (10, 10) while we offered (2, 2) — should be rejected.
        let result = mgr
            .handle_ssa_request(
                alice_pseudonym,
                SsaServerCommitmentMessage::new(
                    session_id,
                    PixParams::try_new(10, 10, 0, LOCAL_PIX_SUITE)?,
                    BTreeMap::new(),
                    HoprPixDepositData::default(),
                ),
            )
            .await;

        bob_sender.close_channel();
        bob_handle.await??;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
        ));

        Ok(())
    }

    /// The Entry must not admit a successor before a default Exit could possibly have produced its
    /// early-recovery request.
    ///
    /// Recovery is counted in completed polynomials, while emission is windowed. Every fully drained
    /// window before the one containing the 85% boundary has already emitted both threshold and
    /// surplus shares. Dividing 85% by the global 1.25x surplus factor therefore underestimates the
    /// boundary: at the deployed dimensions the first possible early signal is about 87% through
    /// emission, not 68%.
    ///
    /// The gate compares against `min_emission_for_early_recovery` directly, so this test recomputes
    /// the boundary independently — from the same first principles, but without calling the function
    /// under test — and checks the two agree. A shared helper would agree with itself.
    #[test]
    fn successor_gate_waits_until_default_early_recovery_can_be_reached() {
        let params = DEFAULT_PIX_PARAMS;
        let polys = params.polys_per_ssa() as u64;
        let threshold = params.shares_per_poly() as u64;
        let surplus = params.surplus_shares() as u64;
        let window = (hopr_protocol_pix::SHARE_EMISSION_WINDOW as u64).min(polys);
        // The floor, because that is what the gate is computed at — see
        // `MIN_EARLY_RECOVERY_THRESHOLD`. The default must not sit below it, or a stock Exit would
        // ask before a stock Entry admits.
        let early = hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD;
        assert!(
            SsaReconstructorConfig::default().early_recovery_threshold >= early,
            "the shipped default must be admissible under the protocol floor it is gated against"
        );
        let needed = (early * polys as f64).ceil() as u64;

        // `needed` lies in this window. Earlier windows are fully exhausted, including surplus;
        // inside this one, `threshold - 1` complete passes and `in_window` shares of the threshold
        // pass are the absolute minimum that can have been emitted before those polynomials recover.
        let prior_windows = (needed - 1) / window;
        let prior_polys = prior_windows * window;
        let current_width = window.min(polys - prior_polys);
        let in_window = needed - prior_polys;
        let minimum_emitted = prior_polys * (threshold + surplus) + (threshold - 1) * current_width + in_window;
        let minimum_fraction = minimum_emitted as f64 / (polys * (threshold + surplus)) as f64;

        let gate = hopr_protocol_pix::min_emission_for_early_recovery(&params, early);
        assert_eq!(
            minimum_emitted,
            gate,
            "the successor gate opens at {} shares ({:.1}% of emission) against an independently derived earliest \
             honest request of {minimum_emitted} ({:.1}%); any shortfall permits a hostile Exit to solicit a deposit \
             before it is earned",
            gate,
            gate as f64 / (polys * (threshold + surplus)) as f64 * 100.0,
            minimum_fraction * 100.0,
        );

        // And the estimate this replaced is demonstrably below it, so the test would have caught it.
        assert!(
            ((early / 1.5) * (polys * (threshold + surplus)) as f64) < gate as f64,
            "the surplus-factor estimate must sit below the real boundary"
        );
    }

    /// Dimensions wide enough that the protocol floor and a stricter local threshold disagree.
    ///
    /// At `small_pix_params`' two polynomials, `ceil(0.85 x 2)` and `ceil(1.0 x 2)` are both 2, so
    /// every threshold in range produces the same boundary and a test built on them cannot tell which
    /// one the gate used. Eight separates them: 7 polynomials against 8.
    fn wide_pix_params() -> PixParams {
        PixParams::try_new(8, 2, TEST_SURPLUS_SHARES, LOCAL_PIX_SUITE).expect("test dimensions must be valid")
    }

    /// Builds an Entry-side manager serving `params`, with the reconstructor at `early_threshold`.
    ///
    /// Returns the manager, its generator, and the PIX event stream, with one PIX Session already
    /// established for the returned pseudonym.
    #[allow(clippy::type_complexity)]
    async fn entry_with_pix_session(
        params: PixParams,
        early_threshold: f64,
    ) -> anyhow::Result<(
        SessionManager<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>,
        Arc<SsaShareGenerator<HoprPixSpec>>,
        HoprPseudonym,
        impl futures::Stream<Item = HoprSessionOutPixEvent> + Unpin,
        UnboundedSender<(DestinationRouting, ApplicationDataOut)>,
        tokio::task::JoinHandle<anyhow::Result<()>>,
    )> {
        use hopr_protocol_start::StartInitiation;

        let generator = Arc::new(SsaShareGenerator::new(SsaGeneratorConfig {
            polynomials_per_ssa: params.polys_per_ssa(),
            threshold: params.shares_per_poly(),
            surplus_shares: params.surplus_shares(),
        }));
        let (pix_toolbox, pix_events) = PixToolbox::new(
            generator.clone(),
            SsaReconstructor::new(SsaReconstructorConfig {
                early_recovery_threshold: early_threshold,
                ..Default::default()
            })
            .into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: params.into_additional_data(0),
            },
        )
        .await?;

        Ok((mgr, generator, pseudonym, pix_events, bob_sender, bob_handle))
    }

    /// One SSA commitment at `index`, from the identity element the Exit-side point is not checked
    /// against here.
    fn ssa_request_for(index: u32) -> std::collections::BTreeMap<SsaIndex, HoprPixGroupElement> {
        let identity = HoprPixGroupElement::try_from(
            hopr_protocol_pix::PixGroup::<HoprPixSpec>::default()
                .to_bytes()
                .as_ref(),
        )
        .expect("identity element must be valid");
        std::collections::BTreeMap::from([(SsaIndex::new(index).expect("non-zero"), identity)])
    }

    fn drain_pix_events(events: &mut (dyn futures::Stream<Item = HoprSessionOutPixEvent> + Unpin)) -> usize {
        let mut seen = 0;
        while futures::FutureExt::now_or_never(events.next()).flatten().is_some() {
            seen += 1;
        }
        seen
    }

    /// Credits `count` Exit → Entry packets to the Session, as its receive path would.
    ///
    /// These fixtures build their slot through `handle_incoming_session_initiation`, whose receive
    /// path counts the *outgoing* direction, so the Entry-side counter the successor gate reads has
    /// to be driven by hand. `returned_packets_are_counted_on_the_entry_receive_path` is what pins
    /// that a real Entry Session advances it; here it only has to move.
    fn credit_returned_packets<S>(mgr: &SessionManager<S>, pseudonym: &HoprPseudonym, count: u64)
    where
        S: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
        S::Error: std::error::Error + Send + Sync + 'static,
    {
        mgr.sessions
            .get(pseudonym)
            .expect("session must exist")
            .returned_packets
            .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
    }

    /// Packets the successor gate requires before it will admit a batch beyond `watermark`.
    ///
    /// Mirrors `handle_ssa_request`'s arithmetic rather than calling it, so that a change to the
    /// boundary has to be restated here — a helper shared with the code under test would agree with
    /// itself whatever either of them did.
    fn required_returned_packets(params: &PixParams, watermark: u32) -> u64 {
        let min_emitted =
            hopr_protocol_pix::min_emission_for_early_recovery(params, hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD);
        let shares_per_cycle = params.polys_per_ssa() as u64 * params.emitted_shares_per_poly() as u64;
        let target = (watermark as u64 - 1) * shares_per_cycle + min_emitted;
        (target as u128 * params.shares_per_poly() as u128 / params.emitted_shares_per_poly() as u128) as u64
    }

    /// The successor gate must open at the protocol floor, not at the local reconstructor's threshold.
    ///
    /// The value that decides when a correct Exit asks for its next batch is the *peer's*
    /// `early_recovery_threshold`, and it does not travel on the wire. Deriving the boundary from the
    /// Entry's own copy meant two individually valid configurations could not communicate: an Exit
    /// configured lower sent its one-shot request before the gate opened, the Entry dropped it
    /// silently, and the Session died on `max_ssa_delivery_time` waiting for a commitment that had
    /// been deliberately refused — a failure visible on neither node as a configuration error.
    ///
    /// So the gate is computed at [`hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD`], which
    /// `validate_pix_supervision` holds every Exit to. This fixture puts the local reconstructor at
    /// `1.0` — the strictest end of the range — and asks at exactly the floor's boundary. A gate built
    /// on the local value wants one more share and refuses.
    #[test_log::test(tokio::test)]
    async fn successor_gate_opens_at_the_protocol_floor_not_the_local_threshold() -> anyhow::Result<()> {
        let params = wide_pix_params();
        let (mgr, generator, pseudonym, mut pix_events, bob_sender, bob_handle) =
            entry_with_pix_session(params, 1.0).await?;

        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));

        // Fully served, so that the emission boundary is the only thing this test can be measuring.
        credit_returned_packets(&mgr, &pseudonym, required_returned_packets(&params, 1));

        let at_floor = hopr_protocol_pix::min_emission_for_early_recovery(
            &params,
            hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD,
        );
        let at_local = hopr_protocol_pix::min_emission_for_early_recovery(&params, 1.0);
        assert!(
            at_floor < at_local,
            "the fixture must separate the two boundaries, got {at_floor} and {at_local}"
        );

        // One share short of the floor's boundary, the request is early under either reading.
        for sent in 1..at_floor as u32 {
            generator.next_share(&pseudonym, &sent.to_be_bytes())?;
        }
        let err = mgr
            .handle_ssa_request(
                pseudonym,
                SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
            )
            .await
            .expect_err("a request one share below the boundary must be refused");
        assert!(
            matches!(
                err,
                TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
            ),
            "expected Unacceptable, got {err:?}"
        );
        assert_eq!(0, drain_pix_events(&mut pix_events));

        // On the boundary itself it is admitted — and would not be if the gate read the local 1.0.
        generator.next_share(&pseudonym, &at_floor.to_be_bytes())?;
        assert_eq!(
            at_floor,
            generator
                .emission_progress(&pseudonym)
                .expect("committed")
                .front_emitted,
            "the fixture must stand exactly on the floor's boundary"
        );
        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
        )
        .await
        .context("a request at the protocol floor must be accepted whatever the local threshold is")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// A successor request arriving after the generator's state was discarded must be refused.
    ///
    /// The gate only applies where there is emission progress to measure, and absent progress is
    /// otherwise the ordinary opening batch. But the generator keeps its per-pseudonym state in a
    /// cache with an idle retention refreshed by share *emission*, so a Session kept alive while the
    /// Entry emits nothing outlives it — and the successor gate is then not relaxed but deleted, along
    /// with the monotonic index the discarded entry carried. An Exit that arranges that farms one
    /// unearned deposit per retention period, which is the exposure the gate exists to close.
    ///
    /// `forget` reaches the same state the retention would, deliberately: an evicted entry and an
    /// explicitly dropped one are indistinguishable from the outside, which is precisely why the fact
    /// that a batch *was* committed has to be held somewhere that lives as long as the Session.
    ///
    /// The opening request in this test is the other half of the property — absent state with nothing
    /// committed is still admitted, or no PIX Session could ever start.
    #[test_log::test(tokio::test)]
    async fn an_ssa_request_against_discarded_generator_state_is_refused_and_closes_the_session() -> anyhow::Result<()>
    {
        let params = small_pix_params();
        let (mgr, generator, pseudonym, mut pix_events, bob_sender, bob_handle) =
            entry_with_pix_session(params, SsaReconstructorConfig::default().early_recovery_threshold).await?;

        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));
        assert_eq!(vec![pseudonym], mgr.active_sessions());

        generator.forget(&pseudonym);
        assert!(
            generator.emission_progress(&pseudonym).is_none(),
            "the fixture must reproduce the state an idle eviction leaves behind"
        );

        let err = mgr
            .handle_ssa_request(
                pseudonym,
                SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
            )
            .await
            .expect_err("a successor request against discarded generator state must be refused");
        assert!(
            matches!(
                err,
                TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
            ),
            "expected Unacceptable, got {err:?}"
        );
        assert_eq!(
            0,
            drain_pix_events(&mut pix_events),
            "a refused request must not emit a single ReadyToDeposit"
        );
        assert!(
            mgr.active_sessions().is_empty(),
            "the Entry can no longer emit shares for the cycles it committed to, so leaving the Session open only \
             makes the Exit wait out its commitment deadline"
        );

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Emission is not service: a successor must be refused until the Exit has returned data.
    ///
    /// This is H6, and it is the case the emission half of the gate cannot see. `emission_progress`
    /// counts shares this node handed to its own packet pipeline — a consumption `create_surb_for_path`
    /// does not even roll back when the rest of the packet build fails. An Exit that requests, is
    /// funded and then returns nothing still walks that counter forward for as long as the Entry keeps
    /// sending, so on its own it prices deposits against work the Entry did to itself.
    ///
    /// The fixture therefore satisfies the emission half completely and the returned half not at all.
    /// Before this gate existed the request below was admitted and a second deposit instruction went
    /// out for it.
    #[test_log::test(tokio::test)]
    async fn entry_refuses_a_successor_the_exit_has_not_paid_for_with_returned_data() -> anyhow::Result<()> {
        let params = wide_pix_params();
        let (mgr, generator, pseudonym, mut pix_events, bob_sender, bob_handle) =
            entry_with_pix_session(params, SsaReconstructorConfig::default().early_recovery_threshold).await?;

        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));

        // Emit the whole cycle: the emission half of the gate is now satisfied several times over,
        // and it is the only half that would have been consulted before.
        for sent in 1..=(params.polys_per_ssa() as u32 * params.emitted_shares_per_poly() as u32) {
            generator.next_share(&pseudonym, &sent.to_be_bytes())?;
        }
        let progress = generator.emission_progress(&pseudonym).expect("committed");
        assert!(
            progress.is_serving_last_committed()
                && progress.front_emitted
                    >= hopr_protocol_pix::min_emission_for_early_recovery(
                        &params,
                        hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD
                    ),
            "the fixture must satisfy the emission half outright, or it is not testing the other one"
        );

        let err = mgr
            .handle_ssa_request(
                pseudonym,
                SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
            )
            .await
            .expect_err("a successor must not be funded before the Exit has returned anything");
        assert!(
            matches!(
                err,
                TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
            ),
            "expected Unacceptable, got {err:?}"
        );
        assert_eq!(
            0,
            drain_pix_events(&mut pix_events),
            "a refused successor must not emit a single ReadyToDeposit"
        );
        assert_eq!(
            vec![pseudonym],
            mgr.active_sessions(),
            "an under-served request is refused, not fatal — the Exit may still earn the batch it holds"
        );

        // And the same request is admitted the moment the service exists.
        credit_returned_packets(&mgr, &pseudonym, required_returned_packets(&params, 1));
        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
        )
        .await
        .context("a successor must be admitted once the Exit has returned what it owes")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// An Exit that has returned nothing is refused immediately, not held.
    ///
    /// The deferral below the boundary exists for reordering, and it holds a bounded
    /// `for_each_concurrent` slot and the per-pseudonym request lock while it runs. That is only safe
    /// because it is entered exclusively for a *near miss*: an Exit which has returned nothing at all
    /// must cost nothing at all to refuse, or "ask early and say nothing" becomes a way to park one of
    /// this node's Start-protocol slots per Session it holds.
    #[test_log::test(tokio::test)]
    async fn an_unserved_successor_is_refused_without_waiting() -> anyhow::Result<()> {
        let params = wide_pix_params();
        let (mgr, generator, pseudonym, mut pix_events, bob_sender, bob_handle) =
            entry_with_pix_session(params, SsaReconstructorConfig::default().early_recovery_threshold).await?;

        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));

        for sent in 1..=(params.polys_per_ssa() as u32 * params.emitted_shares_per_poly() as u32) {
            generator.next_share(&pseudonym, &sent.to_be_bytes())?;
        }

        let started = std::time::Instant::now();
        let err = mgr
            .handle_ssa_request(
                pseudonym,
                SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
            )
            .await
            .expect_err("an unserved successor must be refused");
        let elapsed = started.elapsed();

        assert!(
            matches!(
                err,
                TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
            ),
            "expected Unacceptable, got {err:?}"
        );
        assert!(
            elapsed * 4 < SSA_SUCCESSOR_SERVICE_WAIT,
            "refusing an Exit that returned nothing took {elapsed:?}, which is within reach of the \
             {SSA_SUCCESSOR_SERVICE_WAIT:?} near-miss wait — it must not have entered it at all"
        );

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// A request that arrives just ahead of the packets that earned it is waited for, not refused.
    ///
    /// A conforming Exit asks the instant its reconstructor crosses the threshold, and that request
    /// travels the same mixed path as the returned packets which unlocked the shares — so it can
    /// overtake the last few of them. Refusing it is not a mild outcome: `RequestSsa` is emitted once
    /// per index and never retried, so the Exit sits in `AwaitingCommitment` until
    /// `max_ssa_delivery_time` and then closes the Session as `CommitmentTimeout`. The gate must
    /// therefore absorb the reordering rather than treat it as under-service.
    #[test_log::test(tokio::test)]
    async fn a_near_miss_successor_waits_for_the_packets_still_in_flight() -> anyhow::Result<()> {
        let params = wide_pix_params();
        let (mgr, generator, pseudonym, mut pix_events, bob_sender, bob_handle) =
            entry_with_pix_session(params, SsaReconstructorConfig::default().early_recovery_threshold).await?;

        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));

        for sent in 1..=(params.polys_per_ssa() as u32 * params.emitted_shares_per_poly() as u32) {
            generator.next_share(&pseudonym, &sent.to_be_bytes())?;
        }

        // One packet short: the request is in flight ahead of the last one that earned it.
        let required = required_returned_packets(&params, 1);
        assert!(required > 1, "the fixture needs room to be one short");
        credit_returned_packets(&mgr, &pseudonym, required - 1);

        // The straggler lands while the request is already being evaluated.
        let mgr_clone = mgr.clone();
        let late = tokio::spawn(async move {
            tokio::time::sleep(SSA_SUCCESSOR_SERVICE_POLL).await;
            credit_returned_packets(&mgr_clone, &pseudonym, 1);
        });

        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
        )
        .await
        .context("a request one packet ahead of its own service must be waited for, not refused")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));
        late.await?;

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Service rendered before the first commitment does not pay for the first successor.
    ///
    /// Until a cycle is committed the generator holds no polynomials, so the SURBs going out carry no
    /// shares — and the Exit may legitimately be served up to `max_predeposit_packets` of them before
    /// any deposit exists. Crediting that prefix would let an Exit bank unpaid service against the
    /// first cycle it *is* paid for, which is the one window in a Session's life where the gate has no
    /// prior cycle to measure against.
    #[test_log::test(tokio::test)]
    async fn service_returned_before_the_first_commitment_is_not_credited() -> anyhow::Result<()> {
        let params = wide_pix_params();
        let (mgr, generator, pseudonym, mut pix_events, bob_sender, bob_handle) =
            entry_with_pix_session(params, SsaReconstructorConfig::default().early_recovery_threshold).await?;

        // Generously served *before* anything is committed, and none of it may count.
        let required = required_returned_packets(&params, 1);
        credit_returned_packets(&mgr, &pseudonym, required * 10);

        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(1, drain_pix_events(&mut pix_events));

        for sent in 1..=(params.polys_per_ssa() as u32 * params.emitted_shares_per_poly() as u32) {
            generator.next_share(&pseudonym, &sent.to_be_bytes())?;
        }

        let err = mgr
            .handle_ssa_request(
                pseudonym,
                SsaServerCommitmentMessage::new(pseudonym, params, ssa_request_for(2), HoprPixDepositData::default()),
            )
            .await
            .expect_err("pre-commitment service must not pay for the first successor");
        assert!(
            matches!(
                err,
                TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
            ),
            "expected Unacceptable, got {err:?}"
        );
        assert_eq!(0, drain_pix_events(&mut pix_events));

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// The boundary is discounted by exactly the loss the surplus insures against.
    ///
    /// The Exit unlocks a share when the *first relayer* acknowledges its returned packet, which is
    /// upstream of the Entry — so everything lost after that point is progress the Exit legitimately
    /// has and this node cannot observe. Demanding the undiscounted emission boundary would refuse
    /// conforming Exits on any lossy path; an Exit losing more than the surplus covers could not have
    /// reconstructed the cycle at all, so the surplus ratio is the honest allowance.
    ///
    /// Pinned at the deployed dimensions because the resulting fraction is not something a reader can
    /// check by inspection, and it is what decides when real money moves.
    #[test]
    fn the_successor_boundary_is_discounted_by_the_surplus_it_insures() -> anyhow::Result<()> {
        use crate::{DEFAULT_PIX_POLYS_PER_SSA, DEFAULT_PIX_SHARES_PER_POLY, DEFAULT_PIX_SURPLUS_SHARES};

        let params = PixParams::try_new(
            DEFAULT_PIX_POLYS_PER_SSA,
            DEFAULT_PIX_SHARES_PER_POLY,
            DEFAULT_PIX_SURPLUS_SHARES,
            LOCAL_PIX_SUITE,
        )?;
        let cycle = params.polys_per_ssa() as u64 * params.emitted_shares_per_poly() as u64;
        assert_eq!(655_360, cycle);

        let undiscounted = hopr_protocol_pix::min_emission_for_early_recovery(
            &params,
            hopr_protocol_pix::MIN_EARLY_RECOVERY_THRESHOLD,
        );
        assert_eq!(569_140, undiscounted);

        let required = required_returned_packets(&params, 1);
        assert_eq!(455_312, required);
        assert_eq!(
            undiscounted * DEFAULT_PIX_SHARES_PER_POLY as u64
                / (DEFAULT_PIX_SHARES_PER_POLY + DEFAULT_PIX_SURPLUS_SHARES) as u64,
            required,
            "the discount must be the surplus ratio and nothing else"
        );

        let fraction = required as f64 / cycle as f64;
        assert!(
            (0.694..0.696).contains(&fraction),
            "the deployed successor boundary must sit at ~69.5% of a cycle's returned packets, got {fraction}"
        );

        // Each further cycle of a batch is demanded in full, at the same discount.
        assert_eq!(
            required
                + cycle * DEFAULT_PIX_SHARES_PER_POLY as u64
                    / (DEFAULT_PIX_SHARES_PER_POLY + DEFAULT_PIX_SURPLUS_SHARES) as u64,
            required_returned_packets(&params, 2),
            "a batch of two must demand the first cycle's whole discounted worth plus the second's boundary"
        );

        Ok(())
    }

    /// A real Entry Session must advance the counter the successor gate reads.
    ///
    /// The gate's worst failure is not being too strict, it is being wired to a counter nothing
    /// feeds: that closes it permanently, and every PIX Session then dies on its second cycle with
    /// the Exit blaming its own commitment deadline. The other tests here drive
    /// `returned_packets` by hand, so none of them would notice.
    ///
    /// Deliberately built with `surb_management: None`. That is the branch which passed `session_rx`
    /// through untouched and left `surb_estimator` at its default — the one place where reusing the
    /// balancer's estimate instead of a counter of our own would have produced exactly that silent
    /// permanent closure.
    #[test_log::test(tokio::test)]
    async fn returned_packets_are_counted_on_the_entry_receive_path() -> anyhow::Result<()> {
        use hopr_protocol_start::StartProtocolDiscriminants;

        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let alice_mgr = SessionManager::new(Default::default());
        let bob_mgr = SessionManager::new(Default::default());

        let mut alice_transport = MockMsgSender::new();
        let mut bob_transport = MockMsgSender::new();

        // Alice → Bob, and Bob → Alice, with no filtering: the handshake is not what is under test.
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport.expect_send_message().returning(move |_, data| {
            let bob_mgr_clone = bob_mgr_clone.clone();
            Box::pin(async move {
                let _ = bob_mgr_clone.dispatch_message(
                    alice_pseudonym,
                    ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    },
                );
                Ok(())
            })
        });
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .withf(|_, data| msg_type(data, StartProtocolDiscriminants::SessionEstablished))
            .returning(move |_, data| {
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(
                        alice_pseudonym,
                        ApplicationDataIn {
                            data: data.data,
                            packet_info: Default::default(),
                        },
                    )?;
                    Ok(())
                })
            });

        let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
        let (new_session_tx_alice, _alice_rx) = futures::channel::mpsc::channel(1024);
        alice_mgr.start(alice_sender.clone(), new_session_tx_alice, None)?;

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        bob_mgr.start(bob_sender.clone(), new_session_tx_bob, None)?;
        let _bob_notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx_bob);
            while let Some(_session) = new_session_rx_bob.next().await {}
        });

        let alice_session = alice_mgr
            .new_session(
                bob_peer,
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    pseudonym: alice_pseudonym.into(),
                    capabilities: None.into(),
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await
            .context("the session must establish")?;

        let counter = alice_mgr
            .sessions
            .get(&alice_pseudonym)
            .expect("alice must hold the session slot")
            .returned_packets;
        assert_eq!(
            0,
            counter.load(std::sync::atomic::Ordering::Relaxed),
            "nothing has come back yet"
        );

        // Three packets arriving the way the Exit's return traffic does.
        for i in 0..3u8 {
            alice_mgr.dispatch_message(
                alice_pseudonym,
                ApplicationDataIn {
                    data: ApplicationData::new(SESSION_APPLICATION_TAG, &[i])?,
                    packet_info: Default::default(),
                },
            )?;
        }

        // The count is taken as the Session drains its receiver, and the receiver is only polled by a
        // reader — which is exactly how it behaves in production, where the client is reading.
        let mut alice_session = alice_session;
        let mut buffer = [0u8; 16];
        for _ in 0..50 {
            if counter.load(std::sync::atomic::Ordering::Relaxed) >= 3 {
                break;
            }
            let _ = tokio::time::timeout(
                Duration::from_millis(20),
                futures::AsyncReadExt::read(&mut alice_session, &mut buffer),
            )
            .await;
        }
        assert_eq!(
            3,
            counter.load(std::sync::atomic::Ordering::Relaxed),
            "every Exit → Entry packet must be counted, or the successor gate can never open"
        );

        drop(alice_session);
        alice_sender.close_channel();
        bob_sender.close_channel();
        let _ = alice_handle.await?;
        let _ = bob_handle.await?;
        Ok(())
    }

    /// The Entry's half of the successor gate: a batch asked for before emission has reached the last
    /// cycle of the batch already committed must be refused, committing nothing and depositing
    /// nothing — and the Session must survive it.
    ///
    /// The Exit-side gate in `SessionPixSupervisor` means a correct peer never lands here. This is
    /// what makes a peer that *does* — an unpatched or hostile Exit answering every cycle's
    /// early-recovery signal with a fresh batch — cost the Entry nothing rather than one on-chain
    /// deposit per surplus request.
    ///
    /// Ordering across a batch is pinned where it lives, on the generator, by
    /// `emission_never_crosses_a_cycle_boundary_early` and
    /// `emission_progress_lags_the_commitment_index_across_a_batch`. What this test drives is the
    /// wiring and all three points of the gate: nothing emitted at all, emission having merely
    /// *reached* the last committed cycle, and emission far enough into it.
    ///
    /// The middle point is the regression. Admission used to be "emission has reached the last
    /// committed index", which becomes true on that cycle's very first share — so a successor batch
    /// was admitted ~0 % into the batch rather than the ~85 % at which a conforming Exit asks. See
    /// [`MIN_SUCCESSOR_EMISSION_FRACTION`].
    #[test_log::test(tokio::test)]
    async fn entry_refuses_a_batch_asked_for_before_emission_reaches_the_current_one() -> anyhow::Result<()> {
        use std::collections::BTreeMap;

        use hopr_protocol_pix::{PixGroup, SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        const BATCH: u32 = 3;

        let generator = Arc::new(SsaShareGenerator::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        }));
        let (pix_toolbox, mut pix_events) = PixToolbox::new(
            generator.clone(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            max_ssas_per_ssa_request: BATCH as usize,
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        let identity = HoprPixGroupElement::try_from(PixGroup::<HoprPixSpec>::default().to_bytes().as_ref())
            .expect("identity element must be valid");
        let batch = |from: u32| {
            (from..from + BATCH)
                .map(|i| (SsaIndex::new(i).expect("non-zero"), identity))
                .collect::<BTreeMap<_, _>>()
        };
        // Drains whatever the PIX event stream has ready, without awaiting more.
        // `futures::FutureExt` is spelled out because `moka::future::FutureExt` is also in scope.
        let drain_deposits = |events: &mut (dyn futures::Stream<Item = HoprSessionOutPixEvent> + Unpin)| {
            let mut seen = 0;
            while futures::FutureExt::now_or_never(events.next()).flatten().is_some() {
                seen += 1;
            }
            seen
        };

        // The opening batch is accepted: nothing is committed yet, so there is nothing to be early for.
        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, small_pix_params(), batch(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(
            drain_deposits(&mut pix_events),
            BATCH as usize,
            "the opening batch must yield one deposit address per SSA"
        );

        // Fully served for the batch, so emission ordering is the only thing under test here. The
        // returned-data half of the gate has its own tests.
        credit_returned_packets(&mgr, &pseudonym, required_returned_packets(&small_pix_params(), BATCH));

        // Asking again before a single share has gone out is a whole batch early.
        let err = mgr
            .handle_ssa_request(
                pseudonym,
                SsaServerCommitmentMessage::new(
                    pseudonym,
                    small_pix_params(),
                    batch(BATCH + 1),
                    HoprPixDepositData::default(),
                ),
            )
            .await
            .expect_err("a batch asked for before any share was emitted must be refused");
        assert!(
            matches!(
                err,
                TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
            ),
            "expected Unacceptable, got {err:?}"
        );
        assert_eq!(
            drain_deposits(&mut pix_events),
            0,
            "a refused batch must not emit a single ReadyToDeposit"
        );
        assert_eq!(
            mgr.active_sessions(),
            vec![pseudonym],
            "refusing a request must not close the Session"
        );

        // Emit until the last cycle of the batch has taken the front — but no further.
        let mut sent = 0u32;
        let mut emit_until = |pred: &dyn Fn(&hopr_protocol_pix::EmissionProgress) -> bool| -> anyhow::Result<()> {
            for _ in 0..1024 {
                if generator.emission_progress(&pseudonym).as_ref().is_some_and(pred) {
                    return Ok(());
                }
                sent += 1;
                generator.next_share(&pseudonym, &sent.to_be_bytes())?;
            }
            anyhow::bail!("emission did not reach the expected point")
        };
        emit_until(&|p| p.is_serving_last_committed())?;

        // The boundary the gate actually uses, recomputed from the same negotiated params.
        let min_emitted = hopr_protocol_pix::min_emission_for_early_recovery(
            &small_pix_params(),
            SsaReconstructorConfig::default().early_recovery_threshold,
        );

        // Reaching the last cycle is *not* enough, and this is the H2 regression. The index-only gate
        // this replaced admitted a successor batch right here — on the last cycle's first share, ~0 %
        // of the way through the batch — which is nearly a whole cycle of deposits the Exit has not
        // earned, on the one gate whose purpose is to prevent exactly that.
        let progress = generator.emission_progress(&pseudonym).expect("committed");
        assert!(
            progress.front_emitted < min_emitted,
            "the test must stand at the start of the last cycle, got {} of {min_emitted}",
            progress.front_emitted
        );
        let err = mgr
            .handle_ssa_request(
                pseudonym,
                SsaServerCommitmentMessage::new(
                    pseudonym,
                    small_pix_params(),
                    batch(BATCH + 1),
                    HoprPixDepositData::default(),
                ),
            )
            .await
            .expect_err("reaching the last cycle must not by itself admit the next batch");
        assert!(
            matches!(
                err,
                TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
            ),
            "expected Unacceptable, got {err:?}"
        );
        assert_eq!(
            drain_deposits(&mut pix_events),
            0,
            "a batch refused for being early must not emit a single ReadyToDeposit"
        );

        // Far enough into that last cycle, the same batch is admitted.
        emit_until(&|p| p.front_emitted >= min_emitted)?;
        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(
                pseudonym,
                small_pix_params(),
                batch(BATCH + 1),
                HoprPixDepositData::default(),
            ),
        )
        .await
        .context("a batch asked for on time must be accepted")?;

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Concurrent `SsaRequest`s for one pseudonym must admit exactly one batch.
    ///
    /// Start messages are processed under `for_each_concurrent`, so several requests for the same
    /// Session run at once, and each reads the successor gate before any of them has advanced it.
    /// Requests are issued at distinct, ascending index ranges because that is the shape the
    /// generator's monotonic index cannot refuse — it rejects equal or lower indices, and an Exit
    /// numbering its batches upwards never offers one.
    ///
    /// **What this pins is the invariant, not the mechanism.** At this fixture's dimensions — two
    /// polynomials at threshold two — `new_ssa_commitment` returns in microseconds, so the window
    /// between reading the gate and advancing it is too narrow to hit: removing
    /// [`ssa_request_locks`](SessionManager::ssa_request_locks) leaves the outcome unchanged here.
    /// The window is a function of that call's cost, which at the deployed 8192 x 64 is around a
    /// second, and a fixture that large is not something to build into a unit test. So this test
    /// guards the property and the serialisation is justified by the shape of the code rather than
    /// by this failing without it.
    ///
    /// It does earn its place: it is what would catch the gate being made per-entry, the guard being
    /// taken after the gate instead of before it, or the handler gaining an await between the two.
    #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
    async fn concurrent_ssa_requests_for_one_pseudonym_admit_one_batch() -> anyhow::Result<()> {
        use std::collections::BTreeMap;

        use hopr_protocol_pix::{PixGroup, SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        const BATCH: u32 = 2;
        const RACERS: u32 = 4;

        let generator = Arc::new(SsaShareGenerator::new(SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        }));
        let (pix_toolbox, mut pix_events) = PixToolbox::new(
            generator.clone(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            max_ssas_per_ssa_request: BATCH as usize,
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        let identity = HoprPixGroupElement::try_from(PixGroup::<HoprPixSpec>::default().to_bytes().as_ref())
            .expect("identity element must be valid");
        let batch = |from: u32| {
            (from..from + BATCH)
                .map(|i| (SsaIndex::new(i).expect("non-zero"), identity))
                .collect::<BTreeMap<_, _>>()
        };
        let drain_deposits = |events: &mut (dyn futures::Stream<Item = HoprSessionOutPixEvent> + Unpin)| {
            let mut seen = 0;
            while futures::FutureExt::now_or_never(events.next()).flatten().is_some() {
                seen += 1;
            }
            seen
        };

        // The opening batch, then emission far enough into its last cycle that exactly one successor
        // batch is legitimately admissible.
        mgr.handle_ssa_request(
            pseudonym,
            SsaServerCommitmentMessage::new(pseudonym, small_pix_params(), batch(1), HoprPixDepositData::default()),
        )
        .await
        .context("the opening batch must be accepted")?;
        assert_eq!(BATCH as usize, drain_deposits(&mut pix_events));

        // Served for the batch, so that serialisation is the only thing deciding the outcome below.
        credit_returned_packets(&mgr, &pseudonym, required_returned_packets(&small_pix_params(), BATCH));

        let min_emitted = hopr_protocol_pix::min_emission_for_early_recovery(
            &small_pix_params(),
            SsaReconstructorConfig::default().early_recovery_threshold,
        );
        let mut sent = 0u32;
        for _ in 0..1024 {
            if generator
                .emission_progress(&pseudonym)
                .is_some_and(|p| p.is_serving_last_committed() && p.front_emitted >= min_emitted)
            {
                break;
            }
            sent += 1;
            generator.next_share(&pseudonym, &sent.to_be_bytes())?;
        }

        // Real tasks on a multi-threaded runtime, not `join_all`: `join_all` polls in order, so the
        // first future runs its gate check and its whole commitment phase before the second is polled
        // at all, and the requests never actually overlap. That shape passes with or without the
        // serialisation and so proves nothing about it.
        let barrier = Arc::new(tokio::sync::Barrier::new(RACERS as usize));
        let outcomes = futures::future::join_all((0..RACERS).map(|r| {
            let mgr = mgr.clone();
            let barrier = barrier.clone();
            let commitments = batch(BATCH + 1 + r * BATCH);
            tokio::spawn(async move {
                barrier.wait().await;
                mgr.handle_ssa_request(
                    pseudonym,
                    SsaServerCommitmentMessage::new(
                        pseudonym,
                        small_pix_params(),
                        commitments,
                        HoprPixDepositData::default(),
                    ),
                )
                .await
            })
        }))
        .await
        .into_iter()
        .map(|joined| joined.expect("racer task must not panic"))
        .collect::<Vec<_>>();

        let admitted = outcomes.iter().filter(|o| o.is_ok()).count();
        assert_eq!(
            1, admitted,
            "exactly one of {RACERS} concurrent requests may be admitted, got {admitted}: {outcomes:?}"
        );
        assert_eq!(
            BATCH as usize,
            drain_deposits(&mut pix_events),
            "only the admitted batch may produce deposit instructions"
        );

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// An `SsaRequest` asking for more than [`SessionManagerConfig::max_ssas_per_ssa_request`] SSA
    /// commitments must be rejected outright, before any commitment is generated or any
    /// `ReadyToDeposit` is emitted — and a batch *at* the configured cap must be accepted, so that
    /// raising the knob is what actually admits a larger batch.
    ///
    /// Each accepted entry costs a full client commitment plus its own on-chain deposit, so without
    /// this cap one inbound packet could amplify into up to `MAX_SSAS_PER_REQUEST` (27) deposits.
    #[test_log::test(tokio::test)]
    async fn entry_rejects_ssa_request_exceeding_configured_ssa_cap() -> anyhow::Result<()> {
        use std::collections::BTreeMap;

        use hopr_crypto_packet::prelude::HoprPixGroupElement;
        use hopr_protocol_pix::{PixGroup, SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        /// Outcome of offering a batch: what `handle_ssa_request` returned, how many `SessionError`
        /// messages the Entry sent back, and whether it kept the Session.
        struct Outcome {
            result: errors::Result<()>,
            session_errors: usize,
            session_alive: bool,
        }

        // `batch` entries offered against an Entry configured to accept at most `cap`.
        async fn offer_batch(cap: usize, batch: u32) -> anyhow::Result<Outcome> {
            // The PIX event stream must stay alive: an accepted batch emits one `ReadyToDeposit` per
            // entry, and a dropped receiver would fail the send and mask the acceptance as an error.
            let (pix_toolbox, _pix_events) = PixToolbox::new(
                SsaShareGenerator::new(SsaGeneratorConfig {
                    polynomials_per_ssa: 2,
                    threshold: 2,
                    surplus_shares: 1,
                })
                .into(),
                SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
            );

            let mgr = SessionManager::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    quota_range: 0..=1024 * 1024 * 1024,
                    ..Default::default()
                },
                max_ssas_per_ssa_request: cap,
                ..Default::default()
            });

            // Count the SessionError replies: a refusal has to be *told* to the Exit, which cannot
            // otherwise observe it and has no path back to a new request.
            let session_errors = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let session_errors_tx = session_errors.clone();
            let mut bob_transport = MockMsgSender::new();
            bob_transport
                .expect_send_message()
                .times(1..)
                .returning(move |_, data| {
                    if crate::testing::msg_type(&data, StartProtocolDiscriminants::SessionError) {
                        session_errors_tx.fetch_add(1, Ordering::Relaxed);
                    }
                    Box::pin(async { Ok(()) })
                });

            let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
            let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
            let _notifications = tokio::spawn(async move {
                pin_mut!(new_session_rx);
                while let Some(_session) = new_session_rx.next().await {}
            });
            mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

            let alice_pseudonym = HoprPseudonym::random();

            mgr.handle_incoming_session_initiation(
                alice_pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                    additional_data: small_pix_additional_data(),
                },
            )
            .await?;

            let session_id = alice_pseudonym;
            let identity = HoprPixGroupElement::try_from(PixGroup::<HoprPixSpec>::default().to_bytes().as_ref())
                .expect("identity element must be valid");

            // Indices start above whatever the Exit-side establishment already allocated, so the
            // accepted case is not rejected for non-monotonicity instead of for its size.
            let commitments: BTreeMap<_, _> = (100..100 + batch)
                .map(|i| (SsaIndex::new(i).expect("non-zero"), identity))
                .collect();

            let result = mgr
                .handle_ssa_request(
                    alice_pseudonym,
                    SsaServerCommitmentMessage::new(
                        session_id,
                        PixParams::try_new(2, 2, TEST_SURPLUS_SHARES, LOCAL_PIX_SUITE)?,
                        commitments,
                        HoprPixDepositData::default(),
                    ),
                )
                .await;

            let session_alive = mgr.active_sessions().contains(&session_id);

            bob_sender.close_channel();
            bob_handle.await??;

            Ok(Outcome {
                result,
                session_errors: session_errors.load(Ordering::Relaxed),
                session_alive,
            })
        }

        // One over the default cap is rejected...
        let over_cap = offer_batch(
            DEFAULT_MAX_SSAS_PER_SSA_REQUEST,
            DEFAULT_MAX_SSAS_PER_SSA_REQUEST as u32 + 1,
        )
        .await?;
        assert!(
            matches!(
                over_cap.result,
                Err(TransportSessionError::Manager(SessionManagerError::Unacceptable(_)))
            ),
            "an over-cap SsaRequest must be rejected, got {:?}",
            over_cap.result
        );
        // ...and the refusal must be reported rather than left for the Exit to infer from its own
        // deposit timeout minutes later, and must not leave a Session behind that can never make PIX
        // progress.
        assert_eq!(
            1, over_cap.session_errors,
            "a refused SsaRequest must send exactly one SessionError back to the Exit"
        );
        assert!(
            !over_cap.session_alive,
            "a refused SsaRequest must tear down the Entry's half of the Session"
        );

        // ...and the very same batch is accepted once the cap is raised to admit it, proving the
        // rejection is the configured cap talking and not some other validation.
        let raised = DEFAULT_MAX_SSAS_PER_SSA_REQUEST + 1;
        let accepted = offer_batch(raised, raised as u32).await?;
        assert!(
            accepted.result.is_ok(),
            "a batch at the configured cap of {raised} must be accepted, got {:?}",
            accepted.result
        );
        assert_eq!(
            0, accepted.session_errors,
            "an accepted batch must not send a SessionError"
        );
        assert!(
            accepted.session_alive,
            "an accepted batch must leave the Session running"
        );

        Ok(())
    }

    /// A batch with one undecodable member must publish nothing at all.
    ///
    /// `handle_ssa_request` used to generate, send and emit per entry, deciding each member's fate
    /// before looking at the next. A batch whose *second* exit commitment failed to decode had
    /// therefore already sent the first member's `SsaCommit` burst and emitted its `ReadyToDeposit`
    /// — an instruction to put money on chain — for a request that is then rejected as a whole. The
    /// Exit has no cycle to spend that deposit against, and the deposit key needs its half.
    ///
    /// The first member is valid and the second is not, so a per-entry implementation passes the
    /// "batch is refused" half of this test while still leaking the first deposit.
    #[test_log::test(tokio::test)]
    async fn a_batch_with_one_undecodable_commitment_publishes_nothing() -> anyhow::Result<()> {
        use std::collections::BTreeMap;

        use futures::FutureExt;
        use hopr_crypto_packet::prelude::HoprPixGroupElement;
        use hopr_protocol_pix::{PixGroup, SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        let (pix_toolbox, pix_events) = PixToolbox::new(
            SsaShareGenerator::new(SsaGeneratorConfig {
                polynomials_per_ssa: 2,
                threshold: 2,
                surplus_shares: 1,
            })
            .into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        // `SsaCommit` is the observable half of "a commitment was published"; `SessionError` is the
        // refusal. Both are counted, because the bug shows up as the two happening together.
        let commits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let session_errors = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (commits_tx, errors_tx) = (commits.clone(), session_errors.clone());
        let mut bob_transport = MockMsgSender::new();
        bob_transport.expect_send_message().returning(move |_, data| {
            if crate::testing::msg_type(&data, StartProtocolDiscriminants::SsaCommit) {
                commits_tx.fetch_add(1, Ordering::Relaxed);
            }
            if crate::testing::msg_type(&data, StartProtocolDiscriminants::SessionError) {
                errors_tx.fetch_add(1, Ordering::Relaxed);
            }
            Box::pin(async { Ok(()) })
        });

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        let valid = HoprPixGroupElement::try_from(PixGroup::<HoprPixSpec>::default().to_bytes().as_ref())
            .expect("identity element must be valid");
        // All-ones is not a compressed point on the curve, so it fails before the subgroup check.
        // The length is taken from the type rather than written out: the group representation is
        // curve-dependent, and a literal silently becomes a *length* rejection under a different
        // `HoprPixSpec` — which is a different code path from the one under test.
        // Spelled out because the group repr is a `hybrid_array::Array` with several `AsRef` impls.
        let repr_len = AsRef::<[u8]>::as_ref(&PixGroup::<HoprPixSpec>::default().to_bytes()).len();
        let garbage =
            HoprPixGroupElement::try_from(vec![0xffu8; repr_len].as_slice()).expect("length must be accepted");
        assert!(
            garbage.try_into_pix_group().is_err(),
            "the test fixture must actually be undecodable"
        );

        let commitments: BTreeMap<_, _> = BTreeMap::from([
            (SsaIndex::new(100).expect("non-zero"), valid),
            (SsaIndex::new(101).expect("non-zero"), garbage),
        ]);

        let result = mgr
            .handle_ssa_request(
                alice_pseudonym,
                SsaServerCommitmentMessage::new(
                    alice_pseudonym,
                    PixParams::try_new(2, 2, TEST_SURPLUS_SHARES, LOCAL_PIX_SUITE)?,
                    commitments,
                    HoprPixDepositData::default(),
                ),
            )
            .await;

        let session_alive = mgr.active_sessions().contains(&alice_pseudonym);

        // Drain the mock transport before counting: `mock_packet_planning` delivers on a spawned
        // task, so a count taken here would race the sends rather than observe them.
        bob_sender.close_channel();
        bob_handle.await??;

        assert!(
            matches!(
                result,
                Err(TransportSessionError::Manager(SessionManagerError::Unacceptable(_)))
            ),
            "an undecodable member must be refused, got {result:?}"
        );
        assert_eq!(
            0,
            commits.load(Ordering::Relaxed),
            "no SsaCommit may be sent for a batch that is refused as a whole"
        );
        assert_eq!(
            1,
            session_errors.load(Ordering::Relaxed),
            "the refusal must be reported to the Exit"
        );
        assert!(
            !session_alive,
            "an undecodable commitment is terminal, like every other unacceptable-parameter case"
        );

        // The point of the test: not one deposit instruction escaped for the valid first member.
        pin_mut!(pix_events);
        assert!(
            pix_events.next().now_or_never().flatten().is_none(),
            "no ReadyToDeposit may be emitted for a batch that is refused as a whole"
        );

        Ok(())
    }

    /// The Start protocol ingress channel must be sized for the worst-case commitment burst of a
    /// single SSA cycle, not for the number of sessions.
    ///
    /// A dropped `SsaCommit` is unrecoverable — there is no retransmission — so the queue has to
    /// absorb the whole commitment set a cycle can deliver, capped by the polynomial ceiling
    /// `check_pix_params` enforces.
    #[test]
    fn start_protocol_channel_is_sized_for_the_worst_case_commitment_burst() {
        let cfg = SessionManagerConfig::default();
        let capacity = start_protocol_channel_capacity(&cfg);

        // Number of commitments implied by the largest quota this node accepts, clamped by the
        // number of polynomials it would actually admit.
        let commitments =
            (*cfg.pix_config.quota_range.end() / HoprPacket::PAYLOAD_SIZE as u64).min(MAX_POLYS_PER_SSA as u64);
        let min_expected = commitments.div_ceil(MIN_COMMITMENTS_PER_SSA_COMMIT_MSG as u64) as usize;

        assert!(
            capacity >= min_expected + cfg.maximum_sessions,
            "capacity {capacity} must cover the {min_expected}-message commitment burst plus room for {} concurrent \
             session setups",
            cfg.maximum_sessions
        );

        // The commitment burst must be the term that was added, not incidental slack: the old
        // sizing was `maximum_sessions + 10` and carried no PIX component at all.
        assert_eq!(
            min_expected,
            capacity - cfg.maximum_sessions - START_PROTOCOL_CHANNEL_RESERVE,
            "the PIX commitment burst must be an explicit component of the capacity"
        );
        assert!(
            min_expected > 0,
            "a non-zero accepted quota must imply commitment messages"
        );

        // Guards against a units mistake (e.g. counting bytes rather than messages) turning this
        // into a multi-gigabyte ring allocation.
        assert!(capacity < 1_000_000, "capacity {capacity} is implausibly large");
    }

    /// `quota_range` is operator-settable and the capacity it feeds is *reserved*, not merely
    /// enforced — `crossfire`'s array flavour pre-allocates every slot. So the derivation must be
    /// bounded independently of the configured quota.
    ///
    /// Regression test: an unclamped derivation asked for ~4.2e8 slots here (77 GB), which aborted
    /// the whole unit test binary with an allocation failure rather than failing one assertion. The
    /// sizing test above never caught it because it only exercises the default config.
    #[test]
    fn start_protocol_channel_capacity_is_bounded_for_any_quota_range() {
        // The commitment term saturates once the quota admits more commitments than there are
        // polynomials, so this is the largest value it can ever take.
        let saturated = (MAX_POLYS_PER_SSA as u64).div_ceil(MIN_COMMITMENTS_PER_SSA_COMMIT_MSG as u64) as usize;

        for quota_end in [10_u64.pow(13), u64::MAX] {
            let cfg = SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    quota_range: 0..=quota_end,
                    ..Default::default()
                },
                ..Default::default()
            };
            let capacity = start_protocol_channel_capacity(&cfg);

            assert_eq!(
                capacity,
                saturated + cfg.maximum_sessions + START_PROTOCOL_CHANNEL_RESERVE,
                "quota_range end {quota_end} must not grow the commitment term past the polynomial ceiling"
            );
        }

        // The session term is reserved for exactly the same reason, and `maximum_managed_sessions`
        // validates up to 100 000 — so it needs its own ceiling, not just the commitment one.
        let cfg = SessionManagerConfig {
            maximum_sessions: 100_000,
            ..Default::default()
        };
        assert_eq!(
            start_protocol_channel_capacity(&cfg),
            saturated + MAX_CONCURRENT_START_EXCHANGES + START_PROTOCOL_CHANNEL_RESERVE,
            "a large session limit must not grow the pre-allocated ring past the handshake ceiling"
        );

        // The batch factor is reserved too: a batch of N draws N cycles' commitment sets into this
        // one channel, and a dropped `SsaCommit` is unrecoverable. It must scale the commitment term
        // and nothing else, and must itself be bounded by `MAX_SSA_BATCH_SIZE` even when the config
        // was never clamped by `SessionManager::new`.
        let batched = |ssas_per_request| {
            start_protocol_channel_capacity(&SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    quota_range: 0..=u64::MAX,
                    supervision: SupervisorConfig {
                        ssas_per_request,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
        };
        let unbatched = SessionManagerConfig::default();
        for ssas_per_request in [1, 2, MAX_SSA_BATCH_SIZE] {
            assert_eq!(
                batched(ssas_per_request),
                saturated * ssas_per_request + unbatched.maximum_sessions + START_PROTOCOL_CHANNEL_RESERVE,
                "the commitment term must scale with ssas_per_request = {ssas_per_request}"
            );
        }
        assert_eq!(
            batched(MAX_SSA_BATCH_SIZE + 1),
            batched(MAX_SSA_BATCH_SIZE),
            "an unclamped ssas_per_request must not inflate the pre-allocated ring"
        );
        assert_eq!(
            batched(0),
            batched(1),
            "a zero ssas_per_request must not collapse the commitment term"
        );
    }

    /// Verifies that once the exit/responder (Bob) has set up the SSA state, delivering coefficient
    /// commits for all polynomials causes the PIX event stream to emit `DepositNeeded`.
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` configured for `polynomials_per_ssa=2, threshold=2,
    ///    surplus_shares=1`. Alice's session initiation is processed normally.
    /// 2. The exit has already registered an exit commitment from `handle_incoming_session_initiation`.
    /// 3. Coefficient 0 (constant terms across all polynomials) is delivered via `handle_ssa_commit` using identity
    ///    group elements as dummy commitments.
    /// 4. Coefficient 1 (linear terms) is delivered similarly.
    /// 5. After the second coefficient delivery, Bob's PIX event stream emits `DepositNeeded` with the correct `SsaId`
    ///    and `quota_per_ssa` matching `pix_params_to_quota(2, 2)`.
    /// 6. The event is received within a 2-second timeout.
    #[test_log::test(tokio::test)]
    async fn exit_receives_ssa_commits_and_emits_deposit_needed_event() -> anyhow::Result<()> {
        use std::collections::HashMap;

        use hopr_crypto_packet::prelude::{HoprPixCommitmentProof, HoprPixGroupElement};
        use hopr_protocol_pix::{
            Field, PixGroup, PixScalar, PolynomialIndex, SsaCommitmentProof, SsaGeneratorConfig, SsaReconstructor,
            SsaReconstructorConfig, SsaShareGenerator,
        };
        use hopr_protocol_start::StartInitiation;

        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        };

        let (pix_toolbox, pix_events_rx) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        // handle_incoming_session_initiation sends SessionEstablished + SsaRequest.
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox.clone()))?;

        let alice_pseudonym = HoprPseudonym::random();

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        // The exit commitment is registered by the supervisor's opening `RequestSsa`, which the
        // action driver carries out asynchronously — so wait for it rather than assume it.
        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        {
            let share_processor = pix_toolbox.share_processor.clone();
            tokio::time::timeout(Duration::from_secs(1), async {
                while !share_processor.contains_builder(&ssa_id) {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            })
            .await
            .context("the opening SSA request never registered an exit commitment")?;
        }

        // Deliver coefficient 0 (constant terms across all polynomials).
        // Use the identity/infinity group element as a dummy commitment.
        // PixGroup<HoprPixSpec> = k256::ProjectivePoint, which has identity/infinity as all-zero bytes.
        let identity_element = {
            let g: PixGroup<HoprPixSpec> = Default::default();
            HoprPixGroupElement::try_from(g.to_bytes().as_ref()).expect("identity element must be valid")
        };
        let mut coeff_0_map = HashMap::new();
        for poly in 0..2 {
            coeff_0_map.insert(poly as PolynomialIndex, identity_element);
        }
        // The dummy constant terms sum to the identity, whose discrete logarithm is zero, so an
        // honest proof of knowledge over it is constructible — the Exit refuses the cycle without
        // one.
        let zero = <PixScalar<HoprPixSpec> as Field>::ZERO;
        let identity_proof = HoprPixCommitmentProof::from(
            SsaCommitmentProof::<HoprPixSpec>::prove(&ssa_id, &zero, &PixGroup::<HoprPixSpec>::default())
                .expect("identity proof must be constructible"),
        );
        mgr.handle_ssa_commit(
            alice_pseudonym,
            SsaClientCommitmentMessage {
                session_id: alice_pseudonym,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 0,
                commitment_proof: Some(identity_proof),
                coefficient_commitments: coeff_0_map,
            },
        )
        .await?;

        // Deliver coefficient 1 (linear terms across all polynomials).
        let mut coeff_1_map = HashMap::new();
        for poly in 0..2 {
            coeff_1_map.insert(poly as PolynomialIndex, identity_element);
        }
        mgr.handle_ssa_commit(
            alice_pseudonym,
            SsaClientCommitmentMessage {
                session_id: alice_pseudonym,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 1,
                commitment_proof: None,
                coefficient_commitments: coeff_1_map,
            },
        )
        .await?;

        // The first coefficient commitment should trigger DepositNeeded.
        pin_mut!(pix_events_rx);
        let event = tokio::time::timeout(std::time::Duration::from_secs(2), pix_events_rx.next())
            .await
            .map_err(|e| anyhow::anyhow!("timeout waiting for pix event: {e}"))?
            .ok_or_else(|| anyhow::anyhow!("pix_events_rx closed without emitting an event"))?;

        assert!(matches!(
            event,
            HoprSessionOutPixEvent::DepositNeeded(AgreedSsaQuota { ssa_id: ref received_ssa_id, .. }, _)
            if received_ssa_id == &ssa_id
        ));

        let HoprSessionOutPixEvent::DepositNeeded(quota, _) = event else {
            unreachable!();
        };
        assert_eq!(quota.quota_per_ssa, pix_params_to_quota(&small_pix_params()));

        bob_sender.close_channel();
        bob_handle.await??;

        Ok(())
    }

    /// Verifies that a PIX Session whose Entry never answers the `SsaRequest` is closed when the
    /// commitment deadline expires.
    ///
    /// This used to be spelled as a deposit-timeout test, with `max_ssa_delivery_time: 0` and a
    /// 50 ms `max_deposit_wait`. A zero delivery time makes the *commitment* deadline fire
    /// immediately, so the assertion held whatever the deposit deadline did — it would have passed
    /// with that deadline removed entirely. The deposit deadline cannot be reached from this
    /// fixture at all: arming it needs a `CommitmentVerified`, and a mock transport has no Entry to
    /// send one. That path is covered end-to-end by `hopr-lib`'s `deposit_timeout_closes_session`.
    ///
    /// So `max_deposit_wait` is set far out of reach here, leaving the commitment deadline as the
    /// only thing that can close the Session.
    #[test_log::test(tokio::test)]
    async fn session_is_closed_when_the_entry_never_commits() -> anyhow::Result<()> {
        use std::time::Duration;

        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        };

        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                supervision: SupervisorConfig {
                    // Short, so the deadline under test fires quickly.
                    max_ssa_delivery_time: Duration::from_millis(50),
                    // Far out of reach, so it cannot be what closes the Session.
                    max_deposit_wait: Duration::from_secs(3600),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        });

        let mut bob_transport = MockMsgSender::new();
        // handle_incoming_session_initiation sends SessionEstablished + SsaRequest (2 messages).
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox.clone()))?;

        let alice_pseudonym = HoprPseudonym::random();

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        // Session is active after establishment.
        assert_eq!(vec![alice_pseudonym], mgr.active_sessions());

        // The supervisor arms the commitment deadline once the `SsaRequest` goes out and closes the
        // Session 50 ms later; the teardown that follows is asynchronous.
        tokio::time::timeout(Duration::from_secs(2), async {
            while mgr.num_active_sessions() > 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("session should be closed once the commitment deadline expires")?;

        assert!(mgr.active_sessions().is_empty());

        bob_sender.close_channel();
        bob_handle.await??;

        Ok(())
    }

    /// The surplus an Entry offers must survive the round trip: stored by the Exit, then echoed
    /// back in the `SsaRequest` it sends.
    ///
    /// It is the one negotiated value with no other consumer — it is not part of the priced quota,
    /// so no quota check would notice it going missing, and the Exit does not act on it yet. Every
    /// other dimension is pinned several times over by the checks around it.
    #[test_log::test(tokio::test)]
    async fn exit_stores_and_echoes_the_offered_surplus() -> anyhow::Result<()> {
        use hopr_protocol_pix::SsaReconstructorConfig;
        use hopr_protocol_start::StartInitiation;

        // Distinct from every default, so a value read from local config instead of the wire is
        // visibly wrong rather than accidentally right.
        const OFFERED_SURPLUS: u8 = 37;

        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(SsaGeneratorConfig {
                polynomials_per_ssa: 2,
                threshold: 2,
                surplus_shares: TEST_SURPLUS_SHARES,
            })
            .into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        });

        let echoed: Arc<std::sync::Mutex<Vec<PixParams>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let echoed_clone = echoed.clone();
        let mut bob_transport = MockMsgSender::new();
        bob_transport.expect_send_message().returning(move |_, data| {
            if let Ok(HoprStartProtocol::SsaRequest(req)) = HoprStartProtocol::try_from(data.data)
                && let Ok(params) = req.dimensions()
            {
                echoed_clone.lock().unwrap().push(params);
            }
            Box::pin(async { Ok(()) })
        });

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: pix_additional_data(2, 2, OFFERED_SURPLUS),
            },
        )
        .await?;

        let expected = PixParams::try_new(2, 2, OFFERED_SURPLUS, LOCAL_PIX_SUITE)?;

        let slot = mgr.sessions.get(&alice_pseudonym).context("session must exist")?;
        let ssa_state = slot.current_ssa_state.get().context("pix state must be set")?;
        assert_eq!(
            expected, ssa_state.params,
            "the Exit must keep the surplus the Entry offered, not its own"
        );

        // The echo does not go out inline: the supervisor's opening `RequestSsa` is carried out by
        // the action driver spawned once the Session is published, which is what keeps
        // `SessionEstablished` ahead of `SsaRequest` on the wire.
        tokio::time::timeout(Duration::from_secs(1), async {
            while echoed.lock().unwrap().is_empty() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("the supervisor's opening SsaRequest never reached the wire")?;

        bob_sender.close_channel();
        bob_handle.await??;

        let echoed = echoed.lock().unwrap().clone();
        assert_eq!(
            vec![expected],
            echoed,
            "the SsaRequest must echo back exactly what was offered"
        );

        Ok(())
    }

    /// Verifies that `check_pix_params` rejects out-of-bounds parameters that pass the
    /// quota-range check but exceed the protocol limits.
    ///
    /// This is a regression test for the incentive-bypass fix (round-1 finding #1).
    #[test_log::test(tokio::test)]
    async fn check_pix_params_must_reject_invalid_bounds() -> anyhow::Result<()> {
        let mgr =
            SessionManager::<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    quota_range: 0..=10_000_000_000_000,
                    ..Default::default()
                },
                ..Default::default()
            });

        let offer = |additional_data: u64| StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse().unwrap())),
            capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
            additional_data,
        };
        // Packed by hand rather than through `pix_additional_data`, because that helper refuses to
        // build the very values under test. The layout it mirrors is pinned in `PixParams`' own
        // tests. The suite is this build's, so that the dimension cases below fail on their
        // dimensions rather than on a curve mismatch — that rejection has its own test.
        let packed = |polys: u16, shares: u8, surplus: u8| {
            ((LOCAL_PIX_SUITE as u64) << 62)
                | ((polys as u64) << 48)
                | ((shares as u64) << 40)
                | ((surplus as u64) << 32)
        };

        // polys_per_ssa > MAX_POLYS_PER_SSA (16192), and zero, with a valid quota -> should reject
        for polys in [0, MAX_POLYS_PER_SSA + 1, u16::MAX] {
            assert!(
                mgr.check_pix_params(&offer(packed(polys, 128, 0))).is_none(),
                "should reject polys_per_ssa {polys}"
            );
        }

        // shares_per_poly < MIN_POLY_THRESHOLD with a valid quota -> should reject. There is no
        // matching upper-bound case: the threshold is a byte on the wire, so `MAX_POLY_THRESHOLD`
        // cannot be exceeded by anything a peer is able to send.
        for shares in [0, 1] {
            assert!(
                mgr.check_pix_params(&offer(packed(8192, shares, 0))).is_none(),
                "should reject shares_per_poly {shares}"
            );
        }

        // Valid params should still be accepted, and arrive intact — including the surplus, which
        // no other check looks at and which would therefore be free to go missing.
        let accepted = mgr
            .check_pix_params(&offer(packed(8192, 128, 37)))
            .context("should accept valid params")?;
        assert_eq!(PixParams::try_new(8192, 128, 37, LOCAL_PIX_SUITE)?, accepted);

        // Every surplus a byte can hold is legal.
        for surplus in [0, 1, u8::MAX] {
            assert_eq!(
                Some(PixParams::try_new(8192, 128, surplus, LOCAL_PIX_SUITE)?),
                mgr.check_pix_params(&offer(packed(8192, 128, surplus)))
            );
        }

        Ok(())
    }

    /// The Exit refuses a client whose PIX curve suite is not the one this node was built for.
    ///
    /// The curve is a build-time property on both sides and is not negotiated, so the only two
    /// outcomes are "same suite" and "refused". This matters because it happens *before* the Exit
    /// commits: every later PIX field — each coefficient commitment, the proof of knowledge — is
    /// sized by the curve, so a mismatch that got past here would surface as undecodable Start
    /// traffic on a Session both sides believed they had established.
    ///
    /// The quota range is wide open, so nothing but the suite can be doing the rejecting.
    #[test_log::test(tokio::test)]
    async fn check_pix_params_must_refuse_a_foreign_curve_suite() -> anyhow::Result<()> {
        let mgr =
            SessionManager::<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    quota_range: 0..=10_000_000_000_000,
                    ..Default::default()
                },
                ..Default::default()
            });

        let offer = |suite: hopr_protocol_pix::PixSuite| StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse().unwrap())),
            capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
            additional_data: PixParams::try_new(8192, 128, 37, suite)
                .expect("test dimensions must be valid")
                .into_additional_data(0),
        };

        let foreign = match LOCAL_PIX_SUITE {
            hopr_protocol_pix::PixSuite::BabyJubJub => hopr_protocol_pix::PixSuite::Secp256k1,
            hopr_protocol_pix::PixSuite::Secp256k1 => hopr_protocol_pix::PixSuite::BabyJubJub,
        };

        assert!(
            mgr.check_pix_params(&offer(foreign)).is_none(),
            "a client offering {foreign} must be refused by a {LOCAL_PIX_SUITE} Exit"
        );
        assert!(
            mgr.check_pix_params(&offer(LOCAL_PIX_SUITE)).is_some(),
            "and the same dimensions on this build's own curve must still be accepted"
        );

        // A suite identifier no curve claims is refused as well, rather than read as a third curve.
        let unknown = StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse().unwrap())),
            capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
            additional_data: (0b11u64 << 62) | (8192u64 << 48) | (128u64 << 40) | (37u64 << 32),
        };
        assert!(
            mgr.check_pix_params(&unknown).is_none(),
            "an unknown suite identifier must be refused"
        );

        Ok(())
    }

    /// Verifies that dispatching too many `UnverifiableShare` events closes the session.
    #[test_log::test(tokio::test)]
    async fn too_many_unverifiable_shares_closes_session() -> anyhow::Result<()> {
        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        };

        let (pix_toolbox, _pix_events_rx) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(SsaReconstructorConfig::default()).into(),
        );

        let mgr =
            SessionManager::<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>::new(SessionManagerConfig {
                pix_config: IncomingSessionPixConfig {
                    quota_range: 0..=10_000_000_000_000,
                    supervision: SupervisorConfig {
                        max_deposit_wait: Duration::from_secs(1),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            });

        let mut bob_transport = MockMsgSender::new();
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        bob_transport
            .expect_send_message()
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        let _notifications = tokio::spawn(async move {
            pin_mut!(new_session_rx);
            while let Some(_session) = new_session_rx.next().await {}
        });
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox.clone()))?;

        let alice_pseudonym = HoprPseudonym::random();

        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: small_pix_additional_data(),
            },
        )
        .await?;

        // Session is active
        assert_eq!(vec![alice_pseudonym], mgr.active_sessions());

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(1).expect("non-zero"));

        // One more fault than the tolerance allows, which at the shipped tolerance of zero is a
        // single one. Written against the configured limit, and reporting a rising absolute total
        // as the reconstructor does, so that raising the tolerance would not silently turn this
        // into a test of nothing.
        let tolerance = mgr.cfg.pix_config.supervisor_config().max_unverifiable_shares_per_ssa;
        for observed_total in 1..=tolerance + 1 {
            let result = mgr
                .dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShares { ssa_id, observed_total })
                .await;
            // Forwarding succeeds even for the event that closes: the supervisor decides, and it
            // does so after the send has been accepted.
            assert!(result.is_ok(), "dispatch_pix_event should not return an error");
        }

        // The supervisor's `Close` reaches the driver asynchronously, so the teardown it triggers
        // is observed rather than assumed.
        tokio::time::timeout(Duration::from_secs(1), async {
            while mgr.num_active_sessions() > 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .context("session was not closed after too many unverifiable shares")?;

        assert!(mgr.active_sessions().is_empty());

        bob_sender.close_channel();
        bob_handle.await??;

        Ok(())
    }

    /// Verifies that `allocate_session_slot` signals waiters via `.send(())` rather than
    /// dropping the senders. The `Ok(Ok(()))` path in `handle_ssa_request` must be live.
    #[test_log::test(tokio::test)]
    async fn allocate_session_slot_must_signal_waiters_not_cancel_them() -> anyhow::Result<()> {
        let session_id = HoprPseudonym::random();
        let mgr = SessionManager::<UnboundedSender<(DestinationRouting, ApplicationDataOut)>>::new(Default::default());

        // Register a waiter before the slot is allocated
        let (tx, mut rx) = oneshot::channel::<()>();
        mgr.slot_allocated
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .entry(session_id)
            .or_default()
            .push(tx);

        let (session_tx, _) = crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(SESSION_FORWARD_CAPACITY);

        // Allocate the slot — this should signal the waiter
        let guard = mgr.allocate_session_slot(
            session_id,
            SessionSlot {
                session_tx,
                routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(session_id)),
                abort_handles: Default::default(),
                surb_mgmt: Arc::new(BalancerStateValues::from(SurbBalancerConfig::default())),
                surb_estimator: Default::default(),
                current_ssa_state: Default::default(),
                pix_supervisor: Default::default(),
                pix_egress_gate: Default::default(),
                returned_packets: Default::default(),
                cycle_budget: None,
            },
        );
        assert!(guard.is_some(), "slot allocation must succeed");

        // The waiter should receive Some(()) — the direct signal, not Canceled
        assert!(
            matches!(rx.try_recv(), Ok(Some(()))),
            "waiter must be signaled with Ok(()), not canceled"
        );

        guard.unwrap().commit();

        Ok(())
    }
}
