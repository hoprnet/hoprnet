use std::{
    pin::Pin,
    sync::{Arc, OnceLock, atomic::Ordering},
    time::Duration,
};

use anyhow::anyhow;
use futures::{Sink, SinkExt, StreamExt, TryStreamExt, future::AbortHandle};
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
    prelude::{HoprPacket, HoprPixGroupElement},
};
use hopr_protocol_app::prelude::*;
use hopr_protocol_pix::{
    DEFAULT_POLY_THRESHOLD, DEFAULT_POLYS_PER_SSA, EntryShareGenerator, ExitAcknowledgementShareProcessor,
    GroupEncoding, MAX_POLY_THRESHOLD, MAX_POLYS_PER_SSA, PixSpec, SsaCommitmentGuard, SsaId, SsaIndex,
    SsaReconstructor, SsaShareGenerator,
};
use hopr_protocol_start::{
    KeepAliveFlag, KeepAliveMessage, SsaClientCommitmentMessage, SsaServerCommitmentMessage, StartChallenge,
    StartErrorReason, StartErrorType, StartEstablished, StartInitiation,
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
    supervision::*,
    types::{
        ClosureReason, HoprSessionCapabilities, HoprSessionConfig, HoprSessionInPixEvent, HoprStartProtocol,
        SESSION_APPLICATION_TAG, SsaQuota, pix_params_to_quota,
    },
    utils,
    utils::{SlotNotify, SurbNotificationMode, insert_into_next_slot},
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
    static ref METRIC_PIX_CLOSURES: hopr_api::types::telemetry::MultiCounter = hopr_api::types::telemetry::MultiCounter::new(
        "hopr_session_pix_closures_total",
        "Number of PIX-supervised sessions closed, by close reason",
        &["reason"]
    ).unwrap();
    static ref METRIC_PIX_UNVERIFIABLE_SHARES: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new(
            "hopr_session_pix_unverifiable_shares_total",
            "Total number of unverifiable PIX share observations across all sessions"
        ).unwrap();
}

