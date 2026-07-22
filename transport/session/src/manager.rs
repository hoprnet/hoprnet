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
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_protocol_app::prelude::*;
use hopr_protocol_start::{
    KeepAliveFlag, KeepAliveMessage, StartChallenge, StartErrorReason, StartErrorType, StartEstablished,
    StartInitiation,
};
use hopr_utils::runtime::AbortableList;
use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "telemetry")]
use crate::telemetry::{
    SessionLifecycleState, initialize_session_metrics, remove_session_metrics_state, set_session_balancer_data,
    set_session_state,
};
use crate::{
    Capability, HoprSession, IncomingSession, SESSION_MTU, SessionClientConfig, SessionId, SessionTarget,
    SurbBalancerConfig,
    balancer::{
        AtomicSurbFlowEstimator, BalancerStateValues, RateController, RateLimitSinkExt, SurbBalancer,
        SurbControllerWithCorrection,
        pid::{PidBalancerController, PidControllerGains},
        simple::SimpleBalancerController,
    },
    errors::{SessionManagerError, TransportSessionError},
    types::{ByteCapabilities, ClosureReason, HoprSessionConfig, HoprStartProtocol, SESSION_APPLICATION_TAG},
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

/// How many packets can be buffered if the HoprSession socket is not fast enough.
#[allow(dead_code)]
pub const SESSION_FORWARD_CAPACITY: usize = 10000;

// Needs to use an UnboundedSender instead of oneshot
// because Moka cache requires the value to be Clone, which oneshot Sender is not.
// It also cannot be enclosed in an Arc, since calling `send` consumes the oneshot Sender.
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
    /// These notifications are the only absolute correction of the Entry's dead-reckoned
    /// estimate of the Exit's SURB buffer. Without them, every packet lost in either
    /// direction permanently inflates the Entry's estimate, until the Exit silently runs
    /// out of SURBs and can no longer send any reply data.
    ///
    /// Default is 60 seconds (None disables the notifications), minimum is 1 second.
    #[default(Some(Duration::from_secs(60)))]
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
    /// Default is 10_000.
    #[default(10000)]
    pub session_forward_capacity: usize,
}

// Type-erased sink used by the `SessionManager` to notify about newly incoming sessions.
// The errors produced by the underlying sink are remapped into `SessionManagerError`.
type IncomingSessionSink = Pin<Box<dyn Sink<IncomingSession, Error = SessionManagerError> + Send>>;

type SessionNotifiers = (
    Arc<hopr_utils::runtime::prelude::Mutex<IncomingSessionSink>>,
    crossfire::MTx<crossfire::mpsc::Array<(SessionId, ClosureReason)>>,
);

// Sink for processing Start protocol messages.
// Must be within Arc to be shared across SessionManager clones.
// The inner OnceLock is set once in `start()` and read in `dispatch_message`.
type StartProtocolMsgSink = Arc<OnceLock<crossfire::MTx<crossfire::mpsc::Array<(HoprPseudonym, HoprStartProtocol)>>>>;

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
pub struct SessionManager<S> {
    session_initiations: SessionInitiationCache,
    session_notifiers: Arc<OnceLock<SessionNotifiers>>,
    start_protocol_tx: StartProtocolMsgSink,
    /// Authoritative session count for admission control.
    /// Incremented atomically inside `allocate_session_slot` before the cache insertion,
    /// and decremented at every removal path (explicit close, eviction, guard rollback).
    active_sessions: Arc<std::sync::atomic::AtomicUsize>,
    sessions: moka::sync::Cache<SessionId, SessionSlot>,
    msg_sender: Arc<OnceLock<S>>,
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
        }
    }
}

