use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex, OnceLock, atomic::Ordering},
    time::{Duration, Instant},
};

use anyhow::anyhow;
use futures::{FutureExt, Sink, SinkExt, StreamExt, TryStreamExt, channel::oneshot, future::AbortHandle};
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
    RawSsaIndex, SsaId, SsaIndex, SsaReconstructor, SsaShareGenerator,
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

/// How many of a session's most recent SSA cycles teardown sweeps.
///
/// Only unrecovered cycles hold reconstructor state — a recovered one removes itself. The live set is
/// bounded by the batch size: pipelining runs at most one batch ahead of the active one, so at most
/// `2 × `[`MAX_SSA_BATCH_SIZE`] indices can be unrecovered at once, which this window covers with
/// room to spare. It is deliberately well above even that, because the cost of being wrong is
/// asymmetric in one direction only: a cycle missed here is still reclaimed by
/// `unused_verifier_lifetime`, whereas sweeping every index a session ever used is unbounded work on
/// the eviction listener, and leaves a tombstone per index behind it.
const SSA_TEARDOWN_SWEEP_WINDOW: u32 = 64;

/// Release reconstructor state for the SSA cycles of a session that may still be live.
///
/// Called from teardown paths so no builder, verifier or liveness entry is stranded. Bounded to the
/// most recent [`SSA_TEARDOWN_SWEEP_WINDOW`] indices: the sweep runs inside the moka eviction
/// listener and in `close_session`, and the index grows with every completed cycle, so an unbounded
/// walk makes teardown cost proportional to how long the session lived.
fn retire_all_live_ssa_cycles(session_id: SessionId, ssa_state: &SessionSsaState, pix_toolbox: &PixToolbox) {
    let current = ssa_state.peek_index().get();
    let oldest = current.saturating_sub(SSA_TEARDOWN_SWEEP_WINDOW - 1).max(1);
    if oldest > 1 {
        trace!(
            %session_id,
            oldest,
            current,
            "sweeping only the most recent ssa cycles at teardown; older ones expire on their own timer"
        );
    }
    for i in oldest..=current {
        if let Some(idx) = SsaIndex::new(i) {
            pix_toolbox.share_processor.retire_ssa(SsaId::new(session_id, idx));
        }
    }
}

#[tracing::instrument(level = "debug", skip(session_data))]
fn close_session(session_id: SessionId, session_data: SessionSlot, reason: ClosureReason) {
    debug!("closing session");

    #[cfg(feature = "telemetry")]
    {
        set_session_state(&session_id, SessionLifecycleState::Closed);
        remove_session_metrics_state(&session_id);
    }

    if reason != ClosureReason::EmptyRead {
        // Closing the data sender will also cause it to close from the read side
        debug!("data tx channel closed on session");
    }

    // Terminate any additional tasks spawned by the Session
    session_data.abort_handles.lock().abort_all();

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
/// dies on the deposit kill switch.
///
/// PIX changed this channel's load from roughly one message per session to the *entire* commitment
/// set of an SSA cycle, chunked into packet-sized messages, plus a reserve for ordinary Start
/// traffic. Batching multiplies that: an Exit that asks for
/// [`ssas_per_request`](IncomingSessionPixConfig::ssas_per_request) SSAs at once gets that many
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
    let ssas_per_request = cfg.pix_config.ssas_per_request.clamp(1, MAX_SSA_BATCH_SIZE) as u64;

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

/// Minimum timeout until an unfinished frame is discarded.
const MIN_FRAME_TIMEOUT: Duration = Duration::from_millis(10);

/// Number of PIX verification failures tolerated before the session is closed.
///
/// Zero: the first failure closes the session.
///
/// A failure now means a whole polynomial's share set did not open its commitment (or a single
/// share that was not a valid field element), not an individual share caught by a per-share
/// Feldman check — those were dropped along with the non-constant coefficient commitments, see
/// [`hopr_protocol_pix::SsaPartCommitment`]. That changes what tolerating failures would buy:
///
/// * A failed polynomial already dooms the whole cycle, because the SSA is the sum of *every* polynomial's constant
///   term. There is no partial recovery to preserve.
/// * A share that fails to reconstruct implies a dishonest or broken Entry — the share arrives inside a
///   Sphinx-authenticated SURB and is decrypted with the key its own acknowledgement challenge fixes, so there is no
///   benign path to a corrupt one.
///
/// Detection is also later than it used to be: the failure surfaces on the `threshold`-th share of
/// the polynomial, so the Exit has already served that many packets. Closing on the first failure
/// is what keeps that exposure at `threshold` packets instead of a multiple of it.
const MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES: usize = 0;

/// Hard ceiling on both SSA batch-size knobs, whatever the configuration says.
///
/// Deliberately far below the wire limit (`StartProtocol::MAX_SSAS_PER_REQUEST`, 27), which only
/// bounds what can be *decoded*. The real cost is paid on both sides of the exchange, and neither is
/// small at the profiled dimensions:
///
/// * Entry: every entry in the batch is a full `new_ssa_commitment` (hundreds of thousands of EC commitments), its own
///   burst of thousands of `SsaCommit` packets, and its own `ReadyToDeposit` — i.e. its own on-chain deposit.
/// * Exit: every entry is a live reconstructor cycle, ≈49 MiB of peak state, held until that cycle recovers. At this
///   ceiling that is ≈1 GB per Session.
///
/// Both [`IncomingSessionPixConfig::ssas_per_request`] and
/// [`SessionManagerConfig::max_ssas_per_ssa_request`] are clamped to `1..=Self` in
/// [`SessionManager::new`], so a programmatically built config that never calls `validate()` cannot
/// exceed it.
pub const MAX_SSA_BATCH_SIZE: usize = 20;

/// Default for [`SessionManagerConfig::max_ssas_per_ssa_request`] — how many SSA commitments an Entry
/// accepts in a single [`SsaServerCommitmentMessage`].
///
/// Pipelining needs at most one cycle in flight ahead of the active one, so 2 leaves room for an Exit
/// batching at the default without turning one inbound packet into an unbounded amount of Entry work
/// and on-chain deposits.
pub const DEFAULT_MAX_SSAS_PER_SSA_REQUEST: usize = 2;

/// Default for [`IncomingSessionPixConfig::ssas_per_request`] — how many SSAs an Exit asks for in a
/// single [`SsaServerCommitmentMessage`].
///
/// One, so that the default configuration produces exactly the unbatched exchange: same wire bytes,
/// same kill-switch deadline, same Start protocol channel capacity.
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
    /// Handle to the process which closes the Session unless the handle is aborted.
    ///
    /// Carries the inner `SsaIndex` value so that each cycle gets its own
    /// independent entry in `AbortableList` — pipelining does not cancel an
    /// earlier cycle's deadline.
    PixKillSwitch(u32),
    /// Handle to the process that awaits PIX deposit for a Session.
    ///
    /// Once the deposit is received, the handle [`SessionHandles::PixKillSwitch`] is aborted.
    /// Carries the inner `SsaIndex` value so that each cycle's awaiter is
    /// independent.
    DepositAwaiter(u32),
}

impl std::fmt::Display for SessionHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ingress => write!(f, "Ingress"),
            Self::KeepAlive => write!(f, "KeepAlive"),
            Self::Balancer => write!(f, "Balancer"),
            Self::PixKillSwitch(idx) => write!(f, "PixKillSwitch({idx})"),
            Self::DepositAwaiter(idx) => write!(f, "DepositAwaiter({idx})"),
        }
    }
}

#[derive(Clone)]
struct SessionSsaState {
    current_index: Arc<std::sync::atomic::AtomicU32>,
    /// Cumulative count of PIX verification failures across all SSA cycles (intentional).
    ///
    /// This is *not* reset per SSA cycle: a steady trickle of 1 error per cycle
    /// should still escalate to session closure, since an unreliable channel is
    /// unlikely to improve on its own. The session closes once the *total* number
    /// across all SSA cycles exceeds `MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES`, which is
    /// currently 0 — so the counter is really a tripwire, kept as a counter so the
    /// tolerance remains a one-constant decision. An `AtomicUsize` is safe here
    /// because duplicates are already rejected by the moka cache (keyed by
    /// `HalfKeyChallenge`) and by `SsaPartBuilder` (keyed by share identifier), and a
    /// failed polynomial reports once and then goes quiet, so nothing double-counts.
    num_errors: Arc<std::sync::atomic::AtomicUsize>,
    /// The dimensions this Session negotiated, as they went on the wire.
    params: PixParams,
    /// Serializes the three call sites of [`SessionManager::request_next_ssa`] so that
    /// `peek_index` / fallible work / `advance_index` is never interleaved.
    request_lock: Arc<hopr_utils::runtime::prelude::Mutex<()>>,
}