/// Map a `SessionPixCloseReason` to the public `ClosureReason`.
fn pix_close_to_closure_reason(reason: SessionPixCloseReason) -> ClosureReason {
    use SessionPixCloseReason::*;
    match reason {
        CommitmentTimeout | DepositTimeout | DepositObserverClosed => ClosureReason::UnrealizedDeposit,
        RecoveryIdle
        | RecoveryDeadline
        | CounterRegression
        | TooManyUnverifiableShares
        | SupervisorUnavailable
        | InvalidTransition
        | NoSsaRemaining => ClosureReason::PixFailure,
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

    // Poison the egress gate so any parked writers get GateClosed.
    if let Some(gate) = session_data.pix_egress_gate.get() {
        gate.poison();
    }

    // Terminate any additional tasks spawned by the Session
    session_data.abort_handles.lock().abort_all();

    #[cfg(all(feature = "telemetry", not(test)))]
    METRIC_ACTIVE_SESSIONS.decrement(1.0);
}

fn initiation_timeout_max_one_way(base: Duration, hops: usize) -> Duration {
    base * (hops as u32)
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

/// Timeout when sending Start protocol messages to the sink
const EXTERNAL_SEND_TIMEOUT: Duration = Duration::from_millis(200);

/// How long to wait for the supervisor to produce its initial
/// [`RequestSsa`](SessionPixAction::RequestSsa) action.
///
/// The supervisor creates the action synchronously in the worker's first
/// tick, so this is purely a safety guard in case of an internal stall.
const INITIAL_SSA_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Polling guard when waiting for a session slot to appear.
///
/// The slot is populated by a concurrent start-protocol handler after the
/// SSA server commitment arrives; if it hasn't appeared within this window
/// the loop re-checks rather than sleeping forever.
const SLOT_NOTIFY_TIMEOUT: Duration = Duration::from_millis(200);

// Needs to use an UnboundedSender instead of oneshot
// because Moka cache requires the value to be Clone, which oneshot Sender is not.
// It also cannot be enclosed in an Arc, since calling `send` consumes the oneshot Sender.
// The Session initiation cache is only present on the Entry (client) side.
type SessionInitiationCache = moka::sync::Cache<
    StartChallenge,
    crossfire::MTx<crossfire::mpsc::One<Result<StartEstablished<SessionId>, StartErrorType>>>,
>;

/// Handles to streams and tasks spawned by the Session.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, strum::Display)]
enum SessionHandles {
    /// Handle to the stream that facilitates ingress of data from the HOPR network into the Session.
    Ingress,
    /// Handle to the process that sends keep-alive messages to the Session recipient (Exit).
    KeepAlive,
    /// Handle to the process that monitors and balances SURBs.
    Balancer,
    /// Handle to the PIX action driver task.
    PixActionDriver,
    /// Handle to the PIX deposit observer for one SSA.
    PixDepositObserver(SsaIndex),
}

/// PIX parameters negotiated during session establishment.
#[derive(Debug, Clone, Copy)]
struct SessionSsaParameters {
    polys_per_ssa: u16,
    shares_per_poly: u16,
}

impl SessionSsaParameters {
    #[inline]
    pub const fn quota_per_ssa(&self) -> SsaQuota {
        pix_params_to_quota(self.polys_per_ssa, self.shares_per_poly)
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
    // PIX parameters negotiated during session establishment.
    // Shared via OnceLock so it can be set after cache insertion.
    ssa_params: Arc<OnceLock<SessionSsaParameters>>,
    // Handle to the PIX supervisor worker for this session.
    // Shared via OnceLock so it can be set after cache insertion.
    pix_supervisor: Arc<OnceLock<SessionPixSupervisorHandle>>,
    // Egress gate shared with the supervisor; set when PIX is negotiated.
    pix_egress_gate: Arc<OnceLock<Arc<ServiceGate>>>,
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
#[derive(Clone, Debug, PartialEq, smart_default::SmartDefault)]
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
    /// Default is 128 MB to 512 MB (inclusive).
    #[default(_code = "(134217728..=536870912)")]
    pub quota_range: std::ops::RangeInclusive<u64>,
    /// Configuration for the PIX session supervisor.
    ///
    /// Controls timeouts, unverifiable-share tolerances, predeposit budgets,
    /// and tombstones for incoming PIX sessions.
    #[default(_code = "SupervisorConfig::default()")]
    pub supervisor_cfg: SupervisorConfig,
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
    /// Default is 800 ms.
    #[default(Duration::from_millis(800))]
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
/// During [`SessionManager::new_session`], the Entry encodes its PIX SSA (Secret Sharing
/// Aggregation) parameters in the upper 32 bits of the `StartSession.additional_data` field:
/// `polys_per_ssa` at bits 48–63 and `shares_per_ssa` at bits 32–47. These describe how
/// many polynomials and shares each SSA will use, which together define the data quota per SSA.
///
/// On the Exit side, `check_pix_params` validates these parameters against:
/// - The configured [`IncomingSessionPixConfig::quota_range`] (default 128 MB–512 MB per SSA).
/// - The maximum allowed polynomials ([`MAX_POLYS_PER_SSA`]) and threshold ([`MAX_POLY_THRESHOLD`]).
/// - Optionally, [`IncomingSessionPixConfig::enforce_pix`] rejects Sessions that do not offer PIX.
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
/// The Exit's [`SessionPixSupervisor`] transitions the SSA into the
/// [`SsaRequestSent`](supervision::supervisor::SsaState::SsaRequestSent) phase.  If no
/// commitment arrives before `max_ssa_delivery_time` elapses, the supervisor closes the
/// Session with `SessionPixCloseReason::CommitmentTimeout`.
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
/// emits a [`HoprSessionOutPixEvent::ReadyToDeposit`] to the upper layer, signalling that
/// funds can be deposited at the computed address.
///
/// ### 4. Deposit Awaiting (Exit Side)
///
/// The Exit receives the client commitment messages in `handle_ssa_commit`, inserts the
/// coefficient commitments into the [`SsaReconstructor`], and extracts the deposit address.
/// It emits [`HoprSessionOutPixEvent::DepositNeeded`] to the upper layer with the
/// [`AgreedSsaQuota`] and a channel to confirm the deposit.
///
/// The [`SessionPixSupervisor`] tracks this as the
/// [`AwaitingDeposit`](supervision::supervisor::SsaState::AwaitingDeposit) phase.  A
/// per-session worker actor manages the deadline timer asynchronously.  Once the deposit
/// is confirmed (satisfying the expected quota), the supervisor transitions to
/// [`Recovering`](supervision::supervisor::SsaState::Recovering).  If the deposit times
/// out (`max_deposit_wait`), the supervisor closes the Session with
/// `SessionPixCloseReason::UnrealizedDeposit`.
///
/// ### 5. SSA Collection, Recovery, and Pipelining
///
/// As the Entry sends return-path SURBs during the Session, each SURB can carry a PIX
/// share generated from the client's polynomial set. The Exit's [`SsaReconstructor`]
/// collects these shares.
///
/// The [`SessionPixSupervisor`] tracks recovery through
/// [`Recovering`](supervision::supervisor::SsaState::Recovering).  During recovery, the
/// supervisor enforces two deadlines:
/// - A **service-gated idle timer** (`max_recovery_idle`) — resets every time the [`ServiceGate`] observes useful
///   progress.  If no packets are being served, the timer re-arms instead of closing, preventing disconnection of a
///   slow-but-honest Entry.
/// - A **hard recovery deadline** (`max_recovery_time`) — an absolute per-SSA backstop for resource protection (session
///   slot + reconstructor memory).  Never extended.
///
/// Egress data packets from the Exit back to the Entry are gated by the
/// [`ServiceGate`](supervision::gate::ServiceGate).  Before the first deposit, a
/// bounded predeposit budget (`max_predeposit_packets`) allows provisional bidirectional
/// traffic.  After funding, the gate enforces a ceiling on packets served without recovery
/// progress.
///
/// When the reconstructor reaches the *early recovery threshold* (≈85%), an
/// [`HoprSessionInPixEvent::SsaAlmostRecovered`] event fires, which triggers
/// `request_next_ssa` for the next SSA index — pipelining the costly
/// commitment exchange with the tail of share collection for the current SSA.
///
/// Once fully recovered, [`HoprSessionInPixEvent::SsaRecovered`] fires, allowing the
/// Exit to unlock and redeem the deposited funds.  The recovered SSA enters a **tombstone**
/// phase before being retired.  The supervisor maintains at most two live SSAs in flight
/// plus one in tombstone phase.
///
/// ### 6. Unverifiable Shares
///
/// If the reconstructor receives a share whose proof-of-possession cannot be verified
/// against its polynomial commitment, an [`HoprSessionInPixEvent::UnverifiableShare`]
/// event fires. The [`SessionPixSupervisor`] tracks these per-SSA and per-session:
/// - After `max_unverifiable_shares_per_ssa` (default 3), the offending SSA is closed.
/// - After `max_unverifiable_shares_per_session` (default 10, covering multiple SSAs), the entire Session is forcefully
///   closed to prevent a malicious Entry from wasting the Exit's resources.
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
    /// Notify when a session slot is allocated (for event-driven slot-wait).
    slot_notify: SlotNotify,
    msg_sender: Arc<OnceLock<S>>,
    pix_toolbox: OnceLock<PixToolbox>,
    cfg: SessionManagerConfig,
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
            slot_notify: self.slot_notify.clone(),
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

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_ACTIVE_SESSIONS.set(0.0);

        let active_sessions: Arc<std::sync::atomic::AtomicUsize> = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let active_sessions_for_listener = active_sessions.clone();

        let msg_sender = Arc::new(OnceLock::new());
        Self {
            msg_sender: msg_sender.clone(),
            session_initiations: moka::sync::Cache::builder()
                .max_capacity(maximum_sessions as u64)
                .time_to_live(
                    2 * initiation_timeout_max_one_way(
                        cfg.initiation_timeout_base,
                        RoutingOptions::MAX_INTERMEDIATE_HOPS,
                    ),
                )
                .build(),
            sessions: moka::sync::Cache::builder()
                .max_capacity(maximum_sessions as u64)
                .time_to_idle(cfg.idle_timeout)
                .eviction_listener(move |session_id: Arc<SessionId>, entry, reason| match &reason {
                    moka::notification::RemovalCause::Expired | moka::notification::RemovalCause::Size => {
                        trace!(?session_id, ?reason, "session evicted from the cache");
                        active_sessions_for_listener.fetch_sub(1, Ordering::Relaxed);
                        close_session(*session_id.as_ref(), entry, ClosureReason::Eviction);
                    }
                    _ => {}
                })
                .build(),
            slot_notify: SlotNotify::new(),
            pix_toolbox: OnceLock::new(),
            session_notifiers: Arc::new(OnceLock::new()),
            start_protocol_tx: Arc::new(OnceLock::new()),
            active_sessions,
            cfg,
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
        // Validate the PIX supervision config before populating any OnceLock.
        // This ensures a validation failure does not leave the manager in a
        // permanently unstartable state with msg_sender already set.
        if let Some(pix) = &pix {
            let pix_cfg = self.cfg.pix_config.supervisor_cfg.clone();
            validate_pix_supervision(&pix_cfg, pix.share_processor.config())
                .map_err(|e| TransportSessionError::InvalidConfig(e.to_string()))?;
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
            crossfire::mpsc::bounded_blocking_async(self.cfg.maximum_sessions + 10);
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
                self.slot_notify.notify_waiters();
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

        let ssa_params = cfg.pix_ssa_quota.map(|(polys, shares)| SessionSsaParameters {
            polys_per_ssa: polys,
            shares_per_poly: shares,
        });

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

        // PIX quota parameter announcement is encoded in the upper 32-bits of additional_data
        if let Some(ref params) = ssa_params
            && cfg.capabilities.contains(Capability::UsePIX)
        {
            additional_data |= (params.polys_per_ssa as u64) << 48 | (params.shares_per_poly as u64) << 32;
        }

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

                    // Insert the slot and obtain a guard that rolls it back (also tearing
                    // down the abort handles) if any subsequent setup step fails.
                    let mut slot_guard = self
                        .allocate_session_slot(
                            session_id,
                            SessionSlot {
                                session_tx,
                                routing_opts: forward_routing.clone(),
                                abort_handles: Arc::new(parking_lot::Mutex::new(abort_handles)),
                                surb_mgmt: surb_mgmt.clone(),
                                surb_estimator: surb_estimator.clone(),
                                ssa_params: Arc::new(OnceLock::new()),
                                pix_supervisor: Arc::new(OnceLock::new()),
                                pix_egress_gate: Default::default(),
                            },
                        )
                        .ok_or_else(|| {
                            // Session already exists; it means it is most likely a loopback attempt
                            error!(%session_id, "session already exists - loopback attempt");
                            SessionManagerError::Loopback
                        })?;

                    #[cfg(all(feature = "telemetry", not(test)))]
                    METRIC_NUM_INITIATED_SESSIONS.increment();

                    // Wait for enough SURBs to be sent to the counterparty
                    // TODO: consider making this interactive = other party reports the exact level periodically
                    match level_stream
                        .skip_while(|current_level| {
                            futures::future::ready(*current_level < balancer_config.target_surb_buffer_size / 2)
                        })
                        .next()
                        .timeout(futures_time::time::Duration::from(SESSION_READINESS_TIMEOUT))
                        .await
                    {
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

                    let surb_estimator_for_rx = surb_estimator.clone();
                    let session = HoprSession::new(
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
                                ssa_params: Arc::new(OnceLock::new()),
                                pix_supervisor: Arc::new(OnceLock::new()),
                                pix_egress_gate: Default::default(),
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
                error!(
                    challenge = error.challenge,
                    ?error,
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

    async fn send_ssa_request(
        &self,
        session_id: SessionId,
        slot: &SessionSlot,
        ssa_index: SsaIndex,
    ) -> errors::Result<SsaCommitmentGuard<HoprPixSpec>> {
        let pix_toolbox = self.pix_toolbox.get().cloned().ok_or(SessionManagerError::NotStarted)?;
        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        let params = slot
            .ssa_params
            .get()
            .cloned()
            .ok_or(SessionManagerError::Other(anyhow!(
                "cannot request new ssa on a session without pix state"
            )))?;

        // TODO: based on the offered quota, the Exit can decide here whether to ask for more than just one SSA
        // commitment
        let polys_per_ssa = params.polys_per_ssa;
        let shares_per_poly = params.shares_per_poly;
        let (exit_commitment, guard) = hopr_utils::parallelize::cpu::spawn_blocking(
            move || {
                pix_toolbox
                    .share_processor
                    .new_guarded_exit_commitment(
                        SsaId::new(session_id, ssa_index),
                        polys_per_ssa as usize,
                        shares_per_poly as usize,
                    )
                    .map(|(commitment, guard)| (HoprPixGroupElement(commitment.to_bytes()), guard))
            },
            "server_ssa_commitment",
        )
        .await
        .map_err(SessionManagerError::other)?
        .map_err(SessionManagerError::PixError)?;

        info!(%session_id, %ssa_index, %exit_commitment, "generated exit commitment");

        // Construct and send the Exit SSA commitment request message
        // The parameters were previously verified to be acceptable.
        let data = HoprStartProtocol::SsaRequest(SsaServerCommitmentMessage::new(
            session_id,
            params.polys_per_ssa,
            params.shares_per_poly,
            [(ssa_index, exit_commitment)],
        ));

        send_via_msg_sender(
            &mut msg_sender,
            slot.routing_opts.clone(),
            data,
            "session SSA commitment request message",
        )
        .await
        .map_err(TransportSessionError::packet_sending)?;

        Ok(guard)
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

        let Some(handle) = slot.pix_supervisor.get() else {
            error!(%session_id, "trying to dispatch pix event on a session without pix supervisor");
            return Err(SessionManagerError::NonExistingSession.into());
        };

        let pix_ev = match &event {
            HoprSessionInPixEvent::Progress(p) => SessionPixEvent::RecoveryProgress(*p),
            HoprSessionInPixEvent::UnverifiableShares { ssa_id, observed_total } => {
                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_PIX_UNVERIFIABLE_SHARES.increment();
                SessionPixEvent::UnverifiableShares {
                    ssa_id: *ssa_id,
                    observed_total: *observed_total,
                }
            }
            HoprSessionInPixEvent::SsaAlmostRecovered(ssa_id) => SessionPixEvent::AlmostRecovered(*ssa_id),
            HoprSessionInPixEvent::SsaRecovered(ssa_id) => SessionPixEvent::Recovered(*ssa_id),
        };

        handle.send_event(pix_ev).await.map_err(|_| {
            error!(%session_id, "pix supervisor channel closed");
            TransportSessionError::Closed
        })?;

        Ok(())
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
                        SessionManagerError::other(error)
                    })?)
            } else {
                error!(%session_id, "received data from an unestablished session");
                Err(TransportSessionError::UnknownData)
            };
        }

        trace!(tag = %in_data.data.application_tag, "received data not associated with session protocol or any existing session");

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
            ssa_params: Default::default(),
            pix_supervisor: Default::default(),
            pix_egress_gate: Default::default(),
        };
        self.sessions.insert(session_id, slot);
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
            ssa_params: Default::default(),
            pix_supervisor: Default::default(),
            pix_egress_gate: Default::default(),
        };
        self.sessions.insert(session_id, slot);
        session_rx
    }

    /// Checks the PIX parameters offered by the Entry during the Session Initiation.
    ///
    /// Returns the validated parameters, or `None` if the offered parameters were rejected.
    fn check_pix_params(&self, req: &StartInitiation<SessionTarget, HoprSessionCapabilities>) -> Option<(u16, u16)> {
        // TODO: the Exit may decide to use different quota based on the `target` in the StartInitiation message
        if req.capabilities.0.contains(Capability::UsePIX) {
            // Client offered PIX, so validate the offered parameters
            let polys_per_ssa = ((req.additional_data & 0xFFFF0000_00000000_u64) >> 48) as u16;
            let shares_per_ssa = ((req.additional_data & 0x0000FFFF_00000000_u64) >> 32) as u16;

            let quota_per_ssa = pix_params_to_quota(polys_per_ssa, shares_per_ssa);
            debug!(
                challenge = req.challenge,
                polys_per_ssa,
                shares_per_ssa,
                acceptable_range = ?self.cfg.pix_config.quota_range,
                offered_quota_mb_per_ssa = quota_per_ssa as f64 / (1024.0 * 1024.0),
                "client offered MB SSA quota"
            );

            let in_quota_range = self.cfg.pix_config.quota_range.contains(&quota_per_ssa);
            let valid_polys = polys_per_ssa <= MAX_POLYS_PER_SSA;
            let valid_shares = (2_u16..=MAX_POLY_THRESHOLD).contains(&shares_per_ssa);
            (in_quota_range && valid_polys && valid_shares).then_some((polys_per_ssa, shares_per_ssa))
        } else if self.cfg.pix_config.enforce_pix {
            // Client didn't offer PIX, but PIX is enforced
            None
        } else {
            // Client didn't offer PIX, and PIX is not enforced, so set default values
            // which are not going to be used.
            Some((DEFAULT_POLYS_PER_SSA, DEFAULT_POLY_THRESHOLD))
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
                challenge: session_req.challenge,
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
        let Some((client_polys_per_ssa, client_shares_per_ssa)) = self.check_pix_params(&session_req) else {
            error!(
                challenge = session_req.challenge,
                "client offered unacceptable PIX parameters"
            );

            // Notify the sender that the session could not be established
            let reason = StartErrorReason::UnacceptablePixParams;
            let data = HoprStartProtocol::SessionError(StartErrorType {
                challenge: session_req.challenge,
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

        info!(
            client_polys_per_ssa,
            client_shares_per_ssa, "client offered acceptable PIX parameters"
        );

        let (new_session_notifier, close_session_notifier) = self
            .session_notifiers
            .get()
            .cloned()
            .ok_or(SessionManagerError::NotStarted)?;

        // Reply routing uses SURBs only with the pseudonym of this Session's ID
        let reply_routing = DestinationRouting::Return(pseudonym.into());

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
            ssa_params: Default::default(),
            pix_supervisor: Default::default(),
            pix_egress_gate: Default::default(),
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
                challenge: session_req.challenge,
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

        // If PIX is active, set up the supervisor and install the egress gate
        // BEFORE constructing the session, so the gate is populated before any
        // write reaches the egress adapter.
        let pix: Option<(SessionPixSupervisorHandle, SsaCommitmentGuard<HoprPixSpec>, ActionRx)> =
            if self.pix_toolbox.get().is_some() && session_req.capabilities.0.contains(Capability::UsePIX) {
                let params = SessionSsaParameters {
                    polys_per_ssa: client_polys_per_ssa,
                    shares_per_poly: client_shares_per_ssa,
                };

                let dims = SsaDimensions {
                    polys: params.polys_per_ssa,
                    threshold: params.shares_per_poly,
                };

                let pix_cfg = self.cfg.pix_config.supervisor_cfg.clone();

                let (handle, action_rx) = spawn_supervisor_worker(pix_cfg, dims, session_id, std::time::Instant::now());

                // Receive the initial RequestSsa action and send it synchronously.
                let initial_action = action_rx
                    .recv()
                    .timeout(futures_time::time::Duration::from(INITIAL_SSA_REQUEST_TIMEOUT))
                    .await
                    .map_err(|_| {
                        SessionManagerError::Other(anyhow::anyhow!("timeout waiting for initial SSA request"))
                    })?
                    .map_err(|_| SessionManagerError::Other(anyhow::anyhow!("action driver closed prematurely")))?;

                let ssa_id = match &initial_action {
                    SessionPixAction::RequestSsa { ssa_id, .. } => *ssa_id,
                    other => {
                        error!(?other, "unexpected initial action from supervisor");
                        return Err(SessionManagerError::Other(anyhow::anyhow!("unexpected initial action")).into());
                    }
                };

                // Store params in slot first — send_ssa_request reads them.
                let _ = slot.ssa_params.set(params);

                // Send the SSA request message on the wire and capture the RAII guard.
                let guard = self
                    .send_ssa_request(session_id, &slot, ssa_id.ssa_index())
                    .await
                    .map_err(|e| {
                        error!(%session_id, %e, "failed to send initial SSA request, slot will be rolled back");
                        e
                    })?;

                // Notify supervisor that the request was sent.
                handle
                    .send_event(SessionPixEvent::SsaRequestSent(ssa_id))
                    .await
                    .map_err(|_| SessionManagerError::Other(anyhow::anyhow!("supervisor channel closed")))?;

                // Install the gate in the slot BEFORE session construction so the
                // egress adapters inside HoprSession see a populated gate.
                let _ = slot.pix_supervisor.set(handle.clone());
                let _ = slot.pix_egress_gate.set(handle.gate.clone());

                Some((handle, guard, action_rx))
            } else {
                None
            };

        let session = if !session_req.capabilities.0.contains(Capability::NoRateControl) {
            // Because of SURB scarcity, control the egress rate of incoming sessions
            let egress_rate_control =
                RateController::new(self.cfg.initial_return_session_egress_rate, Duration::from_secs(1));

            // The Session request carries a "hint" as additional data telling what
            // the Session initiator has configured as its target buffer size in the Balancer.
            let target_surb_buffer_size = if session_req.additional_data > 0 {
                session_req
                    .additional_data
                    .min(self.cfg.maximum_surb_buffer_size as u64)
            } else {
                self.cfg.initial_return_session_egress_rate as u64
                    * self
                        .cfg
                        .minimum_surb_buffer_duration
                        .max(MIN_SURB_BUFFER_DURATION)
                        .as_secs()
            };

            let surb_estimator_clone = slot.surb_estimator.clone();
            let surb_estimator_for_egress = slot.surb_estimator.clone();
            let session = HoprSession::new(
                session_id,
                reply_routing.clone(),
                session_config(&self.cfg, session_req.capabilities.into()),
                (
                    // Sent packets = SURB consumption estimate
                    msg_sender
                        .clone()
                        .with({
                            let gate = Arc::clone(&slot.pix_egress_gate);
                            let surb = surb_estimator_for_egress;
                            move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                                let gate = gate.get().cloned();
                                let surb = surb.clone();
                                Box::pin(async move {
                                    // Park while predeposit budget is exhausted;
                                    // on poison, park forever — the task will be
                                    // aborted by session teardown.
                                    if let Some(g) = gate
                                        && g.acquire().await.is_err()
                                    {
                                        // Gate poisoned — session is closing.
                                        futures::future::pending::<()>().await;
                                    }
                                    // Each outgoing packet consumes one SURB
                                    surb.consumed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    #[cfg(feature = "telemetry")]
                                    telemetry::record_session_surb_consumed(&session_id, 1);
                                    Ok::<_, S::Error>((routing, data))
                                })
                            }
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
            };

            slot.surb_mgmt.update(&balancer_config);

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
            //
            // NOTE: The keep-alive egress path intentionally bypasses the PIX service gate
            // (pix_egress_gate.acquire()).  This is a deliberate, bounded exemption:
            //
            //   - Keep-alive payloads carry no client-usable data — they are SURB-level notifications only.
            //   - They cannot advance SSA recovery or deposit redemption.
            //   - The Exit-side cost per unfunded session is ≤1 win-prob-scaled ticket/s for at most
            //     max_ssa_delivery_time + max_deposit_wait (~80 s at defaults).
            //   - poison() does NOT stop the keep-alive stream; only teardown via the KeepAlive abort handle does.
            //
            // The gate governs only data-plane egress, not control-plane keep-alive.
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
                (
                    msg_sender.clone().with({
                        let gate = Arc::clone(&slot.pix_egress_gate);
                        move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                            let gate = gate.get().cloned();
                            Box::pin(async move {
                                if let Some(g) = gate
                                    && g.acquire().await.is_err()
                                {
                                    // Gate poisoned — session is closing.
                                    futures::future::pending::<()>().await;
                                }
                                Ok::<_, S::Error>((routing, data))
                            })
                        }
                    }),
                    session_rx,
                ),
                Some(closure_notifier),
            )?
        };

        // Extract useful information about the session from the Start protocol message
        let incoming_session = IncomingSession {
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

        // If PIX was set up before session construction, spawn the action
        // driver task now (after the session is published).
        if let Some((handle, initial_guard, action_rx)) = pix {
            let myself = self.clone();
            let gate = slot.pix_egress_gate.get().cloned().expect("gate just set");
            let slot_for_driver = slot.clone();
            let supervisor_handle = handle.clone();
            let ah_action_driver = hopr_utils::spawn_as_abortable!(async move {
                // Owned collection of SsaCommitmentGuards — each guard calls
                // retire_ssa on drop, so clearing the vec or dropping it
                // handles cleanup automatically.
                let mut retirement = SsaRetirementGuard {
                    guards: vec![initial_guard],
                };

                loop {
                    let action = match action_rx.recv().await {
                        Ok(a) => a,
                        Err(_) => break, // sender dropped → close
                    };
                    match action {
                        SessionPixAction::RequestSsa {
                            ssa_id,
                            polys,
                            threshold,
                        } => {
                            let action_key = SessionPixAction::RequestSsa {
                                ssa_id,
                                polys,
                                threshold,
                            };
                            // Look up the session slot and send the request.
                            if let Some(s) = myself.sessions.get(&session_id) {
                                let result = myself.send_ssa_request(session_id, &s, ssa_id.ssa_index()).await;
                                if let Some(handle) = s.pix_supervisor.get()
                                    && handle
                                        .send_event(SessionPixEvent::SsaRequestSent(ssa_id))
                                        .await
                                        .is_err()
                                {
                                    error!(%session_id, "failed to send SsaRequestSent to PIX supervisor");
                                }
                                if let Some(handle) = s.pix_supervisor.get()
                                    && handle.send_action_result(action_key, result.is_ok()).await.is_err()
                                {
                                    error!(%session_id, "failed to send action result to PIX supervisor");
                                }
                                if let Ok(guard) = result {
                                    retirement.guards.push(guard);
                                }
                            } else {
                                // Slot missing — supervisor must have already closed it.
                                // Report failure so the supervisor can transition.
                                if supervisor_handle.send_action_result(action_key, false).await.is_err() {
                                    error!(%session_id, "failed to report action result to closed PIX supervisor");
                                }
                            }
                        }
                        SessionPixAction::ReleaseService => {
                            gate.release_service();
                        }
                        SessionPixAction::ProgressNotification => {
                            gate.notify_progress();
                        }
                        SessionPixAction::RetireSsa(ssa_id) => {
                            // Release the old SSA by dropping its guard — the guard's
                            // Drop calls retire_ssa on the reconstructor automatically.
                            retirement.guards.retain(|g| *g.ssa_id() != ssa_id);
                            // Remove the corresponding deposit-observer handle.
                            if let Some(slot) = myself.sessions.get(&session_id) {
                                slot.abort_handles
                                    .lock()
                                    .abort_one(&SessionHandles::PixDepositObserver(ssa_id.ssa_index()));
                            }
                        }
                        SessionPixAction::Close(reason) => {
                            // Poison the gate so any parked writers get GateClosed.
                            gate.poison();
                            // Retire all tracked SSAs from the reconstructor.
                            retirement.retire_all();
                            let closure_reason = pix_close_to_closure_reason(reason);
                            #[cfg(all(feature = "telemetry", not(test)))]
                            METRIC_PIX_CLOSURES.increment(&[&reason.to_string()]);
                            error!(%session_id, ?reason, "pix supervisor closed session");
                            if let Some(slot) = myself.sessions.remove(&session_id) {
                                myself.active_sessions.fetch_sub(1, Ordering::Relaxed);
                                close_session(session_id, slot, closure_reason);
                            }
                            return;
                        }
                    }
                }
                // action channel closed: supervisor worker died.
                // Poison the gate so any parked writers get GateClosed.
                gate.poison();
                // Retire all tracked SSAs from the reconstructor.
                retirement.retire_all();
                #[cfg(all(feature = "telemetry", not(test)))]
                METRIC_PIX_CLOSURES.increment(&["PixFailure"]);
                if let Some(slot) = myself.sessions.remove(&session_id) {
                    myself.active_sessions.fetch_sub(1, Ordering::Relaxed);
                    close_session(session_id, slot, ClosureReason::PixFailure);
                }
            });

            /// Owned collection of [`SsaCommitmentGuard`]s for the PIX action driver.
            ///
            /// Each guard calls [`SsaReconstructor::retire_ssa`] on drop, so simply
            /// clearing the vec or letting it drop handles cleanup automatically.
            struct SsaRetirementGuard {
                guards: Vec<SsaCommitmentGuard<HoprPixSpec>>,
            }

            impl SsaRetirementGuard {
                /// Explicitly retire all tracked SSAs by clearing the guard vec.
                fn retire_all(&mut self) {
                    self.guards.clear();
                }
            }

            slot_for_driver
                .abort_handles
                .lock()
                .insert(SessionHandles::PixActionDriver, ah_action_driver);
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
        if let Some(tx_est) = self.session_initiations.remove(&est.orig_challenge) {
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
    async fn handle_session_error(&self, error_type: StartErrorType) -> errors::Result<()> {
        trace!(
            challenge = error_type.challenge,
            error = ?error_type.reason,
            "failed to initialize a session",
        );
        // Currently, we do not distinguish between individual error types
        // and just discard the initiation attempt and pass on the error.
        if let Some(tx_est) = self.session_initiations.remove(&error_type.challenge) {
            if let Err(error) = tx_est.try_send(Err(error_type)) {
                error!(%error, "could not send session error message");
                return Err(SessionManagerError::other(error).into());
            }
            error!(challenge = error_type.challenge, "session establishment error received");
        } else {
            error!(
                challenge = error_type.challenge,
                "session establishment attempt expired before error could be delivered"
            );
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_RECEIVED_SESSION_ERRS.increment(&[&error_type.reason.to_string()]);

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
        msg: SsaClientCommitmentMessage<SessionId, HoprPixGroupElement>,
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
        let Some(quota_per_ssa) = session_slot.ssa_params.get().map(|s| s.quota_per_ssa()) else {
            return Err(SessionManagerError::Other(anyhow::anyhow!("no SSA state for session {session_id}")).into());
        };

        // Insert the newly received coefficients into the SSA Reconstructor
        let pix_toolbox_for_closure = pix_toolbox.clone();
        let ssa_client_commitment_state = hopr_utils::parallelize::cpu::spawn_blocking(
            move || {
                pix_toolbox_for_closure
                    .share_processor
                    .insert_coefficient_commitments(
                        SsaId::new(pseudonym, msg.ssa_index),
                        msg.coefficient_index,
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

        let commitment_ssa_id = ssa_client_commitment_state.ssa_id;

        let (deposit_done_tx, deposit_done_rx) = futures::channel::mpsc::channel(10);

        if ssa_client_commitment_state.is_verifiable
            && let Some(deposit_address) = ssa_client_commitment_state.ssa_deposit_address
        {
            // Notify the supervisor that the commitment is verifiable.
            // On the Exit side we do not know the expected deposit amount, so
            // pass None (any amount accepted by the supervisor).
            //
            // If a min_deposit is configured, pass it as the expected amount
            // so the supervisor rejects dust confirmations.
            let min_deposit = self.cfg.pix_config.supervisor_cfg.min_deposit;
            let expected_deposit = (!min_deposit.is_zero()).then_some(min_deposit);
            if let Some(handle) = session_slot.pix_supervisor.get()
                && handle
                    .send_event(SessionPixEvent::CommitmentVerified {
                        ssa_id: commitment_ssa_id,
                        expected_deposit,
                    })
                    .await
                    .is_err()
            {
                error!(%session_id, "failed to send CommitmentVerified to PIX supervisor");
            }

            // Spawn the PixDepositObserver that forwards deposit confirmations
            // to the supervisor. This loops to support top-up deposits.
            let pix_supervisor_for_observer = session_slot.pix_supervisor.clone();
            let ssa_id_for_observer = commitment_ssa_id;
            let max_deposit_wait = self.cfg.pix_config.supervisor_cfg.max_deposit_wait;
            session_slot.abort_handles.lock().insert(
                SessionHandles::PixDepositObserver(commitment_ssa_id.ssa_index()),
                hopr_utils::spawn_as_abortable!(async move {
                    let mut deposit_stream = Box::pin(
                        deposit_done_rx.filter(|((evt_pseudonym, evt_index), _)| {
                            futures::future::ready(
                                evt_index == &ssa_id_for_observer.ssa_index()
                                    && evt_pseudonym == ssa_id_for_observer.pseudonym(),
                            )
                        }),
                    );
                    loop {
                        let result = deposit_stream
                            .next()
                            // An initial delay allows the deposit to arrive naturally
                            // before entering the timeout loop.  NOTE: if
                            // max_deposit_wait is at or below this delay, the timeout
                            // fires before the delay completes and every session with
                            // this configuration closes with DepositObserverClosed.
                            .delay(futures_time::time::Duration::from_millis(100))
                            .timeout(futures_time::time::Duration::from(max_deposit_wait))
                            .await;
                        match result {
                            Ok(Some(((..), amount))) => {
                                // Forward every deposit — the supervisor decides
                                // whether the accumulated amount is sufficient.
                                if let Some(h) = pix_supervisor_for_observer.get()
                                    && h.send_event(
                                        SessionPixEvent::DepositConfirmed {
                                            ssa_id: ssa_id_for_observer,
                                            amount,
                                        },
                                    ).await.is_err()
                                {
                                    error!(%session_id, "failed to send DepositConfirmed to PIX supervisor");
                                    break;
                                }
                            }
                            Ok(None) => {
                                error!(%session_id, "deposit channel closed without confirmation; check deposit address and funding");
                                if let Some(h) = pix_supervisor_for_observer.get()
                                    && h.send_event(SessionPixEvent::DepositObserverClosed(
                                        ssa_id_for_observer,
                                    )).await.is_err()
                                {
                                    error!(%session_id, "failed to send DepositObserverClosed to PIX supervisor");
                                }
                                break;
                            }
                            Err(_) => {
                                error!(%session_id, "deposit confirmation timed out; check deposit address and funding");
                                if let Some(h) = pix_supervisor_for_observer.get()
                                    && h.send_event(SessionPixEvent::DepositObserverClosed(
                                        ssa_id_for_observer,
                                    )).await.is_err()
                                {
                                    error!(%session_id, "failed to send DepositObserverClosed to PIX supervisor");
                                }
                                break;
                            }
                        }
                    }
                }),
            );

            // Notify upstream that deposit is needed
            pix_toolbox
                .pix_events
                .try_send(HoprSessionOutPixEvent::DepositNeeded(
                    AgreedSsaQuota {
                        ssa_id: commitment_ssa_id,
                        deposit_address,
                        quota_per_ssa,
                    },
                    deposit_done_tx,
                ))
                .map_err(|_| {
                    SessionManagerError::other(anyhow::anyhow!("failed to send pix event for needed deposit"))
                })?;
            info!(%commitment_ssa_id, %deposit_address, quota_per_ssa, "client SSA commitment verifiable, deposit needed");
        }

        Ok(())
    }

    /// Handled by the Entry, when the Exit sends PIX initiation request
    #[tracing::instrument(level = "debug", skip(self, msg))]
    async fn handle_ssa_request(
        &self,
        pseudonym: HoprPseudonym,
        msg: SsaServerCommitmentMessage<SessionId, HoprPixGroupElement>,
    ) -> errors::Result<()> {
        let Some(pix_toolbox) = self.pix_toolbox.get().cloned() else {
            return Err(SessionManagerError::UnsupportedMessage.into());
        };

        if pseudonym != msg.session_id {
            error!(%pseudonym, %msg.session_id, "received SSA server commitment for a different session");
            return Err(SessionManagerError::NonExistingSession.into());
        }

        // The SsaRequest can arrive before new_session() has finished allocating the
        // session slot, since both SessionEstablished and SsaRequest are sent by the Exit
        // back-to-back and processed concurrently by the Start protocol handler.
        // Wait event-driven for the slot to appear, with a fallback timeout.
        let session_slot = {
            let session_id = msg.session_id;
            let notify = self.slot_notify.clone();
            let start = std::time::Instant::now();
            const DEADLINE: Duration = Duration::from_millis(1000);
            loop {
                if let Some(slot) = self.sessions.get(&session_id) {
                    break slot;
                }
                if start.elapsed() >= DEADLINE {
                    error!(%session_id, "session slot not found within {DEADLINE:?}");
                    return Err(SessionManagerError::NonExistingSession.into());
                }
                // Await notification with timeout guard.
                let _ = notify
                    .notified()
                    .timeout(futures_time::time::Duration::from(SLOT_NOTIFY_TIMEOUT))
                    .await;
            }
        };

        debug!(
            num_server_commitments = msg.commitments.len(),
            "received Exit SSA commitments"
        );

        // Entry-side: store the negotiated params the first time we see an SsaRequest.
        // The Exit (which runs the supervisor) already has these from
        // handle_incoming_session_initiation; the Entry creates the slot without
        // params since the server's response determines the final values.
        let _ = session_slot.ssa_params.get_or_init(|| SessionSsaParameters {
            polys_per_ssa: msg.polys_per_ssa(),
            shares_per_poly: msg.shares_per_poly(),
        });

        let quota_per_ssa = session_slot.ssa_params.get().unwrap().quota_per_ssa();

        // We (Entry) offered some quota in the Session Initiation message, the Exit has accepted it,
        // but could have still replaced it with a different one from its range.
        let server_quota = pix_params_to_quota(msg.polys_per_ssa(), msg.shares_per_poly());
        if quota_per_ssa != server_quota {
            return Err(SessionManagerError::Unacceptable(format!(
                "Exit sent unacceptable quota {server_quota} (our is {quota_per_ssa})"
            ))
            .into());
        }

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        // Read the negotiated PIX params before the for loop (which partially moves msg)
        let _negotiated_polys = msg.polys_per_ssa();
        let _negotiated_shares = msg.shares_per_poly();

        // The server can theoretically send multiple SSA commitments
        // asking us to make the equal number of client commitments and deposits.
        // The server is authoritative in giving the ssa_index, the client only verifies if it's monotonic.
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
            let commitment_msgs = SsaClientCommitmentMessage::new_multiple(msg.session_id, client_commitment);
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
    use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
    use hopr_protocol_start::{StartProtocol, StartProtocolDiscriminants};
    use hopr_utils::network_types::prelude::SealedHost;
    use moka::future::FutureExt;
    use tokio::time::timeout;

    use super::*;
    use crate::{Capabilities, balancer::SurbBalancerConfig, types::SessionTarget};

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
        tokio::task::JoinHandle<()>,
    ) {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        let handle = tokio::task::spawn(async move {
            pin_mut!(rx);
            while let Some((routing, data)) = rx.next().await {
                sender
                    .send_message(routing, data)
                    .await
                    .expect("send message must not fail in mock");
            }
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
                ssa_params: Default::default(),
                pix_supervisor: Default::default(),
                pix_egress_gate: Default::default(),
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
        let _ = alice_handle.await;

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
        let _ = _handle.await;

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
        let _ = handle.await;

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
        let _ = alice_handle.await;
        let _ = bob_handle.await;

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
        let _ = _handle.await;

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
        let _ = _handle.await;

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
        let _ = _handle.await;

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
        let _ = _handle.await;

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
        let _ = _handle.await;

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
            challenge,
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
        let _ = _handle.await;
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
        let _ = _handle.await;

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
        let _ = _handle.await;
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
        let _ = _handle.await;

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
        let _ = _handle.await;

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
                ssa_params: Default::default(),
                pix_supervisor: Default::default(),
                pix_egress_gate: Default::default(),
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
                ssa_params: Default::default(),
                pix_supervisor: Default::default(),
                pix_egress_gate: Default::default(),
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
        let _ = _handle.await;

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
    /// 2. The test encodes `additional_data` as `(polynomials=3000, shares=3000)`, which translates to a quota of
    ///    9,000,000 — far outside the allowed range.
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

        // Encode (polys=3000, shares=3000) => quota = 9_000_000 which is way
        // outside the acceptable range of 0..=2048*1024*1024
        let additional_data = (u64::from(3000u32) << 48) | (u64::from(3000u32) << 32);

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
        assert_eq!(err.challenge, MIN_CHALLENGE);
        assert_eq!(mgr.num_active_sessions(), 0);

        bob_sender.close_channel();
        let _ = bob_handle.await;
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
        assert_eq!(err.challenge, MIN_CHALLENGE);
        assert_eq!(mgr.num_active_sessions(), 0);

        bob_sender.close_channel();
        let _ = bob_handle.await;
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
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
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
                    coefficient_commitments: HashMap::new(),
                },
            )
            .await;

        bob_sender.close_channel();
        let _ = bob_handle.await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::NonExistingSession)
        ));

        Ok(())
    }

    /// Verifies that a session is automatically closed after receiving more than the allowed number
    /// of `UnverifiableShare` PIX events (the configurable fault tolerance threshold).
    ///
    /// ## Steps
    /// 1. Bob's manager is started with a `PixToolbox` and a PIX quota config. Alice's session initiation is processed
    ///    via `handle_incoming_session_initiation`.
    /// 2. The test dispatches four `UnverifiableShare` events for the same `SsaId` in a loop, calling
    ///    `dispatch_pix_event` each time.
    /// 3. After the first 3 events, `num_active_sessions` is still 1 and `active_sessions` is non-empty — the session
    ///    remains open (below the fault threshold).
    /// 4. After the 4th event, the session is closed. `active_sessions` is empty and `num_active_sessions` is 0,
    ///    confirming the threshold was enforced.
    ///
    /// Covers per-session unverifiable share limiter through the supervisor's
    /// async action driver. Uses the interactive session setup so the supervisor
    /// is fully wired.
    #[test_log::test(tokio::test)]
    async fn session_is_closed_after_too_many_unverifiable_shares() -> anyhow::Result<()> {
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
                supervisor_cfg: SupervisorConfig {
                    max_unverifiable_shares_per_session: 3,
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
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
            },
        )
        .await?;

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(1).expect("non-zero"));
        for i in 1..=4 {
            mgr.dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShares {
                ssa_id,
                observed_total: i,
            })
            .await?;
            if i < 4 {
                assert!(
                    !mgr.active_sessions().is_empty(),
                    "session should remain open after {i} error(s)"
                );
                assert_eq!(mgr.num_active_sessions(), 1);
            }
        }

        // Yield to let the action driver process the close.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert!(
            mgr.active_sessions().is_empty(),
            "session must be closed after 4 unverifiable shares"
        );
        assert_eq!(mgr.num_active_sessions(), 0);

        bob_sender.close_channel();
        let _ = bob_handle.await;
        Ok(())
    }

    /// Verifies that when the exit/responder receives an `SsaRecovered` PIX event while the
    /// SSA is still in `AwaitingCommitment` phase (no deposit yet), no new SSA is requested.
    ///
    /// In the new supervisor model, `AlmostRecovered` only requests a next SSA when the current
    /// SSA is in `Recovering` phase. The old kill-switch path that requested a new SSA
    /// unconditionally no longer exists. Supervisor unit tests cover the `Recovering` → RequestSsa
    /// transition.
    #[test_log::test(tokio::test)]
    async fn exit_does_not_request_new_ssa_on_ssa_recovered_event() -> anyhow::Result<()> {
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

        // Accept 2 messages: SessionEstablished (1) + SsaRequest at init (2) only.
        // SsaRecovered must NOT trigger a third message.
        bob_transport.expect_send_message().times(2).returning(move |_, data| {
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
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
            },
        )
        .await?;

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::MIN);
        mgr.dispatch_pix_event(HoprSessionInPixEvent::SsaRecovered(ssa_id))
            .await?;

        bob_sender.close_channel();
        let _ = bob_handle.await;

        assert_eq!(
            sent_ssa_requests.lock().unwrap().len(),
            1,
            "expected exactly 1 SsaRequest message (only the one at init), SsaRecovered must not trigger a second"
        );

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
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
            },
        )
        .await?;

        let session_id = alice_pseudonym;

        // Server sends a quota of (10, 10) while we offered (2, 2) — should be rejected.
        let result = mgr
            .handle_ssa_request(
                alice_pseudonym,
                SsaServerCommitmentMessage::new(session_id, 10, 10, BTreeMap::new()),
            )
            .await;

        bob_sender.close_channel();
        let _ = bob_handle.await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TransportSessionError::Manager(SessionManagerError::Unacceptable(_))
        ));

        Ok(())
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

        use hopr_crypto_packet::prelude::HoprPixGroupElement;
        use hopr_protocol_pix::{
            PixGroup, PolynomialIndex, SsaGeneratorConfig, SsaReconstructor, SsaReconstructorConfig, SsaShareGenerator,
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
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
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
        mgr.handle_ssa_commit(
            alice_pseudonym,
            SsaClientCommitmentMessage {
                session_id: alice_pseudonym,
                ssa_index: SsaIndex::MIN,
                coefficient_index: 0,
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
        assert_eq!(quota.quota_per_ssa, pix_params_to_quota(2, 2));

        bob_sender.close_channel();
        let _ = bob_handle.await;

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
                supervisor_cfg: SupervisorConfig {
                    max_deposit_wait: Duration::from_millis(50),
                    max_ssa_delivery_time: Duration::from_millis(20),
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
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
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
        let _ = bob_handle.await;

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

        // polys_per_ssa > MAX_POLYS_PER_SSA (16192) with valid quota -> should reject
        let result = mgr.check_pix_params(&StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
            additional_data: (u64::from(65535u16) << 48) | (u64::from(128u16) << 32),
        });
        assert!(result.is_none(), "should reject polys_per_ssa > MAX_POLYS_PER_SSA");

        // shares_per_ssa < 2 with valid quota -> should reject
        let result = mgr.check_pix_params(&StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
            additional_data: (u64::from(1u16) << 48) | (u64::from(1u16) << 32),
        });
        assert!(result.is_none(), "should reject shares_per_ssa < 2");

        // shares_per_ssa > MAX_POLY_THRESHOLD (4096) with valid quota -> should reject
        let result = mgr.check_pix_params(&StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
            additional_data: (u64::from(1u16) << 48) | (u64::from(5000u16) << 32),
        });
        assert!(result.is_none(), "should reject shares_per_ssa > MAX_POLY_THRESHOLD");

        // Valid params should still be accepted
        let result = mgr.check_pix_params(&StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
            capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
            additional_data: (u64::from(8192u16) << 48) | (u64::from(128u16) << 32),
        });
        assert!(result.is_some(), "should accept valid params");

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
                    supervisor_cfg: SupervisorConfig {
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
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
            },
        )
        .await?;

        // Session is active
        assert_eq!(vec![alice_pseudonym], mgr.active_sessions());

        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(1).expect("non-zero"));

        // `max_unverifiable_shares_per_session = 10`, so 11 dispatch events close the session.
        // The first 10 increment the counter; the 11th exceeds the limit.
        for i in 0..=10 {
            let result = mgr
                .dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShares {
                    ssa_id,
                    observed_total: i as u64 + 1,
                })
                .await;
            // The first 10 succeed; the 11th also "succeeds" because closing the session returns Ok.
            assert!(result.is_ok(), "dispatch_pix_event should not return an error");
        }

        // Yield to let the action driver process the close.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Session should be closed after too many unverifiable shares.
        assert!(
            mgr.active_sessions().is_empty(),
            "session should be closed after too many unverifiable shares"
        );
        assert_eq!(mgr.num_active_sessions(), 0);

        bob_sender.close_channel();
        let _ = bob_handle.await;

        Ok(())
    }

    /// Verifies that closing a session due to unverifiable shares also poisons the
    /// egress gate, so that any writer parked on the gate receives `GateClosed`.
    ///
    /// ## Steps
    /// 1. Bob's manager establishes a PIX session with Alice.
    /// 2. Unverifiable shares are dispatched until the session closes.
    /// 3. The egress gate is checked after close: `try_acquire_sync` returns `Err`.
    #[test_log::test(tokio::test)]
    async fn unverifiable_shares_close_poisons_gate_and_wakes_writers() -> anyhow::Result<()> {
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
                supervisor_cfg: SupervisorConfig {
                    max_unverifiable_shares_per_session: 0,
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
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
            },
        )
        .await?;

        // Grab the egress gate reference while the session is live.
        let slot = mgr.sessions.get(&alice_pseudonym).expect("session must exist");
        let gate = slot.pix_egress_gate.get().cloned().expect("gate must be set");
        assert!(gate.try_acquire_sync().is_ok(), "gate should be accept before close");

        // Dispatch one unverifiable share → triggers close (max=1).
        let ssa_id = SsaId::new(alice_pseudonym, SsaIndex::new(1).expect("non-zero"));
        mgr.dispatch_pix_event(HoprSessionInPixEvent::UnverifiableShares {
            ssa_id,
            observed_total: 1,
        })
        .await?;

        // Yield to let the action driver process the close.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert!(
            mgr.active_sessions().is_empty(),
            "session must be closed after 1 unverifiable share"
        );
        assert!(
            gate.try_acquire_sync().is_err(),
            "gate must be poisoned after session close"
        );

        bob_sender.close_channel();
        let _ = bob_handle.await;

        Ok(())
    }

    /// Verifies that evicting an idle PIX session also poisons its egress gate.
    ///
    /// ## Steps
    /// 1. Bob's manager is configured with a short `idle_timeout`.
    /// 2. Alice's PIX session is established normally.
    /// 3. The test waits past the idle timeout.
    /// 4. The session is evicted, and the gate is poisoned.
    #[test_log::test(tokio::test)]
    async fn eviction_closes_pix_session_and_poisons_gate() -> anyhow::Result<()> {
        use hopr_protocol_pix::{SsaGeneratorConfig, SsaReconstructorConfig};
        use hopr_protocol_start::StartInitiation;

        let ssa_gen_config = SsaGeneratorConfig {
            polynomials_per_ssa: 2,
            threshold: 2,
            surplus_shares: 1,
        };

        let (pix_toolbox, _) = PixToolbox::new(
            SsaShareGenerator::new(ssa_gen_config).into(),
            SsaReconstructor::new(SsaReconstructorConfig {
                // horizon = 3600 + 3600 + 30 = 7230, so lifetime must be > 7230.
                ssa_counter_lifetime_secs: 8000,
                ..Default::default()
            })
            .into(),
        );

        let mgr = SessionManager::new(SessionManagerConfig {
            maximum_sessions: 1,
            idle_timeout: Duration::from_millis(100),
            pix_config: IncomingSessionPixConfig {
                quota_range: 0..=1024 * 1024 * 1024,
                supervisor_cfg: SupervisorConfig {
                    max_deposit_wait: Duration::from_secs(3600),
                    max_ssa_delivery_time: Duration::from_secs(3600),
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
        mgr.start(bob_sender.clone(), new_session_tx, Some(pix_toolbox))?;

        let alice_pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            alice_pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: HoprSessionCapabilities(Capability::UsePIX.into()),
                additional_data: (u64::from(2u32) << 48) | (u64::from(2u32) << 32),
            },
        )
        .await?;

        // Grab the egress gate reference while the session is live.
        let slot = mgr.sessions.get(&alice_pseudonym).expect("session must exist");
        let gate = slot.pix_egress_gate.get().cloned().expect("gate must be set");
        assert!(gate.try_acquire_sync().is_ok(), "gate should be accept before eviction");

        // Wait past the idle timeout.
        tokio::time::sleep(Duration::from_millis(200)).await;
        mgr.sessions.run_pending_tasks();

        assert!(
            mgr.active_sessions().is_empty(),
            "session should be evicted after idle timeout"
        );
        // close_session now poisons the gate before aborting tasks.
        assert!(
            gate.try_acquire_sync().is_err(),
            "gate should be poisoned after eviction"
        );

        bob_sender.close_channel();
        let _ = bob_handle.await;

        Ok(())
    }
}