fn session_config(cfg: &SessionManagerConfig, capabilities: crate::Capabilities) -> HoprSessionConfig {
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
    capabilities: crate::Capabilities,
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
) -> crate::errors::Result<()>
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
            session_notifiers: Arc::new(OnceLock::new()),
            start_protocol_tx: Arc::new(OnceLock::new()),
            active_sessions,
            cfg,
        }
    }

    /// Starts the instance with the given `msg_sender` `Sink`
    /// and a channel `new_session_notifier` used to notify when a new incoming session is opened to us.
    ///
    /// This method must be called prior to any calls to [`SessionManager::new_session`] or
    /// [`SessionManager::dispatch_message`].
    pub fn start<T>(&self, msg_sender: S, new_session_notifier: T) -> crate::errors::Result<Vec<AbortHandle>>
    where
        T: futures::Sink<IncomingSession> + Send + 'static,
        T::Error: std::error::Error + Send + Sync + 'static,
    {
        self.msg_sender
            .set(msg_sender)
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        // Re-map the user-provided sink errors to `SessionManagerError` and erase the concrete
        // type, so that the `SessionManager` does not need to be generic over it. This also avoids
        // having to spawn a separate task to forward items between channels: senders simply lock
        // the sink and send directly.
        let new_session_notifier: IncomingSessionSink =
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
    ) -> crate::errors::Result<HoprSession> {
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

        // Prepare the session initiation message in the Start protocol
        trace!(challenge, ?cfg, "initiating session with config");
        let start_session_msg = HoprStartProtocol::StartSession(StartInitiation {
            challenge,
            target,
            capabilities: ByteCapabilities(cfg.capabilities),
            additional_data: if !cfg.capabilities.contains(Capability::NoRateControl) {
                cfg.surb_management
                    .map(|c| c.target_surb_buffer_size)
                    .unwrap_or(
                        self.cfg.initial_return_session_egress_rate as u64
                            * self
                                .cfg
                                .minimum_surb_buffer_duration
                                .max(MIN_SURB_BUFFER_DURATION)
                                .as_secs(),
                    )
                    .min(u32::MAX as u64) as u32
            } else {
                0
            },
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
                            crate::telemetry::record_session_surb_produced(&session_id, produced);
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
                                crate::telemetry::record_session_surb_consumed(&session_id, 1);
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
    pub async fn ping_session(&self, id: &SessionId) -> crate::errors::Result<()> {
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
    pub fn update_surb_balancer_config(&self, id: &SessionId, config: SurbBalancerConfig) -> crate::errors::Result<()> {
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
    pub fn get_surb_balancer_config(&self, id: &SessionId) -> crate::errors::Result<Option<SurbBalancerConfig>> {
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
    pub fn get_surb_level_estimates(&self, id: &SessionId) -> crate::errors::Result<(u64, u64)> {
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
    ) -> crate::errors::Result<DispatchResult> {
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
                        hopr_utils::parallelize::SESSION_INBOX_DROPS
                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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
        };
        self.sessions.insert(session_id, slot);
        session_rx
    }

    async fn handle_incoming_session_initiation(
        &self,
        pseudonym: HoprPseudonym,
        session_req: StartInitiation<SessionTarget, ByteCapabilities>,
    ) -> crate::errors::Result<()> {
        trace!(challenge = session_req.challenge, "received session initiation request");

        debug!(%pseudonym, "got new session request, searching for a free session slot");

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        let (new_session_notifier, close_session_notifier) = self
            .session_notifiers
            .get()
            .cloned()
            .ok_or(SessionManagerError::NotStarted)?;

        // Reply routing uses SURBs only with the pseudonym of this Session's ID
        let reply_routing = DestinationRouting::Return(pseudonym.into());

        // Use constant application tag for all sessions
        self.sessions.run_pending_tasks();

        // A repeated initiation for a pseudonym that already has a Session means the
        // initiator has lost or abandoned its side of it (it never received our
        // SessionEstablished reply, or it reuses its pseudonym on reconnect).
        // The pseudonym is known only to the initiator, so the existing Session cannot
        // serve anyone else anymore: close it and let this initiation take the slot over.
        // Otherwise, re-initiations would keep being rejected with NoSlotsAvailable
        // until the stale Session gets evicted by the idle timeout.
        if let Some(stale_slot) = self.sessions.remove(&pseudonym) {
            self.active_sessions.fetch_sub(1, Ordering::Relaxed);
            info!(%pseudonym, "closing stale session superseded by a new initiation with the same pseudonym");
            close_session(pseudonym, stale_slot, ClosureReason::Eviction);
        }

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
        };
        slot.abort_handles.lock().insert(SessionHandles::Ingress, session_rx_ah);

        // Insert the slot and obtain a guard. Any failure from here on rolls the slot
        // back, otherwise it would block this pseudonym until idle eviction. The atomic
        // insert (inside the helper) also prevents a TOCTOU race, so only one concurrent
        // request can claim the slot for a given pseudonym.
        let Some(mut slot_guard) = self.allocate_session_slot(session_id, slot.clone()) else {
            // Either the maximum number of sessions has been reached, or a concurrent
            // initiation for the same pseudonym has claimed the slot first.
            error!(%pseudonym, "no session slot available");
            let reason = StartErrorReason::NoSlotsAvailable;
            let data = HoprStartProtocol::SessionError(StartErrorType {
                challenge: session_req.challenge,
                reason,
            });
            send_via_msg_sender(&mut msg_sender, reply_routing.clone(), data, "session error message").await?;
            return Ok(());
        };

        debug!(?pseudonym, ?session_req, "assigned a new session");

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
            let target_surb_buffer_size = if session_req.additional_data > 0 {
                (session_req.additional_data as u64).min(self.cfg.maximum_surb_buffer_size as u64)
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
                            crate::telemetry::record_session_surb_consumed(&session_id, 1);
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
                        crate::telemetry::record_session_surb_produced(&session_id, produced);
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
                            crate::telemetry::record_session_surb_consumed(&session_id, 1);
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

        info!(%session_id, "new session established");

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_NUM_ESTABLISHED_SESSIONS.increment();

        slot_guard.commit();
        Ok(())
    }

    async fn handle_session_established(&self, est: StartEstablished<SessionId>) -> crate::errors::Result<()> {
        trace!(
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

    async fn handle_session_error(&self, error_type: StartErrorType) -> crate::errors::Result<()> {
        trace!(
            challenge = error_type.challenge,
            error = ?error_type.reason,
            "failed to initialize a session",
        );
        // Currently, we do not distinguish between individual error types
        // and just discard the initiation attempt and pass on the error.
        if let Some(tx_est) = self.session_initiations.remove(&error_type.challenge) {
            if let Err(error) = tx_est.try_send(Err(error_type)) {
                error!(%error, ?error_type, "could not send session error message");
                return Err(SessionManagerError::other(error).into());
            }
            error!(
                challenge = error_type.challenge,
                ?error_type,
                "session establishment error received"
            );
        } else {
            error!(
                challenge = error_type.challenge,
                ?error_type,
                "session establishment attempt expired before error could be delivered"
            );
        }

        #[cfg(all(feature = "telemetry", not(test)))]
        METRIC_RECEIVED_SESSION_ERRS.increment(&[&error_type.reason.to_string()]);

        Ok(())
    }

    async fn handle_keep_alive(&self, msg: KeepAliveMessage<SessionId>) -> crate::errors::Result<()> {
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
                    crate::telemetry::record_session_surb_consumed(&session_id, 1);
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
                    crate::telemetry::record_session_surb_produced(&session_id, produced);
                }
            }
        } else {
            debug!(%session_id, "received keep-alive request for an unknown session");
        }
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

    #[test_log::test(tokio::test)]
    async fn session_manager_should_follow_start_protocol_to_establish_new_session_and_close_it() -> anyhow::Result<()>
    {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let alice_mgr = SessionManager::new(Default::default());
        let bob_mgr = SessionManager::new(Default::default());

        let mut sequence = mockall::Sequence::new();
        let mut alice_transport = MockMsgSender::new();
        let mut bob_transport = MockMsgSender::new();

        // Alice sends the StartSession message
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                info!("alice sends {}", data.data.application_tag);
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
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                info!("bob sends {}", data.data.application_tag);
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
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

        // Alice sends the terminating segment to close the Session
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                hopr_protocol_session::types::SessionMessage::<{ ApplicationData::PAYLOAD_SIZE }>::try_from(
                    data.data.plain_text.as_ref(),
                )
                .expect("must be a session message")
                .try_as_segment()
                .expect("must be a segment")
                .is_terminating()
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
        ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice)?);
        assert!(alice_mgr.is_started());

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob)?);
        assert!(bob_mgr.is_started());

        let target = SealedHost::Plain("127.0.0.1:80".parse()?);

        pin_mut!(new_session_rx_bob);
        let (alice_session, bob_session) = timeout(
            Duration::from_secs(2),
            futures::future::join(
                alice_mgr.new_session(
                    bob_peer,
                    SessionTarget::TcpStream(target.clone()),
                    SessionClientConfig {
                        pseudonym: alice_pseudonym.into(),
                        capabilities: Capability::NoRateControl | Capability::Segmentation,
                        surb_management: None,
                        ..Default::default()
                    },
                ),
                new_session_rx_bob.next(),
            ),
        )
        .await?;

        let mut alice_session = alice_session?;
        let bob_session = bob_session.ok_or(anyhow!("bob must get an incoming session"))?;

        assert_eq!(
            alice_session.config().capabilities,
            Capability::Segmentation | Capability::NoRateControl
        );
        assert_eq!(
            alice_session.config().capabilities,
            bob_session.session.config().capabilities
        );
        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        assert_eq!(vec![*alice_session.id()], alice_mgr.active_sessions());
        assert_eq!(None, alice_mgr.get_surb_balancer_config(alice_session.id())?);
        assert!(
            alice_mgr
                .update_surb_balancer_config(alice_session.id(), SurbBalancerConfig::default())
                .is_err()
        );

        assert_eq!(vec![*bob_session.session.id()], bob_mgr.active_sessions());
        assert_eq!(None, bob_mgr.get_surb_balancer_config(bob_session.session.id())?);
        assert!(
            bob_mgr
                .update_surb_balancer_config(bob_session.session.id(), SurbBalancerConfig::default())
                .is_err()
        );

        tokio::time::sleep(Duration::from_millis(100)).await;
        alice_session.close().await?;

        tokio::time::sleep(Duration::from_millis(100)).await;

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

    #[test_log::test(tokio::test)]
    async fn session_manager_should_close_idle_session_automatically() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let cfg = SessionManagerConfig {
            idle_timeout: Duration::from_millis(200),
            ..Default::default()
        };

        let alice_mgr = SessionManager::new(cfg);
        let bob_mgr = SessionManager::new(Default::default());

        let mut sequence = mockall::Sequence::new();
        let mut alice_transport = MockMsgSender::new();
        let mut bob_transport = MockMsgSender::new();

        // Alice sends the StartSession message
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
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
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
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

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        let (alice_sender, alice_handle) = mock_packet_planning(alice_transport);
        ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob)?);
        assert!(bob_mgr.is_started());

        let target = SealedHost::Plain("127.0.0.1:80".parse()?);

        pin_mut!(new_session_rx_bob);
        let (alice_session, bob_session) = timeout(
            Duration::from_secs(2),
            futures::future::join(
                alice_mgr.new_session(
                    bob_peer,
                    SessionTarget::TcpStream(target.clone()),
                    SessionClientConfig {
                        pseudonym: alice_pseudonym.into(),
                        capabilities: Capability::NoRateControl | Capability::Segmentation,
                        surb_management: None,
                        ..Default::default()
                    },
                ),
                new_session_rx_bob.next(),
            ),
        )
        .await?;

        let alice_session = alice_session?;
        let bob_session = bob_session.ok_or(anyhow!("bob must get an incoming session"))?;

        assert_eq!(
            alice_session.config().capabilities,
            Capability::Segmentation | Capability::NoRateControl,
        );
        assert_eq!(
            alice_session.config().capabilities,
            bob_session.session.config().capabilities
        );
        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        // Let the session timeout at Alice
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
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
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
        alice_mgr.start(alice_sender.clone(), new_session_tx_alice)?;
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

        drop(new_session_rx_alice);

        // Cleanup: close sender and await handle
        alice_sender.close_channel();
        let _ = alice_handle.await;

        Ok(())
    }

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
        alice_mgr.start(alice_sender.clone(), new_session_tx_alice)?;
        assert!(alice_mgr.is_started());

        // Start Bob
        let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
        let (bob_sender, _bob_handle) = mock_packet_planning(bob_transport);
        bob_mgr.start(bob_sender.clone(), new_session_tx_bob)?;
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

        Ok(())
    }

    #[cfg(feature = "telemetry")]
    #[test_log::test(tokio::test)]
    async fn failed_incoming_session_establishment_does_not_register_telemetry() -> anyhow::Result<()> {
        let mgr = SessionManager::new(Default::default());

        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        drop(new_session_rx);
        let (sender, _handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        let pseudonym = HoprPseudonym::random();
        let result = mgr
            .handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: ByteCapabilities(Capabilities::empty()),
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

        // Cleanup: close sender and await handle
        sender.close_channel();
        let _ = _handle.await;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn session_manager_should_roll_back_slot_when_incoming_session_setup_fails() -> anyhow::Result<()> {
        let mgr = SessionManager::new(Default::default());

        // Drop the receiver so that notifying about the new incoming session fails
        // *after* the session slot has already been inserted into the cache.
        let transport = MockMsgSender::new();
        let (new_session_tx, new_session_rx) = futures::channel::mpsc::channel(1);
        drop(new_session_rx);
        let (sender, handle) = mock_packet_planning(transport);
        mgr.start(sender.clone(), new_session_tx)?;
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
                    capabilities: ByteCapabilities(Capabilities::empty()),
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
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
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
            .times(5..)
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
            .times(5..)
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
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
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
        ahs.extend(alice_mgr.start(alice_sender.clone(), new_session_tx_alice)?);
        assert!(alice_mgr.is_started());

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        let (bob_sender, bob_handle) = mock_packet_planning(bob_transport);
        ahs.extend(bob_mgr.start(bob_sender.clone(), new_session_tx_bob)?);
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

    #[test_log::test(tokio::test)]
    async fn session_manager_should_supersede_stale_session_on_reinitiation_with_same_pseudonym() -> anyhow::Result<()>
    {
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
        bob_mgr.start(sender.clone(), new_session_tx)?;
        assert!(bob_mgr.is_started());

        let pseudonym = HoprPseudonym::random();

        // First session initiation - should succeed
        let result = bob_mgr
            .handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: ByteCapabilities(Capabilities::empty()),
                    additional_data: 0,
                },
            )
            .await;

        assert!(result.is_ok(), "first session initiation should succeed");

        // Verify one session exists
        let active = bob_mgr.active_sessions();
        assert_eq!(active.len(), 1, "should have exactly one active session");

        // Second session initiation with the same pseudonym: the stale session is
        // closed and the new initiation takes the slot over (a re-initiation means
        // the initiator has lost or abandoned its side of the old session).
        let result = bob_mgr
            .handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE + 1,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: ByteCapabilities(Capabilities::empty()),
                    additional_data: 0,
                },
            )
            .await;

        assert!(result.is_ok(), "re-initiation should supersede the stale session");

        // The stale session must have been replaced, not duplicated
        let active = bob_mgr.active_sessions();
        assert_eq!(active.len(), 1, "should still have exactly one active session");

        // Cleanup: close sender and await handle
        sender.close_channel();
        let _ = _handle.await;

        Ok(())
    }

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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
        let result = mgr.close_session(&fake_session_id);

        assert!(!result, "closing non-existent session should return false");

        Ok(())
    }

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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        let fake_session_id = HoprPseudonym::random();
        let result = mgr.update_surb_balancer_config(&fake_session_id, SurbBalancerConfig::default());

        assert!(result.is_err());

        // Cleanup: close sender and await handle
        sender.close_channel();
        let _ = _handle.await;

        Ok(())
    }

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
        mgr.start(sender.clone(), new_session_tx)?;
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
        mgr.start(sender.clone(), new_session_tx)?;
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
        mgr.start(sender.clone(), new_session_tx)?;
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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        // First session - should succeed
        let pseudonym1 = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym1,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: ByteCapabilities(Capabilities::empty()),
                additional_data: 0,
            },
        )
        .await?;

        // Verify one session exists
        assert_eq!(mgr.active_sessions().len(), 1);

        // Second session - should fail with TooManySessions
        let pseudonym2 = HoprPseudonym::random();
        let _result = mgr
            .handle_incoming_session_initiation(
                pseudonym2,
                StartInitiation {
                    challenge: MIN_CHALLENGE,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: ByteCapabilities(Capabilities::empty()),
                    additional_data: 0,
                },
            )
            .await;

        // The error is handled internally (sends SessionError), so result is Ok
        // But we can verify no new session was added
        assert_eq!(mgr.active_sessions().len(), 1);

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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        // Fill the cache with two incoming sessions (Exits).
        for i in 0..2 {
            let pseudonym = HoprPseudonym::random();
            mgr.handle_incoming_session_initiation(
                pseudonym,
                StartInitiation {
                    challenge: MIN_CHALLENGE + i as u64,
                    target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                    capabilities: ByteCapabilities(Capabilities::empty()),
                    additional_data: 0,
                },
            )
            .await?;
        }
        assert_eq!(mgr.active_sessions().len(), 2);

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
        mgr.start(tx, new_session_tx)?;
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
        alice_mgr.start(alice_sender.clone(), new_session_tx_alice)?;
        assert!(alice_mgr.is_started());

        let (bob_sender, _bob_handle) = mock_packet_planning(bob_transport);
        let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
        bob_mgr.start(bob_sender.clone(), new_session_tx_bob)?;
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
        // The pending challenge must have been removed from `session_initiations`
        // after the timeout error propagated.
        assert_eq!(
            alice_mgr.session_initiations.entry_count(),
            0,
            "session_initiations was not cleaned up after timeout"
        );

        Ok(())
    }

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
        mgr.start(sender.clone(), new_session_tx)?;
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

        // Cleanup: close sender and await handle
        sender.close_channel();
        let _ = _handle.await;

        Ok(())
    }

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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        // Create a session
        let pseudonym = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: ByteCapabilities(Capabilities::empty()),
                additional_data: 0,
            },
        )
        .await?;

        // Verify session exists
        assert_eq!(mgr.active_sessions().len(), 1);

        // Close the session - should return true
        let result = mgr.close_session(&pseudonym);
        assert!(result, "closing existing session should return true");

        // Verify session is closed
        assert_eq!(mgr.active_sessions().len(), 0);

        // Cleanup: close sender and await handle
        sender.close_channel();
        let _ = _handle.await;

        Ok(())
    }

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
        let _ahs = alice_mgr.start(mock_sender, new_session_tx)?;
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
        let _ahs = alice_mgr.start(mock_sender, new_session_tx)?;
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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        // Create first session
        let pseudonym1 = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym1,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: ByteCapabilities(Capabilities::empty()),
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
        mgr.start(sender.clone(), new_session_tx)?;
        assert!(mgr.is_started());

        // Create first session
        let pseudonym1 = HoprPseudonym::random();
        mgr.handle_incoming_session_initiation(
            pseudonym1,
            StartInitiation {
                challenge: MIN_CHALLENGE,
                target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                capabilities: ByteCapabilities(Capabilities::empty()),
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
                    capabilities: ByteCapabilities(Capabilities::empty()),
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
}