impl SessionSsaState {
    pub fn new(params: PixParams) -> Self {
        Self {
            // SSA index starts from 1, not 0.
            current_index: std::sync::atomic::AtomicU32::new(1).into(),
            num_errors: Default::default(),
            params,
            request_lock: Arc::new(hopr_utils::runtime::prelude::Mutex::new(())),
        }
    }

    /// Record an unverifiable share error.
    ///
    /// Returns the cumulative error count after this increment.
    pub fn increment_errors(&self) -> usize {
        self.num_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// Returns the current index without consuming it.
    ///
    /// Use this to inspect the next index before fallible operations;
    /// call [`advance_index`](Self::advance_index) to commit *after* they succeed.
    pub fn peek_index(&self) -> SsaIndex {
        SsaIndex::new(self.current_index.load(std::sync::atomic::Ordering::Relaxed)).expect("ssa index cannot be 0")
    }

    /// Advances the index past a batch of `n` allocated SSAs, returning the new next index.
    ///
    /// Call *after* every fallible step of the request has succeeded, so a failed request keeps its
    /// indices for the retry.
    ///
    /// `n` is expected to be the batch size actually requested, which
    /// [`request_next_ssa`](SessionManager::request_next_ssa) has already shrunk to fit inside `u32`
    /// — hence the panic on overflow rather than a saturating advance: silently reusing an index
    /// would collide with a live cycle.
    pub fn advance_index(&self, n: u32) -> SsaIndex {
        let old = self
            .current_index
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |i| i.checked_add(n))
            .expect("SSA index overflow after u32::MAX cycles");
        SsaIndex::new(old.checked_add(n).expect("just advanced")).expect("non-zero just advanced")
    }

    /// Data quota a single SSA of this Session buys, as per [`pix_params_to_quota`] — the whole
    /// cycle, surplus included.
    #[inline]
    pub const fn quota_per_ssa(&self) -> SsaQuota {
        pix_params_to_quota(&self.params)
    }
}

impl std::fmt::Debug for SessionSsaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionSsaState")
            .field(
                "current_index",
                &self.current_index.load(std::sync::atomic::Ordering::Relaxed),
            )
            .field(
                "num_errors",
                &self.num_errors.load(std::sync::atomic::Ordering::Relaxed),
            )
            .field("polys_per_ssa", &self.params.polys_per_ssa())
            .field("shares_per_poly", &self.params.shares_per_poly())
            .field("surplus_shares", &self.params.surplus_shares())
            .finish()
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
    /// Maximum time to wait for the SSA to be fully committed and delivered to the Exit.
    ///
    /// The Session is allowed to be used unincentivized for `max_deposit_time` + `max_ssa_delivery_time` the deposit
    /// wait time because the Client has to be able to deliver its SSA commitment.
    #[default(Duration::from_secs(20))]
    #[serde(with = "humantime_serde")]
    pub max_ssa_delivery_time: Duration,
    /// Maximum time to wait for the funds to be deposited in the SSA.
    ///
    /// The Session is allowed to be used unincentivized for `max_deposit_time` + `max_ssa_delivery_time` the deposit
    /// wait time because the Client has to be able to deliver its SSA commitment.
    ///
    /// Default is 1 minute.
    #[default(Duration::from_secs(60))]
    #[serde(with = "humantime_serde")]
    pub max_deposit_wait: Duration,
    /// Number of SSAs this Exit asks the Entry to commit to in a single
    /// [`SsaServerCommitmentMessage`].
    ///
    /// Batching amortizes the round trip over several deposit cycles, at the cost of holding that
    /// many live reconstructor cycles at once (≈49 MiB of peak state each at the profiled
    /// dimensions) and fronting that many SSA quotas of unincentivized service before the first
    /// deposit lands. It applies to every request, including the first one at Session establishment.
    ///
    /// The deposit deadline scales with it: each cycle in a batch gets a kill switch at
    /// `ssas_per_request × (max_deposit_wait + max_ssa_delivery_time)`, and the deposit awaiter's
    /// timeout is scaled by the same factor, so a batch is judged as a whole rather than per cycle.
    ///
    /// **This must not exceed the peer Entry's `max_ssas_per_request`.** There is no negotiation of
    /// the batch size — `StartSession.additional_data` is fully allocated (PIX dimensions in the
    /// upper 32 bits, SURB balancer target in the lower 32), so the Entry has no way to advertise its
    /// cap and this Exit has no way to learn it. An Entry that considers the batch too large rejects
    /// the whole request and replies with a
    /// [`StartErrorReason::UnacceptablePixParams`] `SessionError`, which closes the Session on both
    /// sides within about a round trip — see `refuse_ssa_request`. Every Session is still lost, so
    /// raising this requires raising `pix.max_ssas_per_request` on every Entry that will use this
    /// Exit; the reply only means the failure is immediate and reported rather than showing up as a
    /// deposit timeout after the whole `ssas_per_request`-scaled window has elapsed.
    ///
    /// Clamped to `1..=`[`MAX_SSA_BATCH_SIZE`] in [`SessionManager::new`].
    ///
    /// Defaults to [`DEFAULT_SSAS_PER_SSA_REQUEST`] (1), which reproduces the unbatched exchange
    /// exactly.
    #[default(DEFAULT_SSAS_PER_SSA_REQUEST)]
    pub ssas_per_request: usize,
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
    /// [`IncomingSessionPixConfig::ssas_per_request`] for why a mismatch loses every Session.
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
/// Once the PIX parameters are accepted, the Exit calls `request_next_ssa`
/// to create a new SSA commitment via the server-side [`SsaReconstructor`]. This produces an
/// *Exit commitment* (a group element) that is sent back to the Entry as a
/// [`SsaServerCommitmentMessage`].
///
/// One message can carry a whole batch of them:
/// [`IncomingSessionPixConfig::ssas_per_request`] SSAs at contiguous indices, sharing the single
/// `params` field, since every SSA in a Session uses the same negotiated dimensions. The Entry caps
/// what it will accept at [`SessionManagerConfig::max_ssas_per_ssa_request`], and rejects an over-cap
/// request in full while replying with an `UnacceptablePixParams` [`StartErrorType`] so the Exit does
/// not have to infer the refusal from its own deposit timeout. The default is a batch of one, which is
/// byte-for-byte the unbatched exchange.
///
/// The Exit also installs a *PIX kill switch* per requested index — one shared deadline of
/// `ssas_per_request × (max_deposit_wait + max_ssa_delivery_time)`. Scaling it by the batch size is
/// what lets an Entry work through a batch in order; any single deposit may be late as long as the
/// batch lands inside the window. If a deposit is still missing when the deadline passes, the Session
/// is closed with `ClosureReason::UnrealizedDeposit`.
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
/// A *deposit awaiter* task waits for the deposit confirmation. Once confirmed, the PIX
/// kill switch is aborted. If the deposit times out, the kill switch closes the Session. The awaiter's
/// own timeout is scaled by `ssas_per_request` too, and has to be: it is the only thing that aborts
/// the kill switch, so an awaiter that gave up before the widened deadline would let a
/// legitimately-late deposit go unobserved and the Session be closed for an unrealized deposit that
/// was in fact realized.
///
/// ### 5. SSA Collection, Recovery and Pipelining
///
/// As the Entry sends return-path SURBs during the Session, each SURB can carry a PIX
/// share generated from the client's polynomial set. The Exit's [`SsaReconstructor`]
/// collects these shares.
///
/// When the reconstructor reaches the *early recovery threshold* (≈85%), an
/// [`HoprSessionInPixEvent::SsaAlmostRecovered`] event fires, which triggers
/// `request_next_ssa` for the next SSA index — pipelining the costly
/// commitment exchange with the tail of the share collection for the current SSA.
///
/// Once fully recovered, [`HoprSessionInPixEvent::SsaRecovered`] fires, allowing the
/// Exit to unlock and redeem the deposited funds. The deposit awaiter for the next SSA
/// replaces the kill switch aborted for the previous one.
///
/// ### 6. Unverifiable Shares
///
/// Shares are not checked individually. Once a polynomial has collected `threshold` of them,
/// the reconstructor interpolates its constant term and compares it against the commitment; if
/// they disagree, at least one of those shares did not come from the committed polynomial and an
/// [`HoprSessionInPixEvent::UnverifiableShare`] event fires. `MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES`
/// is 0, so the first such event closes the Session — the cycle is already unrecoverable, and
/// closing immediately caps what a malicious Entry is served at `threshold` packets.
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
}

impl<S> Clone for SessionManager<S> {
    fn clone(&self) -> Self {
        Self {
            session_initiations: self.session_initiations.clone(),
            session_notifiers: self.session_notifiers.clone(),
            start_protocol_tx: self.start_protocol_tx.clone(),
            active_sessions: self.active_sessions.clone(),
            sessions: self.sessions.clone(),
            cfg: self.cfg.clone(),
            msg_sender: self.msg_sender.clone(),
            pix_toolbox: self.pix_toolbox.clone(),
            slot_allocated: Arc::clone(&self.slot_allocated),
        }
    }
}

fn session_config(cfg: &SessionManagerConfig, capabilities: Capabilities) -> HoprSessionConfig {
    HoprSessionConfig {
        capabilities,
        frame_mtu: cfg.frame_mtu,
        frame_timeout: cfg.max_frame_timeout,
        max_buffered_segments: cfg.max_buffered_segments,
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
        cfg.pix_config.ssas_per_request = cfg.pix_config.ssas_per_request.clamp(1, MAX_SSA_BATCH_SIZE);

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_ACTIVE_SESSIONS.set(0.0);

        let active_sessions: Arc<std::sync::atomic::AtomicUsize> = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let active_sessions_for_listener = active_sessions.clone();

        let msg_sender = Arc::new(OnceLock::new());
        let initiation_timeout =
            2 * initiation_timeout_max_one_way(cfg.initiation_timeout_base, RoutingOptions::MAX_INTERMEDIATE_HOPS);
        let pix_toolbox: Arc<OnceLock<PixToolbox>> = Arc::new(OnceLock::new());
        let pix_toolbox_eviction = pix_toolbox.clone();
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
                            if let (Some(ssa_state), Some(pix_toolbox)) =
                                (entry.current_ssa_state.get(), pix_toolbox_eviction.get())
                            {
                                retire_all_live_ssa_cycles(*session_id.as_ref(), ssa_state, pix_toolbox);
                            }
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
            cfg,
            slot_allocated: Arc::new(Mutex::new(HashMap::new())),
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
                            // Release reconstructor state for all live SSA cycles
                            // before closing.
                            if let (Some(ssa_state), Some(pix_toolbox)) =
                                (session_data.current_ssa_state.get(), myself.pix_toolbox.get())
                            {
                                retire_all_live_ssa_cycles(session_id, ssa_state, pix_toolbox);
                            }
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
                        session_config(&self.cfg, cfg.capabilities),
                        (
                            reduced_surb_scoring_sender,
                            session_rx.inspect(move |_| {
                                // Received packets = SURB consumption estimate
                                // The received packets always consume a single SURB.
                                surb_estimator_for_rx
                                    .consumed
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
                        session_config(&self.cfg, cfg.capabilities),
                        (reduced_surb_sender, session_rx),
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

    async fn request_next_ssa(
        &self,
        session_id: SessionId,
        slot: SessionSlot,
        expected_ssa_index: Option<SsaIndex>,
    ) -> errors::Result<()> {
        let pix_toolbox = self.pix_toolbox.get().cloned().ok_or(SessionManagerError::NotStarted)?;
        // Clone for the deposit-timeout kill switch below; `pix_toolbox` itself is
        // moved into the blocking commitment task.
        let pix_toolbox_killswitch = pix_toolbox.clone();
        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        let current_ssa_state = slot.current_ssa_state.get().ok_or(SessionManagerError::Other(anyhow!(
            "cannot request new ssa on a session without pix state"
        )))?;

        // Serialize the three call sites (establishment, SsaAlmostRecovered, SsaRecovered)
        // so that peek_index → fallible work → advance_index is never interleaved.
        // The 30-second timeout bounds waiting behind a genuinely slow in-flight request
        // (blocked commitment work or stalled send_via_msg_sender), returning an error
        // instead of waiting indefinitely.
        let _guard = current_ssa_state
            .request_lock
            .lock()
            .timeout(futures_time::time::Duration::from(Duration::from_secs(30)))
            .await
            .map_err(|_| SessionManagerError::Other(anyhow!("request_next_ssa lock timed out")))?;

        // Stale-cycle guard: if this request was triggered by a PIX event (e.g.
        // SsaAlmostRecovered), verify the index hasn't already been advanced by a
        // concurrent handler. Under the lock this is race-free.
        if let Some(expected) = expected_ssa_index {
            let current = current_ssa_state.current_index.load(Ordering::Relaxed);
            if expected.get() != current.saturating_sub(1) {
                trace!(%session_id, %expected, "stale SSA event — index already advanced");
                return Ok(());
            }
        }

        // Peek at the next SSA index *before* fallible operations so that a failed
        // commitment generation or send does not permanently consume it.
        let first_ssa_index = current_ssa_state.peek_index();

        // Indices this request allocates: `first .. first + ssas_per_request`, contiguous. The Entry
        // only requires strict monotonicity (gaps are legal), but there is no reason to leave any.
        //
        // `checked_add` rather than `+`: a wrapped index would be a *reused* index, colliding with a
        // live cycle, and in release builds nothing would say so. Overflow truncates the batch
        // instead, so `batch_size` below is what was actually allocated rather than what was asked
        // for. Reaching this needs 2^32 cycles in one Session — the index is exhausted either way at
        // that point, and `advance_index` reports it — but truncating keeps the failure loud.
        let requested = self.cfg.pix_config.ssas_per_request.clamp(1, MAX_SSA_BATCH_SIZE);
        let ssa_indices = (0..requested as RawSsaIndex)
            .map_while(|offset| first_ssa_index.get().checked_add(offset).and_then(SsaIndex::new))
            .collect::<Vec<_>>();
        let batch_size = ssa_indices.len();
        if batch_size < requested {
            warn!(
                %session_id, %first_ssa_index, batch_size, requested,
                "ssa batch truncated — index space exhausted"
            );
        }

        let (polys_per_ssa, shares_per_poly) = (
            current_ssa_state.params.polys_per_ssa(),
            current_ssa_state.params.shares_per_poly(),
        );
        let indices_for_commitments = ssa_indices.clone();
        // One blocking task for the whole batch rather than one per SSA: each commitment is a single
        // random scalar and one generator multiplication, so the per-task overhead would dominate.
        //
        // Guarded, because registering the batch precedes the one fallible send that publishes it. A
        // failure anywhere from here to the end of the send — including partway through the batch —
        // drops the guards, which releases every registration *without* writing a resurrection
        // tombstone. That distinction is what makes the retry work: the index is deliberately not
        // advanced on failure, so the next attempt reuses these very indices and would otherwise be
        // refused as a `DuplicateCommitment` by the registrations this attempt stranded.
        let (exit_commitments, commitment_guards) = hopr_utils::parallelize::cpu::spawn_blocking(
            move || {
                let mut commitments = Vec::with_capacity(indices_for_commitments.len());
                let mut guards = Vec::with_capacity(indices_for_commitments.len());
                for ssa_index in indices_for_commitments {
                    let (commitment, guard) = pix_toolbox.share_processor.new_guarded_exit_commitment(
                        SsaId::new(session_id, ssa_index),
                        polys_per_ssa as usize,
                        shares_per_poly as usize,
                    )?;
                    commitments.push((ssa_index, HoprPixGroupElement(commitment.to_bytes())));
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
            %session_id, ?current_ssa_state, batch_size, %first_ssa_index,
            "generated exit commitments for the SSA batch"
        );

        // Set up the kill switches before sending the SSA request so there is no
        // window where the commitments are in flight but no timeout is installed.
        //
        // The whole batch shares one deadline, scaled by its size: the Entry has to deliver
        // `batch_size` commitment sets and make `batch_size` deposits before any of them can be
        // considered late, and holding each cycle to an unscaled window would kill the Session for a
        // peer that is working through the batch in order. Any single deposit may therefore arrive
        // late, as long as the batch as a whole lands inside the window.
        let session_deadline = std::time::Instant::now()
            + batch_size as u32 * (self.cfg.pix_config.max_deposit_wait + self.cfg.pix_config.max_ssa_delivery_time);
        {
            // Per-index keys, so each cycle's deadline is an independent entry that its own deposit
            // awaiter can abort without touching the others.
            let mut abort_handles = slot.abort_handles.lock();
            for &ssa_index in &ssa_indices {
                let session_cache = self.sessions.clone();
                let active_sessions_clone = self.active_sessions.clone();
                let pix_toolbox_killswitch = pix_toolbox_killswitch.clone();
                // Snapshot the SSA state before the closure so peek_index remains
                // available after close_session consumes the slot.
                let ssa_state_snapshot = current_ssa_state.clone();
                abort_handles.insert(
                    SessionHandles::PixKillSwitch(ssa_index.get()),
                    hopr_utils::spawn_as_abortable!(futures_time::task::sleep_until(session_deadline.into()).then(
                        move |_| async move {
                            if let Some(session_slot) = session_cache.remove(&session_id) {
                                active_sessions_clone.fetch_sub(1, Ordering::Relaxed);
                                close_session(session_id, session_slot, ClosureReason::UnrealizedDeposit);
                                // Release reconstructor state for all live cycles for this
                                // session, not just the timed-out index.  Pipelining and batching
                                // may have created earlier unpaid cycles whose builders/verifiers
                                // must also be cleaned up.
                                retire_all_live_ssa_cycles(session_id, &ssa_state_snapshot, &pix_toolbox_killswitch);
                                error!(%session_id, %ssa_index, "pix session deposit timeout");
                            } else {
                                warn!(%session_id, "pix session deposit timeout - session not found");
                            }
                        }
                    )),
                );
            }
        }
        info!(%session_id, batch_size, "pix session deposit timeout set");

        // Construct and send the Exit SSA commitment request message
        // The parameters were previously verified to be acceptable. One `params` field covers the
        // whole batch, which is correct: every SSA in it uses the dimensions negotiated for this
        // Session.
        let data = HoprStartProtocol::SsaRequest(SsaServerCommitmentMessage::new(
            session_id,
            current_ssa_state.params,
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

        // All fallible steps succeeded — commit the index advance past the whole batch, and hand the
        // registrations over to the cycles they now belong to.
        current_ssa_state.advance_index(batch_size as RawSsaIndex);
        for guard in commitment_guards {
            guard.disarm();
        }

        Ok(())
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
            // Release reconstructor state for all live SSA cycles:
            // every index from 1 through the current peek index.
            if let (Some(ssa_state), Some(pix_toolbox)) = (slot.current_ssa_state.get(), self.pix_toolbox.get()) {
                retire_all_live_ssa_cycles(*id, ssa_state, pix_toolbox);
            }
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

    /// Dispatches [`HoprSessionInPixEvent`] that notifies the `SessionManager` about PIX protocol
    /// state update.
    ///
    /// Such an event can affect existing Sessions that use the PIX protocol.
    pub async fn dispatch_pix_event(&self, event: HoprSessionInPixEvent) -> errors::Result<()> {
        let session_id = event.pseudonym();
        let Some(slot) = self.sessions.get(event.pseudonym()) else {
            error!(%session_id, "trying to dispatch pix event on a non-existing session");
            return Err(SessionManagerError::NonExistingSession.into());
        };

        match event {
            // When the early recovery threshold is reached, issue a new SSA server request
            // to pipeline deposit preparation with the last ~15% of share collection.
            HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id) => {
                self.request_next_ssa(*session_id, slot, Some(ssa_id.ssa_index()))
                    .await?;
            }
            // SSA fully recovered. Deposit-key processing is handled in the upper layer.
            // If early-recovery pipelining already advanced the cycle (via SsaAlmostRecovered),
            // the guard below is false and this is a no-op. Otherwise — e.g. when
            // early_recovery_threshold == 1.0, or ceil(threshold * num_polys) == num_polys for
            // small num_polys, where the early signal never fires before full recovery — this is
            // the only remaining trigger, so request the next SSA here. Same stale-cycle guard as
            // SsaAlmostRecovered ⇒ fires at most once per cycle and never double-requests.
            HoprSessionInPixEvent::SsaRecovered(ssa_id) => {
                self.request_next_ssa(*session_id, slot, Some(ssa_id.ssa_index()))
                    .await?;
            }
            HoprSessionInPixEvent::UnverifiableShare(ssa_id) => {
                let state = slot.current_ssa_state.get().ok_or(SessionManagerError::Other(anyhow!(
                    "cannot register unverified share on a session without pix state"
                )))?;
                // Skip stale events from a previous SSA cycle (late arrivals after
                // SsaAlmostRecovered advanced the current index).  current_index is
                // the *next* index to allocate; `request_next_ssa` peeks it and only increments
                // (via `advance_index`) after the SsaRequest send succeeds, so the active
                // cycle's index is current_index - 1.
                if ssa_id.ssa_index().get()
                    != state
                        .current_index
                        .load(std::sync::atomic::Ordering::Relaxed)
                        .saturating_sub(1)
                {
                    trace!(
                        %session_id, event_ssa_index = %ssa_id.ssa_index(),
                        "ignoring unverifiable share from stale SSA cycle"
                    );
                    return Ok(());
                }
                let num_errors = state.increment_errors();
                trace!(%session_id, ssa_index = %ssa_id.ssa_index(), num_errors, "encountered unverifiable share in session with pix");

                if num_errors > MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES && self.close_session(session_id) {
                    error!(%session_id, "closed session due to too many unverifiable shares");
                }
            }
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
        };
        self.sessions.insert(session_id, slot);
        self.slot_allocated
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&session_id);
        session_rx
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
            let session = HoprSession::new(
                session_id,
                reply_routing.clone(),
                session_config(&self.cfg, session_req.capabilities.into()),
                (
                    // Sent packets = SURB consumption estimate
                    msg_sender
                        .clone()
                        .with(move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                            // Each outgoing packet consumes one SURB
                            surb_estimator_clone
                                .consumed
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            #[cfg(feature = "telemetry")]
                            telemetry::record_session_surb_consumed(&session_id, 1);
                            futures::future::ok::<_, S::Error>((routing, data))
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
            HoprSession::new(
                session_id,
                reply_routing.clone(),
                session_config(&self.cfg, session_req.capabilities.into()),
                (msg_sender.clone(), session_rx),
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

        // If client requested PIX, and we support it
        // (it was previously verified that the offered parameters are acceptable for us),
        // then create initial Server SSA commitment message and send it to the client.
        // Defer slot_guard.commit() until after PIX setup succeeds: if request_next_ssa
        // fails, the Drop impl rolls back the session slot (removes it from the cache and
        // closes the session), preventing an established session without a PixKillSwitch.
        if self.pix_toolbox.get().is_some() && session_req.capabilities.0.contains(Capability::UsePIX) {
            // We use the same quota that the client offered
            slot.current_ssa_state
                .set(SessionSsaState::new(client_params))
                .map_err(|_| SessionManagerError::other(anyhow::anyhow!("session pix state must be uninitialized")))?;

            // SessionEstablished was already sent to the Entry (above). If PIX
            // setup fails now, the slot_guard Drop rolls back locally, but the
            // Entry would be left with a zombie session that has no PIX kill
            // switch and will never receive the SsaRequest it is waiting for.
            // Best-effort notify the Entry so it can tear down its side too.
            // Known asymmetry: SessionEstablished is sent to the Entry *before* PIX setup
            // below. If request_next_ssa fails here, the Entry sees an established session
            // with no PIX kill switch and will never receive the SsaRequest it is waiting
            // for. The SessionError below notifies the Entry to tear down via
            // handle_session_error(ErrorIdentifier::SessionId(..)), which calls
            // close_session. This is best-effort: network loss or concurrent processing
            // may leave the Entry's session alive (it will time out on its own).
            if let Err(e) = self.request_next_ssa(session_id, slot, None).await {
                if let Err(send_err) = send_via_msg_sender(
                    &mut msg_sender,
                    reply_routing,
                    HoprStartProtocol::SessionError(StartErrorType {
                        identifier: ErrorIdentifier::SessionId(session_id),
                        reason: StartErrorReason::Unknown,
                    }),
                    "session error after PIX setup failure",
                )
                .await
                {
                    tracing::warn!(%session_id, %send_err, "failed to send SessionError after PIX setup failure");
                }
                return Err(e);
            }
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

        if ssa_client_commitment_state.deposit_address_first_encountered
            && let Some(deposit_address) = ssa_client_commitment_state.ssa_deposit_address
        {
            // Inside the guard on purpose: every other `SsaCommit` of a cycle takes the other branch,
            // so allocating this before the `if` built and dropped a channel per message.
            let (deposit_done_tx, deposit_done_rx) = futures::channel::mpsc::channel(10);
            let slot_clone = session_slot.clone();
            // Scaled by the batch size for the same reason the kill switch is, and it has to be: this
            // awaiter is the *only* thing that aborts `PixKillSwitch(ssa_index)`. If it gave up after
            // an unscaled `max_deposit_wait` while the kill switch waited for the whole batch window,
            // a deposit that arrived legitimately late — the N-th of a batch the Entry is funding in
            // order — would land with nothing left to observe it, and the Session would be closed for
            // an unrealized deposit that was in fact realized.
            let max_deposit_wait = self.cfg.pix_config.ssas_per_request.clamp(1, MAX_SSA_BATCH_SIZE) as u32
                * self.cfg.pix_config.max_deposit_wait;
            // TODO: generalize the awaiter into a perpetual Session task that either awaits for Deposit or a signal
            // that sends Exit commitment and reinstates the kill-switch.
            session_slot.abort_handles.lock().insert(
                SessionHandles::DepositAwaiter(ssa_id.ssa_index().get()),
                hopr_utils::spawn_as_abortable!(async move {
                    let deposit_done_rx_result = deposit_done_rx
                        .filter(|((evt_pseudonym, evt_index), _)| {
                            futures::future::ready(
                                evt_index == &ssa_id.ssa_index() && evt_pseudonym == ssa_id.pseudonym(),
                            )
                        })
                        .next()
                        .delay(futures_time::time::Duration::from_millis(100))
                        .timeout(futures_time::time::Duration::from(max_deposit_wait))
                        .await;
                    match deposit_done_rx_result {
                        Ok(Some(_)) => {
                            // Abort the kill switch once the deposit has been done
                            // This kill-switch is reinstated once the SSA has been recovered and a new Client
                            // commitment is needed.
                            // TODO: how to kill the Session if we do not observe progress towards the current SSA
                            // deposit recovery?
                            slot_clone
                                .abort_handles
                                .lock()
                                .abort_one(&SessionHandles::PixKillSwitch(ssa_id.ssa_index().get()));
                            info!(%session_id, "SSA deposit successful");
                        }
                        Ok(None) => {
                            warn!(%session_id, "deposit channel closed without confirmation");
                        }
                        Err(_) => {
                            warn!(%session_id, "deposit confirmation timed out");
                        }
                    }
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
    /// Without the notification the refusal is invisible to the Exit, and it has no way to recover
    /// from it: it armed one kill switch per requested index *before* sending, it will never receive
    /// an `SsaCommit`, and no PIX event can fire to make it re-request — `request_next_ssa` is only
    /// reached from establishment and from share-recovery events, and no shares are produced for a
    /// cycle the Entry never committed to. So it serves the Session unincentivized for the whole
    /// `ssas_per_request × (max_deposit_wait + max_ssa_delivery_time)` window and then closes it as
    /// `UnrealizedDeposit` — a reason that names the deposit rather than the refusal, on the one node
    /// whose operator can act on it. Telling it collapses that to roughly one round trip and puts the
    /// cause in its log where the failure happens.
    ///
    /// The Exit's `handle_session_error` closes the Session on a `SessionId`-identified error, which
    /// also retires the reconstructor cycles it registered for the batch rather than leaving them to
    /// their own expiry. It sends nothing back, so there is no error exchange to loop.
    ///
    /// No new capability is handed to an attacker by closing on a refusal: an `SsaRequest` only
    /// reaches here Sphinx-authenticated and with `pseudonym == session_id`, so only the Exit can
    /// produce one — and the Exit can already close the Session whenever it likes.
    ///
    /// Best-effort. A failed send changes nothing, because the Exit's kill switch remains the
    /// backstop; the local close is unconditional because a refused request is terminal for the
    /// Session either way (the Exit re-derives every request from state that cannot drift within a
    /// Session, so a later one would be refused identically), and leaving the slot up would keep an
    /// unusable Session alive until the idle timeout.
    async fn refuse_ssa_request(&self, session_id: SessionId, routing: DestinationRouting) {
        let reason = StartErrorReason::UnacceptablePixParams;
        if let Some(mut msg_sender) = self.msg_sender.get().cloned() {
            match send_via_msg_sender(
                &mut msg_sender,
                routing,
                HoprStartProtocol::SessionError(StartErrorType {
                    identifier: ErrorIdentifier::SessionId(session_id),
                    reason,
                }),
                "session error message due to a refused SSA request",
            )
            .await
            {
                Ok(()) => {
                    #[cfg(all(feature = "telemetry", not(test)))]
                    METRIC_SENT_SESSION_ERRS.increment(&[&reason.to_string()]);
                }
                Err(error) => {
                    warn!(%session_id, %error, "failed to notify the Exit about a refused SSA request");
                }
            }
        } else {
            warn!(%session_id, "cannot notify the Exit about a refused SSA request - manager not started");
        }

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
        // code — but `pix_params_to_quota` deliberately ignores the surplus, so no quota comparison
        // can say anything about it at all.  Comparing the params is both stricter and simpler, and
        // costs nothing now that all four travel together.
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

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        // The server can theoretically send multiple SSA commitments
        // asking us to make the equal number of client commitments and deposits.
        //
        // The server is authoritative in giving the ssa_index; the client only verifies that it is
        // strictly monotonic. That monotonicity is enforced inside `new_ssa_commitment` below, which
        // rejects any `ssa_index` that is `<=` the last one generated for this pseudonym with
        // `PixError::InvalidInput` (see `SsaShareGenerator::new_ssa_commitment`). Because that call
        // happens *before* the deposit address is derived and `ReadyToDeposit` is emitted, a stale,
        // duplicate, or out-of-order `SsaRequest` cannot cause a second deposit — the whole message is
        // rejected first. The per-pseudonym baseline lives in the generator's `polynomials` cache
        // (30-min idle TTL, refreshed on every use), so it persists for the life of an active session.
        // Gaps (an index strictly greater than the last, but not the immediate successor) are allowed
        // by design, since the Exit may advance by more than one SSA at a time.
        for (ssa_index, exit_commitment) in msg.commitments {
            trace!(ssa_index, "received Exit SSA commitment");

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

            // Construct the full SSA by adding the client and exit commitments, getting the deposit address
            let full_ssa = client_commitment.ssa_commitment
                + exit_commitment
                    .try_into_pix_group()
                    .map_err(SessionManagerError::other)?;
            let deposit_address = HoprPixSpec::group_to_deposit_address(full_ssa).ok_or(SessionManagerError::other(
                anyhow::anyhow!("failed to convert SSA to deposit address"),
            ))?;

            // Split the SSA client commitment into Start protocol commitment messages
            let commitment_msgs = SsaClientCommitmentMessage::new_multiple(msg.session_id, client_commitment)
                .map_err(SessionManagerError::other)?;
            debug!(%ssa_index, count = commitment_msgs.len(), "generated client SSA commitment messages");

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
    use hopr_protocol_pix::{MAX_POLY_THRESHOLD, SsaGeneratorConfig, SsaReconstructorConfig};
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

    /// Verifies that a session is closed on the very first `UnverifiableShare` PIX event.
    ///
    /// An event now means a whole polynomial failed to open its commitment, which already dooms
    /// the cycle — so [`MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES`] is 0 and there is nothing to
    /// tolerate. See that constant for the reasoning.
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` and a PIX quota config. Alice's session initiation is processed
    ///    via `handle_incoming_session_initiation`.
    /// 2. One `UnverifiableShare` event is dispatched for the session's `SsaId`.
    /// 3. The session is closed: `active_sessions` is empty and `num_active_sessions` is 0, confirming the kill switch
    ///    fired.
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
        mgr.dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShare(ssa_id))
            .await?;

        assert!(
            mgr.active_sessions().is_empty(),
            "session must be closed by the first unverifiable share"
        );
        assert_eq!(mgr.num_active_sessions(), 0);

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Verifies that when the exit/responder receives an `SsaAlmostRecovered` PIX event, it requests
    /// a fresh SSA by sending another `SsaRequest` to the entry/initiator.
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` and a PIX quota config. Alice's session initiation is processed
    ///    via `handle_incoming_session_initiation`, which triggers an initial `SsaRequest` (message 1).
    /// 2. The mock transport tracks all outbound messages; exactly 3 messages are expected: `SessionEstablished`,
    ///    initial `SsaRequest` at init, and a second `SsaRequest` after recovery.
    /// 3. `dispatch_pix_event(SsaAlmostRecovered(ssa_id))` is called on Bob's manager.
    /// 4. The manager processes the event and emits a second `SsaRequest` to Alice.
    /// 5. The test asserts that exactly 2 `SsaRequest` messages were sent, confirming the early recovery path triggered
    ///    a new SSA request.
    #[test_log::test(tokio::test)]
    async fn exit_requests_new_ssa_after_almost_recovered_event() -> anyhow::Result<()> {
        use std::sync::Arc;

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

        let sent_ssa_requests = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut bob_transport = MockMsgSender::new();
        let sent_ssa_requests_clone = sent_ssa_requests.clone();

        // Accept all messages; track SsaRequest calls.
        // 3 messages expected: SessionEstablished (1) + SsaRequest at init (2) + SsaRequest after early event (3).
        bob_transport.expect_send_message().times(3).returning(move |_, data| {
            let sent_ssa_requests_clone = sent_ssa_requests_clone.clone();
            Box::pin(async move {
                if let Ok(HoprStartProtocol::SsaRequest(_)) =
                    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
                {
                    sent_ssa_requests_clone.lock().unwrap().push(());
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

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id))
            .await?;

        bob_sender.close_channel();
        bob_handle.await??;

        assert_eq!(
            sent_ssa_requests.lock().unwrap().len(),
            2,
            "expected exactly 2 SsaRequest messages (one at init, one after SsaAlmostRecovered)"
        );

        Ok(())
    }

    /// Verifies that a stale `SsaAlmostRecovered` event from a previous SSA cycle is silently
    /// ignored and does NOT trigger a duplicate `request_next_ssa`.
    ///
    /// ## Steps
    /// 1. Bob's manager starts with PIX; process Alice's session initiation (1 SsaRequest at init).
    /// 2. Dispatch `SsaAlmostRecovered(ssa_id)` — matches active cycle, triggers a second SsaRequest.
    /// 3. Dispatch the *same* `SsaAlmostRecovered(ssa_id)` again — now stale (index advanced in step 2), must be
    ///    silently ignored.
    /// 4. Assert exactly 2 SsaRequest messages total (init + step 2, no duplicate from step 3).
    #[test_log::test(tokio::test)]
    async fn exit_ignores_stale_ssa_almost_recovered_event() -> anyhow::Result<()> {
        use std::sync::Arc;

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

        let sent_ssa_requests = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut bob_transport = MockMsgSender::new();
        let sent_ssa_requests_clone = sent_ssa_requests.clone();

        // 3 messages: SessionEstablished (1) + SsaRequest at init (2) + SsaRequest after first
        // SsaAlmostRecovered (3). The stale dispatch must not trigger a fourth message.
        bob_transport.expect_send_message().times(3).returning(move |_, data| {
            let sent_ssa_requests_clone = sent_ssa_requests_clone.clone();
            Box::pin(async move {
                if let Ok(HoprStartProtocol::SsaRequest(_)) =
                    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
                {
                    sent_ssa_requests_clone.lock().unwrap().push(());
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

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);

        // First dispatch — matches the active cycle index (1 = current_index - 1).
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id))
            .await?;

        // Second dispatch — same ssa_id, but request_next_ssa in step 2 advanced the
        // current_index, so this is now stale and must be silently ignored.
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id))
            .await?;

        bob_sender.close_channel();
        bob_handle.await??;

        assert_eq!(
            sent_ssa_requests.lock().unwrap().len(),
            2,
            "expected exactly 2 SsaRequest messages (init + first AlmostRecovered), stale event must not trigger a \
             third"
        );

        Ok(())
    }

    /// Verifies that concurrent [`SsaAlmostRecovered`] and [`SsaRecovered`] for the same SSA cycle
    /// are serialized by the `request_lock` and produce exactly one extra SSA request.
    ///
    /// ## Rationale
    ///
    /// Both events are dispatched by the PIX reconstructor upon reaching the early-recovery
    /// threshold and full recovery, respectively. In non-deterministic environments (Tokio task
    /// scheduling, shared thread pools), both can arrive in quick succession — sometimes racing
    /// inside [`SessionManager::dispatch_pix_event`]. The `request_lock` ensures that only one
    /// thread passes through `peek_index → fallible work → advance_index`; the other sees a stale
    /// index (the cycle advanced before it acquired the lock) and becomes a no-op via the
    /// [stale-cycle guard](SessionManager::request_next_ssa).
    ///
    /// ## Steps
    ///
    /// 1. Bob's manager is started with a `PixToolbox` and PIX quota config.
    /// 2. Alice's session initiation is processed (triggers 1st `SsaRequest` via internal `request_next_ssa` from
    ///    `handle_incoming_session_initiation`).
    /// 3. `SsaAlmostRecovered(ssa_id)` and `SsaRecovered(ssa_id)` are dispatched concurrently with `tokio::join!`.
    /// 4. Exactly 2 `SsaRequest` messages total — the PIX event that acquires `request_lock` first advances the index;
    ///    the second finds it stale and returns without sending.
    #[test_log::test(tokio::test)]
    async fn exit_handles_concurrent_almost_and_full_recovery_for_same_ssa() -> anyhow::Result<()> {
        use std::sync::Arc;

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

        let sent_ssa_requests = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut bob_transport = MockMsgSender::new();
        let sent_ssa_requests_clone = sent_ssa_requests.clone();

        // 3 messages: SessionEstablished (1) + SsaRequest at init (2) + SsaRequest from
        // whichever PIX event wins the lock (3). The other event is a no-op.
        bob_transport.expect_send_message().times(3).returning(move |_, data| {
            let sent_ssa_requests_clone = sent_ssa_requests_clone.clone();
            Box::pin(async move {
                if let Ok(HoprStartProtocol::SsaRequest(_)) =
                    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
                {
                    sent_ssa_requests_clone.lock().unwrap().push(());
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

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);

        // Dispatch both events concurrently. The request_lock serializes them:
        // whichever acquires the lock first advances the index; the other returns
        // early via the stale-cycle guard.
        let (almost_result, recovered_result) = tokio::join!(
            mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id)),
            mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaRecovered(ssa_id)),
        );
        almost_result?;
        recovered_result?;

        bob_sender.close_channel();
        bob_handle.await??;

        assert_eq!(
            sent_ssa_requests.lock().unwrap().len(),
            2,
            "expected exactly 2 SsaRequest messages (init + one from the concurrent dispatch), the second event must \
             be a no-op due to the stale-cycle guard"
        );

        Ok(())
    }

    /// Verifies that the exit/responder requests the next SSA on `SsaRecovered` when
    /// early-recovery pipelining did NOT already do so for this cycle.
    ///
    /// With `polynomials_per_ssa == 2` and the default `0.85` threshold,
    /// `ceil(0.85 * 2) == 2 == num_polys`, so `SsaAlmostRecovered` never fires before full
    /// recovery — `SsaRecovered` is the only remaining trigger for the next SSA. (The same holds
    /// for `early_recovery_threshold == 1.0` at any `num_polys`.) This is the M1 regression guard.
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` and a PIX quota config. Alice's session initiation is processed,
    ///    which triggers an initial `SsaRequest` (message 1).
    /// 2. `dispatch_pix_event(SsaRecovered(ssa_id))` is called on Bob's manager, with no preceding
    ///    `SsaAlmostRecovered`.
    /// 3. The test asserts that 2 `SsaRequest` messages were sent (init + the one triggered by `SsaRecovered`),
    ///    confirming full recovery advances the cycle when no early-recovery event pipelined it.
    #[test_log::test(tokio::test)]
    async fn exit_requests_new_ssa_on_recovery_when_not_already_pipelined() -> anyhow::Result<()> {
        use std::sync::Arc;

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

        let sent_ssa_requests = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut bob_transport = MockMsgSender::new();
        let sent_ssa_requests_clone = sent_ssa_requests.clone();

        // Accept 3 messages: SessionEstablished (1) + SsaRequest at init (2) +
        // SsaRequest triggered by SsaRecovered (3).
        bob_transport.expect_send_message().times(3).returning(move |_, data| {
            let sent_ssa_requests_clone = sent_ssa_requests_clone.clone();
            Box::pin(async move {
                if let Ok(HoprStartProtocol::SsaRequest(_)) =
                    HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
                {
                    sent_ssa_requests_clone.lock().unwrap().push(());
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

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaRecovered(ssa_id))
            .await?;

        bob_sender.close_channel();
        bob_handle.await??;

        assert_eq!(
            sent_ssa_requests.lock().unwrap().len(),
            2,
            "expected 2 SsaRequest messages (init + one triggered by SsaRecovered, since no SsaAlmostRecovered \
             pipelined it)"
        );

        Ok(())
    }

    /// Verifies that pipelining a second SSA does NOT abort the first cycle's
    /// deposit kill-switch.  With per-index keys, `PixKillSwitch(1)` and
    /// `PixKillSwitch(2)` are independent entries in the `AbortableList`.
    #[test_log::test(tokio::test)]
    async fn pipelined_ssa_preserves_earlier_deposit_deadline() -> anyhow::Result<()> {
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

        let slot = mgr.sessions.get(&alice_pseudonym).unwrap();
        assert!(
            slot.abort_handles.lock().contains(&SessionHandles::PixKillSwitch(1)),
            "first cycle's PixKillSwitch must be present after init"
        );

        // Pipeline the second SSA via early recovery.
        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id))
            .await?;

        // After pipelining both must coexist (scoped to drop MutexGuard before .await).
        {
            let handles = slot.abort_handles.lock();
            assert!(
                handles.contains(&SessionHandles::PixKillSwitch(1)),
                "first cycle's deposit deadline was removed by pipelining"
            );
            assert!(
                handles.contains(&SessionHandles::PixKillSwitch(2)),
                "second cycle's deposit deadline was not installed"
            );
        }

        bob_sender.close_channel();
        bob_handle.await??;
        Ok(())
    }

    /// Verifies that explicit close_session retires all SSA cycles, not just
    /// the last two.  After several pipelined cycles, every index from 1
    /// through current must be retired.
    #[test_log::test(tokio::test)]
    async fn close_session_retires_all_ssa_cycles() -> anyhow::Result<()> {
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

        // Pipeline two more SSAs via SsaAlmostRecovered so that current_index
        // advances past index 1, 2, 3.
        let ssa1 = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa1))
            .await?;
        let ssa2 = SsaId::new(alice_pseudonym, 2.try_into()?);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa2))
            .await?;

        // Grab a reference to the reconstructor before close_session consumes
        // the slot.
        let pix_toolbox_ref = mgr.pix_toolbox.get().unwrap().clone();
        let share_processor = pix_toolbox_ref.share_processor;

        // Precondition: builders exist for indices 1 through 3 before teardown,
        // so that the post-close assertions below prove actual retirement rather
        // than absence of never-created builders.
        for i in 1..=3_u32 {
            let sid = SsaId::new(alice_pseudonym, i.try_into()?);
            assert!(
                share_processor.contains_builder(&sid),
                "precondition: builder for SsaId index {i} must exist before close_session"
            );
        }

        mgr.close_session(&alice_pseudonym);

        // After close_session, every builder for indices 1,2,3 must be gone.
        for i in 1..=3_u32 {
            let sid = SsaId::new(alice_pseudonym, i.try_into()?);
            assert!(
                !share_processor.contains_builder(&sid),
                "builder for SsaId index {i} should have been retired"
            );
        }

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

    /// The Exit must pack [`IncomingSessionPixConfig::ssas_per_request`] commitments into a *single*
    /// `SsaRequest` at contiguous indices, and advance the SSA index past the whole batch.
    ///
    /// The index advance is what the pipelining guard reads: after a batch, `current_index - 1` must
    /// be the *last* index of the batch, so that only that cycle's `SsaAlmostRecovered` triggers the
    /// next batch and the earlier ones are correctly treated as stale.
    #[test_log::test(tokio::test)]
    async fn exit_batches_configured_number_of_ssas_into_one_request() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        const BATCH: usize = 3;

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
                ssas_per_request: BATCH,
                ..Default::default()
            },
            ..Default::default()
        });

        // Capture the commitment sets of every SsaRequest that goes out.
        let requested: Arc<std::sync::Mutex<Vec<Vec<u32>>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let requested_clone = requested.clone();
        let mut bob_transport = MockMsgSender::new();
        bob_transport.expect_send_message().returning(move |_, data| {
            if let Ok(HoprStartProtocol::SsaRequest(req)) = HoprStartProtocol::try_from(data.data) {
                requested_clone
                    .lock()
                    .unwrap()
                    .push(req.commitments.keys().map(|i| i.get()).collect());
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

        let slot = mgr.sessions.get(&alice_pseudonym).context("session must exist")?;
        let ssa_state = slot.current_ssa_state.get().context("pix state must be set")?;

        // Batching happens on the first request too, not only on pipelined refills.
        assert_eq!(
            ssa_state.peek_index().get(),
            BATCH as u32 + 1,
            "index must advance past the whole batch"
        );

        // Only the last index of the batch may trigger the next one.
        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(BATCH as u32).expect("non-zero"));
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id))
            .await?;
        assert_eq!(
            ssa_state.peek_index().get(),
            2 * BATCH as u32 + 1,
            "the last index of a batch must trigger the next batch"
        );

        // An earlier index of the previous batch is stale and must not request anything.
        let stale = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(stale))
            .await?;
        assert_eq!(
            ssa_state.peek_index().get(),
            2 * BATCH as u32 + 1,
            "a stale index from an earlier batch must not trigger a request"
        );

        bob_sender.close_channel();
        bob_handle.await??;

        let requested = requested.lock().unwrap().clone();
        assert_eq!(
            requested,
            vec![vec![1, 2, 3], vec![4, 5, 6]],
            "each request must carry the whole batch at contiguous indices, in one message"
        );

        Ok(())
    }

    /// Within a *live* batch, `SsaAlmostRecovered` for every index but the last must be a no-op.
    ///
    /// Nothing suppresses the earlier signals upstream: each SSA of a batch has its own builder and
    /// its own `check_early_threshold` latch, and the Entry's polynomial queue is FIFO, so the Exit
    /// reconstructs `[1, 2, 3]` in index order and genuinely raises the early-recovery signal for
    /// each member in turn. The stale-cycle guard in `request_next_ssa` is the only thing that stops
    /// each of them from allocating a batch of its own — which would burn `BATCH` indices and emit
    /// `BATCH` deposits per member.
    ///
    /// This is the arrival order that [`exit_batches_configured_number_of_ssas_into_one_request`]
    /// does not exercise: there the earlier index arrives only *after* the last one already advanced
    /// the cycle, so it is stale against a superseded batch rather than an active one.
    ///
    /// `peek_index` is the mid-test signal rather than the captured traffic, because `advance_index`
    /// runs inside the awaited `request_next_ssa` whereas the send only reaches
    /// [`mock_packet_planning`]'s forwarding task afterwards. The requests are asserted at the end,
    /// where they also prove no extra one slipped out.
    #[test_log::test(tokio::test)]
    async fn only_the_last_index_of_a_live_batch_triggers_the_next_one() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        const BATCH: usize = 3;

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
                ssas_per_request: BATCH,
                ..Default::default()
            },
            ..Default::default()
        });

        let requested: Arc<std::sync::Mutex<Vec<Vec<u32>>>> = Arc::new(std::sync::Mutex::new(Vec::new()));
        let requested_clone = requested.clone();
        let mut bob_transport = MockMsgSender::new();
        bob_transport.expect_send_message().returning(move |_, data| {
            if let Ok(HoprStartProtocol::SsaRequest(req)) = HoprStartProtocol::try_from(data.data) {
                requested_clone
                    .lock()
                    .unwrap()
                    .push(req.commitments.keys().map(|i| i.get()).collect());
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

        let slot = mgr.sessions.get(&alice_pseudonym).context("session must exist")?;
        let ssa_state = slot.current_ssa_state.get().context("pix state must be set")?;

        let after_first_batch = BATCH as u32 + 1;
        assert_eq!(
            ssa_state.peek_index().get(),
            after_first_batch,
            "establishment must allocate the whole batch [1, 2, 3]"
        );

        // 1 and 2 cross their own early-recovery threshold first, in that order, while [1, 2, 3] is
        // still the current batch. Neither may allocate anything.
        for non_last in 1..BATCH as u32 {
            let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(non_last).expect("non-zero"));
            mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id))
                .await?;
            assert_eq!(
                ssa_state.peek_index().get(),
                after_first_batch,
                "SsaAlmostRecovered({non_last}) is not the last index of the live batch, so it must not request \
                 anything"
            );
        }

        // Only the last index pipelines the refill.
        let last = SsaId::new(alice_pseudonym, SsaIndex::new(BATCH as u32).expect("non-zero"));
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaAlmostRecovered(last))
            .await?;
        assert_eq!(
            ssa_state.peek_index().get(),
            2 * BATCH as u32 + 1,
            "SsaAlmostRecovered({BATCH}) must request the next batch [4, 5, 6]"
        );

        bob_sender.close_channel();
        bob_handle.await??;

        let requested = requested.lock().unwrap().clone();
        assert_eq!(
            requested,
            vec![vec![1, 2, 3], vec![4, 5, 6]],
            "exactly two requests: the establishment batch, and the one the last index triggered"
        );

        Ok(())
    }

    /// Every index of a batch gets its own kill switch, and they all share one deadline scaled by the
    /// batch size.
    ///
    /// Both halves matter: per-index handles are what let one cycle's deposit awaiter abort its own
    /// deadline without touching its siblings', and the scaling is what stops the Session being closed
    /// while the Entry is still legitimately working through the batch.
    #[test_log::test(tokio::test)]
    async fn batched_ssa_request_scales_the_deposit_deadline() -> anyhow::Result<()> {
        use std::time::Duration;

        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        const BATCH: usize = 3;
        // One cycle's worth of window. At BATCH=3 the batch window is 150 ms, so an unscaled
        // implementation would already have closed the Session by the 100 ms checkpoint below.
        const UNIT: Duration = Duration::from_millis(50);

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
                ssas_per_request: BATCH,
                max_deposit_wait: UNIT,
                max_ssa_delivery_time: Duration::ZERO,
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

        // One independent kill switch per requested index.
        {
            let slot = mgr.sessions.get(&alice_pseudonym).context("session must exist")?;
            let handles = slot.abort_handles.lock();
            for idx in 1..=BATCH as u32 {
                assert!(
                    handles.contains(&SessionHandles::PixKillSwitch(idx)),
                    "every index of the batch needs its own kill switch, {idx} is missing"
                );
            }
        }

        // Past one cycle's window, well short of the batch's. No deposit is ever made.
        tokio::time::sleep(2 * UNIT).await;
        assert_eq!(
            vec![alice_pseudonym],
            mgr.active_sessions(),
            "the deadline must cover the whole batch, not a single cycle"
        );

        // Past the batch window.
        tokio::time::sleep(BATCH as u32 * UNIT).await;
        assert!(
            mgr.active_sessions().is_empty(),
            "session must close once the whole batch has missed its deposits"
        );

        bob_sender.close_channel();
        bob_handle.await??;

        Ok(())
    }

    /// A failed `SsaRequest` send must leave no Exit commitment registered for *any* index of the
    /// batch, so that the retry — which reuses the very same indices, since the index advance is
    /// deliberately deferred until the send succeeds — is not refused as a duplicate.
    ///
    /// This is what the `SsaCommitmentGuard` in `request_next_ssa` buys, and batching is what makes it
    /// load-bearing: one failed send would otherwise strand every index of the batch permanently, and
    /// the Session could never make progress again.
    #[test_log::test(tokio::test)]
    async fn failed_ssa_request_send_leaves_no_stranded_commitment() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};

        const BATCH: usize = 3;

        let reconstructor = Arc::new(SsaReconstructor::new(SsaReconstructorConfig::default()));
        let (pix_toolbox, _pix_events) = PixToolbox::new(
            SsaShareGenerator::new(SsaGeneratorConfig {
                polynomials_per_ssa: 2,
                threshold: 2,
                surplus_shares: 1,
            })
            .into(),
            reconstructor.clone(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                ssas_per_request: BATCH,
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

        // Closing the transport is what makes the send fail: the mock's own errors surface on the
        // relay task's join handle, not at the `send_via_msg_sender` call inside `request_next_ssa`.
        bob_sender.close_channel();
        bob_handle.await??;

        let alice_pseudonym = HoprPseudonym::random();
        let (dummy_tx, _dummy_rx) = crossfire::mpsc::bounded_blocking_async::<ApplicationDataIn>(1);
        let slot = SessionSlot {
            session_tx: dummy_tx,
            routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(alice_pseudonym)),
            abort_handles: Default::default(),
            surb_mgmt: Default::default(),
            surb_estimator: Default::default(),
            current_ssa_state: Default::default(),
        };
        slot.current_ssa_state
            .set(SessionSsaState::new(PixParams::try_new(
                2,
                2,
                TEST_SURPLUS_SHARES,
                LOCAL_PIX_SUITE,
            )?))
            .map_err(|_| anyhow!("pix state must be uninitialized"))?;
        mgr.sessions.insert(alice_pseudonym, slot.clone());

        // The failing send must surface as an error and must not consume the indices.
        let result = mgr.request_next_ssa(alice_pseudonym, slot.clone(), None).await;
        assert!(result.is_err(), "a failed send must be reported, got {result:?}");
        assert_eq!(
            slot.current_ssa_state.get().unwrap().peek_index().get(),
            1,
            "a failed request must not consume its indices"
        );

        // Every index the attempt registered must be free again. `new_exit_commitment` is exactly what
        // the retry would call, and it rejects an index that is still registered with
        // `DuplicateCommitment` — so a success here is the whole batch having been released.
        for i in 1..=BATCH as u32 {
            let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(i).expect("non-zero"));
            reconstructor
                .new_exit_commitment(ssa_id, 2, 2)
                .with_context(|| format!("index {i} of the failed batch was left registered"))?;
        }

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
                    ssas_per_request,
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

        // The exit commitment is already set up by handle_incoming_session_initiation.
        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);

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

    /// Verifies that a PIX session is closed automatically if the deposit is not realized within
    /// the configured `max_deposit_wait` period.
    ///
    /// ## Steps
    /// 1. Bob's manager is configured with `max_deposit_wait: 50ms` and `max_ssa_delivery_time: 0` (total kill-switch
    ///    window: 50ms).
    /// 2. A `PixToolbox` is provided so the PIX state machine runs. Alice's session initiation is processed via
    ///    `handle_incoming_session_initiation`.
    /// 3. Immediately after establishment, `active_sessions` contains Alice's pseudonym — session is live.
    /// 4. The test sleeps 100ms (past the 50ms deadline). No deposit is ever made.
    /// 5. `active_sessions` is empty and `num_active_sessions` is 0, confirming the kill switch closed the session due
    ///    to the unrealized deposit.
    #[test_log::test(tokio::test)]
    async fn session_is_closed_when_deposit_timeout_fires() -> anyhow::Result<()> {
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

        // Short timeouts so the kill switch fires quickly.
        let mgr = SessionManager::new(SessionManagerConfig {
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                max_deposit_wait: Duration::from_millis(50),
                max_ssa_delivery_time: Duration::ZERO,
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

        // Wait for the kill switch to fire (max_deposit_wait + max_ssa_delivery_time = 50ms + 0 = 50ms).
        // Add a 100ms buffer to be safe.
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Session must be closed due to unrealized deposit.
        assert!(
            mgr.active_sessions().is_empty(),
            "session should be closed after deposit timeout"
        );
        assert_eq!(mgr.num_active_sessions(), 0);

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
                    max_deposit_wait: Duration::from_secs(1),
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

        // One more event than the tolerance allows. At the current tolerance of 0 that is a single
        // event; the loop is written against the constant so lifting the tolerance would not
        // silently turn this into a test of nothing.
        for _ in 0..=MAX_ALLOWED_UNVERIFIABLE_PIX_SHARES {
            let result = mgr
                .dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShare(ssa_id))
                .await;
            // The closing event also "succeeds" because closing the session returns Ok.
            assert!(result.is_ok(), "dispatch_pix_event should not return an error");
        }

        // Session should be closed after too many unverifiable shares.
        assert!(
            mgr.active_sessions().is_empty(),
            "session should be closed after too many unverifiable shares"
        );
        assert_eq!(mgr.num_active_sessions(), 0);

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
