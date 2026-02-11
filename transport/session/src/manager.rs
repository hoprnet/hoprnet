use std::{
    ops::Range,
    sync::{Arc, OnceLock},
    time::Duration,
};

use futures::{
    FutureExt, SinkExt, StreamExt, TryStreamExt,
    channel::mpsc::{Sender, UnboundedSender},
    future::AbortHandle,
    pin_mut,
};
use futures_time::future::FutureExt as TimeExt;
use hopr_crypto_random::Randomizable;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::Address;
use hopr_protocol_app::prelude::*;
use hopr_protocol_start::{
    KeepAliveMessage, StartChallenge, StartErrorReason, StartErrorType, StartEstablished, StartInitiation,
};
use tracing::{debug, error, info, trace, warn};

#[cfg(feature = "stats")]
use crate::stats::{SessionLifecycleState, SessionStats, SessionStatsSnapshot};
use crate::{
    Capability, HoprSession, IncomingSession, SESSION_MTU, SessionClientConfig, SessionId, SessionTarget,
    SurbBalancerConfig,
    balancer::{
        AtomicSurbFlowEstimator, BalancerConfigFeedback, RateController, RateLimitSinkExt, SurbBalancer,
        SurbControllerWithCorrection,
        pid::{PidBalancerController, PidControllerGains},
        simple::SimpleBalancerController,
    },
    errors::{SessionManagerError, TransportSessionError},
    types::{ByteCapabilities, ClosureReason, HoprSessionConfig, HoprStartProtocol},
    utils,
    utils::insert_into_next_slot,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ACTIVE_SESSIONS: hopr_metrics::SimpleGauge = hopr_metrics::SimpleGauge::new(
        "hopr_session_num_active_sessions",
        "Number of currently active HOPR sessions"
    ).unwrap();
    static ref METRIC_NUM_ESTABLISHED_SESSIONS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
        "hopr_session_established_sessions_count",
        "Number of sessions that were successfully established as an Exit node"
    ).unwrap();
    static ref METRIC_NUM_INITIATED_SESSIONS: hopr_metrics::SimpleCounter = hopr_metrics::SimpleCounter::new(
        "hopr_session_initiated_sessions_count",
        "Number of sessions that were successfully initiated as an Entry node"
    ).unwrap();
    static ref METRIC_RECEIVED_SESSION_ERRS: hopr_metrics::MultiCounter = hopr_metrics::MultiCounter::new(
        "hopr_session_received_error_count",
        "Number of HOPR session errors received from an Exit node",
        &["kind"]
    ).unwrap();
    static ref METRIC_SENT_SESSION_ERRS: hopr_metrics::MultiCounter = hopr_metrics::MultiCounter::new(
        "hopr_session_sent_error_count",
        "Number of HOPR session errors sent to an Entry node",
        &["kind"]
    ).unwrap();
}

fn close_session(session_id: SessionId, session_data: SessionSlot, reason: ClosureReason) {
    debug!(?session_id, ?reason, "closing session");

    #[cfg(feature = "stats")]
    {
        session_data.stats.set_state(SessionLifecycleState::Closed);
        session_data.stats.touch_activity();
    }

    if reason != ClosureReason::EmptyRead {
        // Closing the data sender will also cause it to close from the read side
        session_data.session_tx.close_channel();
        trace!(?session_id, "data tx channel closed on session");
    }

    // Terminate any additional tasks spawned by the Session
    session_data.abort_handles.into_iter().for_each(|h| h.abort());

    #[cfg(all(feature = "prometheus", not(test)))]
    METRIC_ACTIVE_SESSIONS.decrement(1.0);
}

fn initiation_timeout_max_one_way(base: Duration, hops: usize) -> Duration {
    base * (hops as u32)
}

/// Minimum time the SURB buffer must endure if no SURBs are being produced.
pub const MIN_SURB_BUFFER_DURATION: Duration = Duration::from_secs(1);

/// The first challenge value used in Start protocol to initiate a session.
pub(crate) const MIN_CHALLENGE: StartChallenge = 1;

/// Maximum time to wait for counterparty to receive the target number of SURBs.
const SESSION_READINESS_TIMEOUT: Duration = Duration::from_secs(10);

/// Minimum timeout until an unfinished frame is discarded.
const MIN_FRAME_TIMEOUT: Duration = Duration::from_millis(10);

// Needs to use an UnboundedSender instead of oneshot
// because Moka cache requires the value to be Clone, which oneshot Sender is not.
// It also cannot be enclosed in an Arc, since calling `send` consumes the oneshot Sender.
type SessionInitiationCache =
    moka::future::Cache<StartChallenge, UnboundedSender<Result<StartEstablished<SessionId>, StartErrorType>>>;

#[derive(Clone)]
struct SessionSlot {
    // Sender needs to be put in Arc, so that no clones are made by `moka`.
    // This makes sure that the entire channel closes once the one and only sender is closed.
    session_tx: Arc<UnboundedSender<ApplicationDataIn>>,
    routing_opts: DestinationRouting,
    abort_handles: Vec<AbortHandle>,
    #[cfg(feature = "stats")]
    stats: Arc<SessionStats>,
    // Allows reconfiguring of the SURB balancer on-the-fly
    // Set on both Entry and Exit sides.
    surb_mgmt: Option<SurbBalancerConfig>,
    // Set only on the Exit side.
    surb_estimator: Option<AtomicSurbFlowEstimator>,
}

/// Allows forward and backward propagation of SURB management configuration changes.
struct SessionCacheBalancerFeedback(moka::future::Cache<SessionId, SessionSlot>);

#[async_trait::async_trait]
impl BalancerConfigFeedback for SessionCacheBalancerFeedback {
    async fn get_config(&self, id: &SessionId) -> crate::errors::Result<SurbBalancerConfig> {
        // Intentionally using `iter().find()` instead of `get()` here,
        // so that the popularity estimator is not hit.
        self.0
            .iter()
            .find(|(sid, _)| sid.as_ref() == id)
            .ok_or(SessionManagerError::NonExistingSession)?
            .1
            .surb_mgmt
            .ok_or(SessionManagerError::Other("missing surb balancer config".into()).into())
    }

    async fn on_config_update(&self, id: &SessionId, cfg: SurbBalancerConfig) -> crate::errors::Result<()> {
        if let moka::ops::compute::CompResult::ReplacedWith(_) = self
            .0
            .entry_by_ref(id)
            .and_compute_with(|entry| {
                futures::future::ready(match entry.map(|e| e.into_value()) {
                    None => moka::ops::compute::Op::Nop,
                    Some(mut updated_slot) => {
                        if let Some(balancer_cfg) = &mut updated_slot.surb_mgmt {
                            *balancer_cfg = cfg;
                            moka::ops::compute::Op::Put(updated_slot)
                        } else {
                            moka::ops::compute::Op::Nop
                        }
                    }
                })
            })
            .await
        {
            Ok(())
        } else {
            Err(SessionManagerError::NonExistingSession.into())
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
    /// Ranges of tags available for Sessions.
    ///
    /// **NOTE**: If the range starts lower than [`ReservedTag`]'s range end,
    /// it will be automatically transformed to start at this value.
    /// This is due to the reserved range by the Start sub-protocol.
    ///
    /// Default is 16..1024.
    #[doc(hidden)]
    #[default(_code = "16u64..1024u64")]
    pub session_tag_range: Range<u64>,

    /// The maximum number of sessions (incoming and outgoing) that is allowed
    /// to be managed by the manager.
    ///
    /// When reached, creating [new sessions](SessionManager::new_session) will return
    /// the [`SessionManagerError::TooManySessions`] error, and incoming sessions will be rejected
    /// with [`StartErrorReason::NoSlotsAvailable`] Start protocol error.
    ///
    /// Default is 128, minimum is 1; maximum is given by the `session_tag_range`.
    #[default(128)]
    pub maximum_sessions: usize,

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

    /// Determines if the `target_surb_buffer_size` can grow if the simple moving average of the
    /// current SURB buffer size estimate surpasses the given threshold over the given period.
    ///
    /// This only applies if the used [`SurbBalancerController`] has this option.
    ///
    /// The default is `(60, 0.20)` (20% growth if the average SURB buffer size surpasses 20%
    /// of the target buffer size in a 60-second window).
    #[default(_code = "Some((Duration::from_secs(60), 0.20))")]
    pub growable_target_surb_buffer: Option<(Duration, f64)>,
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
/// There are 4 different scenarios of local vs. remote SURB balancing configuration, but
/// an equilibrium (= matching the SURB production and consumption) is most likely to be reached
/// only when both are configured (the ideal case below):
///
/// #### 1. Ideal local and remote SURB balancing
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
/// ### Automatic `target_surb_buffer_size` increase
/// This mechanism only applies to the Session recipient (Exit) and on Sessions without the
/// [`Capability::NoRateControl`] flag set.
/// In this case, the Exit throttles the Session egress based on the ratio between
/// of the estimated SURB balance and the *hint* of Entry's `target_surb_buffer_size` set during the Session initiation.
///
/// However, as the Entry might [increase](SessionManager::update_surb_balancer_config) the `target_surb_buffer_size`
/// of the Session dynamically, the new value is never hinted again to the Exit (this only happens once during
/// the Session initiation).
/// For this reason, the Exit then might observe the ratio going higher than 1. When this happens consistently
/// over some given time period, the Exit can decide to increase the initial hint to the newly observed value.
///
/// See the `growable_target_surb_buffer` field in the [`SessionManagerConfig`] for details.
pub struct SessionManager<S, T> {
    session_initiations: SessionInitiationCache,
    #[allow(clippy::type_complexity)]
    session_notifiers: Arc<OnceLock<(T, Sender<(SessionId, ClosureReason)>)>>,
    sessions: moka::future::Cache<SessionId, SessionSlot>,
    msg_sender: Arc<OnceLock<S>>,
    cfg: SessionManagerConfig,
}

impl<S, T> Clone for SessionManager<S, T> {
    fn clone(&self) -> Self {
        Self {
            session_initiations: self.session_initiations.clone(),
            session_notifiers: self.session_notifiers.clone(),
            sessions: self.sessions.clone(),
            cfg: self.cfg.clone(),
            msg_sender: self.msg_sender.clone(),
        }
    }
}

const EXTERNAL_SEND_TIMEOUT: Duration = Duration::from_millis(200);

impl<S, T> SessionManager<S, T>
where
    S: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
    T: futures::Sink<IncomingSession> + Clone + Send + Sync + Unpin + 'static,
    S::Error: std::error::Error + Send + Sync + Clone + 'static,
    T::Error: std::error::Error + Send + Sync + Clone + 'static,
{
    /// Creates a new instance given the [`config`](SessionManagerConfig).
    pub fn new(mut cfg: SessionManagerConfig) -> Self {
        let min_session_tag_range_reservation = ReservedTag::range().end;
        debug_assert!(
            min_session_tag_range_reservation > HoprStartProtocol::START_PROTOCOL_MESSAGE_TAG.as_u64(),
            "invalid tag reservation range"
        );

        // Accommodate the lower bound if too low.
        if cfg.session_tag_range.start < min_session_tag_range_reservation {
            let diff = min_session_tag_range_reservation - cfg.session_tag_range.start;
            cfg.session_tag_range = min_session_tag_range_reservation..cfg.session_tag_range.end + diff;
        }
        cfg.maximum_sessions = cfg
            .maximum_sessions
            .clamp(1, (cfg.session_tag_range.end - cfg.session_tag_range.start) as usize);

        // Ensure the Frame MTU is at least the size of the Session segment MTU payload
        cfg.frame_mtu = cfg.frame_mtu.max(SESSION_MTU);
        cfg.max_frame_timeout = cfg.max_frame_timeout.max(MIN_FRAME_TIMEOUT);

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_ACTIVE_SESSIONS.set(0.0);

        let msg_sender = Arc::new(OnceLock::new());
        Self {
            msg_sender: msg_sender.clone(),
            session_initiations: moka::future::Cache::builder()
                .max_capacity(cfg.maximum_sessions as u64)
                .time_to_live(
                    2 * initiation_timeout_max_one_way(
                        cfg.initiation_timeout_base,
                        RoutingOptions::MAX_INTERMEDIATE_HOPS,
                    ),
                )
                .build(),
            sessions: moka::future::Cache::builder()
                .max_capacity(cfg.maximum_sessions as u64)
                .time_to_idle(cfg.idle_timeout)
                .eviction_listener(|session_id: Arc<SessionId>, entry, reason| match &reason {
                    moka::notification::RemovalCause::Expired | moka::notification::RemovalCause::Size => {
                        trace!(?session_id, ?reason, "session evicted from the cache");
                        close_session(*session_id.as_ref(), entry, ClosureReason::Eviction);
                    }
                    _ => {}
                })
                .build(),
            session_notifiers: Arc::new(OnceLock::new()),
            cfg,
        }
    }

    /// Starts the instance with the given `msg_sender` `Sink`
    /// and a channel `new_session_notifier` used to notify when a new incoming session is opened to us.
    ///
    /// This method must be called prior to any calls to [`SessionManager::new_session`] or
    /// [`SessionManager::dispatch_message`].
    pub fn start(&self, msg_sender: S, new_session_notifier: T) -> crate::errors::Result<Vec<AbortHandle>> {
        self.msg_sender
            .set(msg_sender)
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        let (session_close_tx, session_close_rx) = futures::channel::mpsc::channel(self.cfg.maximum_sessions + 10);
        self.session_notifiers
            .set((new_session_notifier, session_close_tx))
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        let myself = self.clone();
        let ah_closure_notifications = hopr_async_runtime::spawn_as_abortable!(session_close_rx.for_each_concurrent(
            None,
            move |(session_id, closure_reason)| {
                let myself = myself.clone();
                async move {
                    // These notifications come from the Sessions themselves once
                    // an empty read is encountered, which means the closure was done by the
                    // other party.
                    if let Some(session_data) = myself.sessions.remove(&session_id).await {
                        close_session(session_id, session_data, closure_reason);
                    } else {
                        // Do not treat this as an error
                        debug!(
                            ?session_id,
                            ?closure_reason,
                            "could not find session id to close, maybe the session is already closed"
                        );
                    }
                }
            },
        ));

        // This is necessary to evict expired entries from the caches if
        // no session-related operations happen at all.
        // This ensures the dangling expired sessions are properly closed
        // and their closure is timely notified to the other party.
        let myself = self.clone();
        let ah_session_expiration = hopr_async_runtime::spawn_as_abortable!(async move {
            let jitter = hopr_crypto_random::random_float_in_range(1.0..1.5);
            let timeout = 2 * initiation_timeout_max_one_way(
                myself.cfg.initiation_timeout_base,
                RoutingOptions::MAX_INTERMEDIATE_HOPS,
            )
            .min(myself.cfg.idle_timeout)
            .mul_f64(jitter)
                / 2;
            futures_time::stream::interval(timeout.into())
                .for_each(|_| {
                    trace!("executing session cache evictions");
                    futures::future::join(
                        myself.sessions.run_pending_tasks(),
                        myself.session_initiations.run_pending_tasks(),
                    )
                    .map(|_| ())
                })
                .await;
        });

        Ok(vec![ah_closure_notifications, ah_session_expiration])
    }

    /// Check if [`start`](SessionManager::start) has been called and the instance is running.
    pub fn is_started(&self) -> bool {
        self.session_notifiers.get().is_some()
    }

    async fn insert_session_slot(&self, session_id: SessionId, slot: SessionSlot) -> crate::errors::Result<()> {
        // We currently do not support loopback Sessions on ourselves.
        let abort_handles_clone = slot.abort_handles.clone();
        if let moka::ops::compute::CompResult::Inserted(_) = self
            .sessions
            .entry(session_id)
            .and_compute_with(|entry| {
                futures::future::ready(if entry.is_none() {
                    moka::ops::compute::Op::Put(slot)
                } else {
                    moka::ops::compute::Op::Nop
                })
            })
            .await
        {
            #[cfg(all(feature = "prometheus", not(test)))]
            {
                METRIC_NUM_INITIATED_SESSIONS.increment();
                METRIC_ACTIVE_SESSIONS.increment(1.0);
            }

            Ok(())
        } else {
            // Session already exists; it means it is most likely a loopback attempt
            error!(%session_id, "session already exists - loopback attempt");
            abort_handles_clone.into_iter().for_each(|ah| ah.abort());
            Err(SessionManagerError::Loopback.into())
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
        self.sessions.run_pending_tasks().await;
        if self.cfg.maximum_sessions <= self.sessions.entry_count() as usize {
            return Err(SessionManagerError::TooManySessions.into());
        }

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        let (tx_initiation_done, rx_initiation_done) = futures::channel::mpsc::unbounded();
        let (challenge, _) = insert_into_next_slot(
            &self.session_initiations,
            |ch| {
                if let Some(challenge) = ch {
                    ((challenge + 1) % hopr_crypto_random::MAX_RANDOM_INTEGER).max(MIN_CHALLENGE)
                } else {
                    hopr_crypto_random::random_integer(MIN_CHALLENGE, None)
                }
            },
            |_| tx_initiation_done,
        )
        .await
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
        msg_sender
            .send((
                forward_routing.clone(),
                ApplicationDataOut::with_no_packet_info(start_session_msg.try_into()?),
            ))
            .timeout(futures_time::time::Duration::from(EXTERNAL_SEND_TIMEOUT))
            .await
            .map_err(|_| {
                error!(challenge, %pseudonym, %destination, "timeout sending session request message");
                TransportSessionError::Timeout
            })?
            .map_err(|e| TransportSessionError::PacketSendingError(e.to_string()))?;

        // The timeout is given by the number of hops requested
        let initiation_timeout: futures_time::time::Duration = initiation_timeout_max_one_way(
            self.cfg.initiation_timeout_base,
            cfg.forward_path_options.count_hops() + cfg.return_path_options.count_hops() + 2,
        )
        .into();

        // Await session establishment response from the Exit node or timeout
        pin_mut!(rx_initiation_done);

        trace!(challenge, "awaiting session establishment");
        match rx_initiation_done.try_next().timeout(initiation_timeout).await {
            Ok(Ok(Some(est))) => {
                // Session has been established, construct it
                let session_id = est.session_id;
                debug!(challenge = est.orig_challenge, ?session_id, "started a new session");

                let (tx, rx) = futures::channel::mpsc::unbounded::<ApplicationDataIn>();
                let notifier = self
                    .session_notifiers
                    .get()
                    .map(|(_, notifier)| {
                        let mut notifier = notifier.clone();
                        Box::new(move |session_id: SessionId, reason: ClosureReason| {
                            let _ = notifier
                                .try_send((session_id, reason))
                                .inspect_err(|error| error!(%session_id, %error, "failed to notify session closure"));
                        })
                    })
                    .ok_or(SessionManagerError::NotStarted)?;

                #[cfg(feature = "stats")]
                let metrics = Arc::new(SessionStats::new(
                    session_id,
                    HoprSessionConfig {
                        capabilities: cfg.capabilities,
                        frame_mtu: self.cfg.frame_mtu,
                        frame_timeout: self.cfg.max_frame_timeout,
                    },
                ));

                // NOTE: the Exit node can have different `max_surb_buffer_size`
                // setting on the Session manager, so it does not make sense to cap it here
                // with our maximum value.
                if let Some(balancer_config) = cfg.surb_management {
                    let surb_estimator = AtomicSurbFlowEstimator::default();

                    #[cfg(feature = "stats")]
                    metrics.set_surb_estimator(surb_estimator.clone(), balancer_config.target_surb_buffer_size);

                    // Sender responsible for keep-alive and Session data will be counting produced SURBs
                    let surb_estimator_clone = surb_estimator.clone();
                    let full_surb_scoring_sender =
                        msg_sender.with(move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                            // Count how many SURBs we sent with each packet
                            surb_estimator_clone.produced.fetch_add(
                                data.estimate_surbs_with_msg() as u64,
                                std::sync::atomic::Ordering::Relaxed,
                            );
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

                    let mut abort_handles = Vec::new();

                    // Spawn the SURB-bearing keep alive stream
                    let (ka_controller, ka_abort_handle) =
                        utils::spawn_keep_alive_stream(session_id, full_surb_scoring_sender, forward_routing.clone());
                    abort_handles.push(ka_abort_handle);

                    #[cfg(feature = "stats")]
                    metrics.set_refill_in_flight(true);

                    // Spawn the SURB balancer, which will decide on the initial SURB rate.
                    debug!(%session_id, ?balancer_config ,"spawning entry SURB balancer");
                    let balancer = SurbBalancer::new(
                        session_id,
                        // The setpoint and output limit is immediately reconfigured by SurbBalancer
                        PidBalancerController::from_gains(PidControllerGains::from_env_or_default()),
                        surb_estimator.clone(),
                        ka_controller,
                        balancer_config,
                    );

                    let (level_stream, balancer_abort_handle) = balancer.start_control_loop(
                        self.cfg.balancer_sampling_interval,
                        SessionCacheBalancerFeedback(self.sessions.clone()),
                        None,
                    );
                    abort_handles.push(balancer_abort_handle);

                    // If the insertion fails prematurely, it will also kill all the abort handles
                    self.insert_session_slot(
                        session_id,
                        SessionSlot {
                            session_tx: Arc::new(tx),
                            routing_opts: forward_routing.clone(),
                            abort_handles,
                            surb_mgmt: Some(balancer_config),
                            surb_estimator: None, // Outgoing sessions do not have SURB estimator
                            #[cfg(feature = "stats")]
                            stats: metrics.clone(),
                        },
                    )
                    .await?;

                    #[cfg(feature = "stats")]
                    metrics.set_state(SessionLifecycleState::Active);

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
                                SessionManagerError::Other("surb balancer was cancelled prematurely".into()).into(),
                            );
                        }
                        Err(_) => {
                            warn!(%session_id, "session didn't reach target SURB buffer size in time");
                        }
                    }

                    HoprSession::new(
                        session_id,
                        forward_routing,
                        HoprSessionConfig {
                            capabilities: cfg.capabilities,
                            frame_mtu: self.cfg.frame_mtu,
                            frame_timeout: self.cfg.max_frame_timeout,
                        },
                        (
                            reduced_surb_scoring_sender,
                            rx.inspect(move |_| {
                                // Received packets = SURB consumption estimate
                                // The received packets always consume a single SURB.
                                surb_estimator
                                    .consumed
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            }),
                        ),
                        Some(notifier),
                        #[cfg(feature = "stats")]
                        metrics,
                    )
                } else {
                    warn!(%session_id, "session ready without SURB balancing");

                    self.insert_session_slot(
                        session_id,
                        SessionSlot {
                            session_tx: Arc::new(tx),
                            routing_opts: forward_routing.clone(),
                            abort_handles: vec![],
                            surb_mgmt: None,
                            surb_estimator: None,
                            #[cfg(feature = "stats")]
                            stats: metrics.clone(),
                        },
                    )
                    .await?;
                    #[cfg(feature = "stats")]
                    metrics.set_state(SessionLifecycleState::Active);

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

                    HoprSession::new(
                        session_id,
                        forward_routing,
                        HoprSessionConfig {
                            capabilities: cfg.capabilities,
                            frame_mtu: self.cfg.frame_mtu,
                            frame_timeout: self.cfg.max_frame_timeout,
                        },
                        (reduced_surb_sender, rx),
                        Some(notifier),
                        #[cfg(feature = "stats")]
                        metrics,
                    )
                }
            }
            Ok(Ok(None)) => Err(SessionManagerError::Other(
                "internal error: sender has been closed without completing the session establishment".into(),
            )
            .into()),
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

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_SESSION_ERRS.increment(&["timeout"]);

                Err(TransportSessionError::Timeout)
            }
        }
    }

    /// Sends a keep-alive packet with the given [`SessionId`].
    ///
    /// This currently "fires & forgets" and does not expect nor await any "pong" response.
    pub async fn ping_session(&self, id: &SessionId) -> crate::errors::Result<()> {
        if let Some(session_data) = self.sessions.get(id).await {
            trace!(session_id = ?id, "pinging manually session");
            Ok(self
                .msg_sender
                .get()
                .cloned()
                .ok_or(SessionManagerError::NotStarted)?
                .send((
                    session_data.routing_opts.clone(),
                    ApplicationDataOut::with_no_packet_info(HoprStartProtocol::KeepAlive((*id).into()).try_into()?),
                ))
                .timeout(futures_time::time::Duration::from(EXTERNAL_SEND_TIMEOUT))
                .await
                .map_err(|_| {
                    error!("timeout sending session ping message");
                    TransportSessionError::Timeout
                })?
                .map_err(|e| TransportSessionError::PacketSendingError(e.to_string()))?)
        } else {
            Err(SessionManagerError::NonExistingSession.into())
        }
    }

    /// Returns [`SessionIds`](SessionId) of all currently active sessions.
    pub async fn active_sessions(&self) -> Vec<SessionId> {
        self.sessions.run_pending_tasks().await;
        self.sessions.iter().map(|(k, _)| *k).collect()
    }

    /// Updates the configuration of the SURB balancer on the given [`SessionId`].
    ///
    /// Returns an error if the Session with the given `id` does not exist, or
    /// if it does not use SURB balancing.
    pub async fn update_surb_balancer_config(
        &self,
        id: &SessionId,
        config: SurbBalancerConfig,
    ) -> crate::errors::Result<()> {
        match self
            .sessions
            .entry_by_ref(id)
            .and_compute_with(|entry| {
                futures::future::ready(if let Some(mut cached_session) = entry.map(|e| e.into_value()) {
                    // Only update the config if there already was one before
                    if cached_session.surb_mgmt.is_some() {
                        cached_session.surb_mgmt = Some(config);
                        moka::ops::compute::Op::Put(cached_session)
                    } else {
                        moka::ops::compute::Op::Nop
                    }
                } else {
                    moka::ops::compute::Op::Nop
                })
            })
            .await
        {
            moka::ops::compute::CompResult::ReplacedWith(_) => Ok(()),
            moka::ops::compute::CompResult::Unchanged(_) => {
                Err(SessionManagerError::Other("session does not use SURB balancing".into()).into())
            }
            _ => Err(SessionManagerError::NonExistingSession.into()),
        }
    }

    /// Retrieves the configuration of SURB balancing for the given Session.
    ///
    /// Returns an error if the Session with the given `id` does not exist.
    pub async fn get_surb_balancer_config(&self, id: &SessionId) -> crate::errors::Result<Option<SurbBalancerConfig>> {
        match self.sessions.get(id).await {
            Some(session) => Ok(session.surb_mgmt),
            None => Err(SessionManagerError::NonExistingSession.into()),
        }
    }

    /// Retrieves useful statistics of the given [session](HoprSession).
    ///
    /// Returns an error if the Session with the given `id` does not exist.
    #[cfg(feature = "stats")]
    pub async fn get_session_stats(&self, id: &SessionId) -> crate::errors::Result<SessionStatsSnapshot> {
        match self.sessions.get(id).await {
            Some(session) => Ok(session.stats.snapshot()),
            None => Err(SessionManagerError::NonExistingSession.into()),
        }
    }

    /// The main method to be called whenever data are received.
    ///
    /// It tries to recognize the message and correctly dispatches either
    /// the Session protocol or Start protocol messages.
    ///
    /// If the data are not recognized, they are returned as [`DispatchResult::Unrelated`].
    pub async fn dispatch_message(
        &self,
        pseudonym: HoprPseudonym,
        in_data: ApplicationDataIn,
    ) -> crate::errors::Result<DispatchResult> {
        if in_data.data.application_tag == HoprStartProtocol::START_PROTOCOL_MESSAGE_TAG {
            // This is a Start protocol message, so we handle it
            trace!("dispatching Start protocol message");
            return self
                .handle_start_protocol_message(pseudonym, in_data)
                .await
                .map(|_| DispatchResult::Processed);
        } else if self
            .cfg
            .session_tag_range
            .contains(&in_data.data.application_tag.as_u64())
        {
            let session_id = SessionId::new(in_data.data.application_tag, pseudonym);

            return if let Some(session_data) = self.sessions.get(&session_id).await {
                trace!(?session_id, "received data for a registered session");

                Ok(session_data
                    .session_tx
                    .unbounded_send(in_data)
                    .map(|_| DispatchResult::Processed)
                    .map_err(|e| SessionManagerError::Other(e.to_string()))?)
            } else {
                error!(%session_id, "received data from an unestablished session");
                Err(TransportSessionError::UnknownData)
            };
        }

        trace!(tag = %in_data.data.application_tag, "received data not associated with session protocol or any existing session");
        Ok(DispatchResult::Unrelated(in_data))
    }

    async fn handle_incoming_session_initiation(
        &self,
        pseudonym: HoprPseudonym,
        session_req: StartInitiation<SessionTarget, ByteCapabilities>,
    ) -> crate::errors::Result<()> {
        trace!(challenge = session_req.challenge, "received session initiation request");

        debug!(%pseudonym, "got new session request, searching for a free session slot");

        let mut msg_sender = self.msg_sender.get().cloned().ok_or(SessionManagerError::NotStarted)?;

        let (mut new_session_notifier, mut close_session_notifier) = self
            .session_notifiers
            .get()
            .cloned()
            .ok_or(SessionManagerError::NotStarted)?;

        // Reply routing uses SURBs only with the pseudonym of this Session's ID
        let reply_routing = DestinationRouting::Return(pseudonym.into());

        let (tx_session_data, rx_session_data) = futures::channel::mpsc::unbounded::<ApplicationDataIn>();

        // Search for a free Session ID slot
        self.sessions.run_pending_tasks().await; // Needed so that entry_count is updated
        let allocated_slot = if self.cfg.maximum_sessions > self.sessions.entry_count() as usize {
            insert_into_next_slot(
                &self.sessions,
                |sid| {
                    // NOTE: It is allowed to insert sessions using the same tag
                    // but different pseudonyms because the SessionId is different.
                    let next_tag: Tag = match sid {
                        Some(session_id) => ((session_id.tag().as_u64() + 1) % self.cfg.session_tag_range.end)
                            .max(self.cfg.session_tag_range.start)
                            .into(),
                        None => hopr_crypto_random::random_integer(
                            self.cfg.session_tag_range.start,
                            Some(self.cfg.session_tag_range.end),
                        )
                        .into(),
                    };
                    SessionId::new(next_tag, pseudonym)
                },
                |_sid| SessionSlot {
                    session_tx: Arc::new(tx_session_data),
                    routing_opts: reply_routing.clone(),
                    abort_handles: vec![],
                    surb_mgmt: None,
                    surb_estimator: None,
                    #[cfg(feature = "stats")]
                    stats: SessionStats::new(
                        _sid,
                        HoprSessionConfig {
                            capabilities: session_req.capabilities.0,
                            frame_mtu: self.cfg.frame_mtu,
                            frame_timeout: self.cfg.max_frame_timeout,
                        },
                    )
                    .into(),
                },
            )
            .await
        } else {
            error!(%pseudonym, "cannot accept incoming session, the maximum number of sessions has been reached");
            None
        };

        // If some of the following code throws an error, the allocated slot will be kept
        // but will be later re-claimed when it expires.
        if let Some((session_id, _slot)) = allocated_slot {
            debug!(%session_id, ?session_req, "assigned a new session");

            #[cfg(feature = "stats")]
            let stats = _slot.stats;

            let closure_notifier = Box::new(move |session_id: SessionId, reason: ClosureReason| {
                if let Err(error) = close_session_notifier.try_send((session_id, reason)) {
                    error!(%session_id, %error, %reason, "failed to notify session closure");
                }
            });

            let session = if !session_req.capabilities.0.contains(Capability::NoRateControl) {
                let surb_estimator = AtomicSurbFlowEstimator::default();

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
                #[cfg(feature = "stats")]
                stats.set_surb_estimator(surb_estimator.clone(), target_surb_buffer_size);

                let surb_estimator_clone = surb_estimator.clone();
                let session = HoprSession::new(
                    session_id,
                    reply_routing.clone(),
                    HoprSessionConfig {
                        capabilities: session_req.capabilities.into(),
                        frame_mtu: self.cfg.frame_mtu,
                        frame_timeout: self.cfg.max_frame_timeout,
                    },
                    (
                        // Sent packets = SURB consumption estimate
                        msg_sender
                            .clone()
                            .with(move |(routing, data): (DestinationRouting, ApplicationDataOut)| {
                                // Each outgoing packet consumes one SURB
                                surb_estimator_clone
                                    .consumed
                                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                futures::future::ok::<_, S::Error>((routing, data))
                            })
                            .rate_limit_with_controller(&egress_rate_control)
                            .buffer((2 * target_surb_buffer_size) as usize),
                        // Received packets = SURB retrieval estimate
                        rx_session_data.inspect(move |data| {
                            // Count the number of SURBs delivered with each incoming packet
                            surb_estimator_clone
                                .produced
                                .fetch_add(data.num_surbs_with_msg() as u64, std::sync::atomic::Ordering::Relaxed);
                        }),
                    ),
                    Some(closure_notifier),
                    #[cfg(feature = "stats")]
                    stats.clone(),
                )?;

                // The SURB balancer will start intervening by rate-limiting the
                // egress of the Session, once the estimated number of SURBs drops below
                // the target defined here. Otherwise, the maximum egress is allowed.
                let balancer_config = SurbBalancerConfig {
                    target_surb_buffer_size,
                    // At maximum egress, the SURB buffer drains in `minimum_surb_buffer_duration` seconds
                    max_surbs_per_sec: target_surb_buffer_size
                        / self
                            .cfg
                            .minimum_surb_buffer_duration
                            .max(MIN_SURB_BUFFER_DURATION)
                            .as_secs(),
                    // No SURB decay at the Exit, since we know almost exactly how many SURBs
                    // were received
                    surb_decay: None,
                };

                // Assign the SURB balancer and abort handles to the already allocated Session slot
                let (balancer_abort_handle, balancer_abort_reg) = AbortHandle::new_pair();
                if let moka::ops::compute::CompResult::ReplacedWith(_) = self
                    .sessions
                    .entry(session_id)
                    .and_compute_with(|entry| {
                        if let Some(mut cached_session) = entry.map(|c| c.into_value()) {
                            cached_session.abort_handles.push(balancer_abort_handle);
                            cached_session.surb_mgmt = Some(balancer_config);
                            cached_session.surb_estimator = Some(surb_estimator.clone());
                            #[cfg(feature = "stats")]
                            {
                                cached_session.stats = stats.clone();
                            }
                            futures::future::ready(moka::ops::compute::Op::Put(cached_session))
                        } else {
                            futures::future::ready(moka::ops::compute::Op::Nop)
                        }
                    })
                    .await
                {
                    // Spawn the SURB balancer only once we know we have registered the
                    // abort handle with the pre-allocated Session slot
                    debug!(%session_id, ?balancer_config ,"spawning exit SURB balancer");
                    let balancer = SurbBalancer::new(
                        session_id,
                        if let Some((growth_window, ratio_threshold)) = self.cfg.growable_target_surb_buffer.as_ref() {
                            // Allow automatic setpoint increase
                            SimpleBalancerController::with_increasing_setpoint(
                                *ratio_threshold,
                                (growth_window
                                    .div_duration_f64(self.cfg.balancer_sampling_interval)
                                    .round() as usize)
                                    .max(1),
                            )
                        } else {
                            SimpleBalancerController::default()
                        },
                        surb_estimator,
                        SurbControllerWithCorrection(egress_rate_control, 1), // 1 SURB per egress packet
                        balancer_config,
                    );

                    let _ = balancer.start_control_loop(
                        self.cfg.balancer_sampling_interval,
                        SessionCacheBalancerFeedback(self.sessions.clone()),
                        Some(balancer_abort_reg),
                    );
                } else {
                    // This should never happen, but be sure we handle this error
                    return Err(SessionManagerError::Other(
                        "failed to spawn SURB balancer - inconsistent cache".into(),
                    )
                    .into());
                }

                session
            } else {
                HoprSession::new(
                    session_id,
                    reply_routing.clone(),
                    HoprSessionConfig {
                        capabilities: session_req.capabilities.into(),
                        frame_mtu: self.cfg.frame_mtu,
                        frame_timeout: self.cfg.max_frame_timeout,
                    },
                    (msg_sender.clone(), rx_session_data),
                    Some(closure_notifier),
                    #[cfg(feature = "stats")]
                    stats,
                )?
            };

            // Extract useful information about the session from the Start protocol message
            let incoming_session = IncomingSession {
                session,
                target: session_req.target,
            };

            // Notify that a new incoming session has been created
            match new_session_notifier
                .send(incoming_session)
                .timeout(futures_time::time::Duration::from(EXTERNAL_SEND_TIMEOUT))
                .await
            {
                Err(_) => {
                    error!(%session_id, "timeout to notify about new incoming session");
                    return Err(TransportSessionError::Timeout);
                }
                Ok(Err(error)) => {
                    error!(%session_id, %error, "failed to notify about new incoming session");
                    return Err(SessionManagerError::Other(error.to_string()).into());
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

            msg_sender
                .send((reply_routing, ApplicationDataOut::with_no_packet_info(data.try_into()?)))
                .timeout(futures_time::time::Duration::from(EXTERNAL_SEND_TIMEOUT))
                .await
                .map_err(|_| {
                    error!(%session_id, "timeout sending session establishment message");
                    TransportSessionError::Timeout
                })?
                .map_err(|e| {
                    SessionManagerError::Other(format!(
                        "failed to send session {session_id} establishment message: {e}"
                    ))
                })?;

            info!(%session_id, "new session established");

            #[cfg(all(feature = "prometheus", not(test)))]
            {
                METRIC_NUM_ESTABLISHED_SESSIONS.increment();
                METRIC_ACTIVE_SESSIONS.increment(1.0);
            }
        } else {
            error!(%pseudonym,"failed to reserve a new session slot");

            // Notify the sender that the session could not be established
            let reason = StartErrorReason::NoSlotsAvailable;
            let data = HoprStartProtocol::SessionError(StartErrorType {
                challenge: session_req.challenge,
                reason,
            });

            msg_sender
                .send((reply_routing, ApplicationDataOut::with_no_packet_info(data.try_into()?)))
                .timeout(futures_time::time::Duration::from(EXTERNAL_SEND_TIMEOUT))
                .await
                .map_err(|_| {
                    error!("timeout sending session error message");
                    TransportSessionError::Timeout
                })?
                .map_err(|e| {
                    SessionManagerError::Other(format!("failed to send session establishment error message: {e}"))
                })?;

            trace!(%pseudonym, "session establishment failure message sent");

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_SENT_SESSION_ERRS.increment(&[&reason.to_string()])
        }

        Ok(())
    }

    async fn handle_start_protocol_message(
        &self,
        pseudonym: HoprPseudonym,
        data: ApplicationDataIn,
    ) -> crate::errors::Result<()> {
        match HoprStartProtocol::try_from(data.data)? {
            HoprStartProtocol::StartSession(session_req) => {
                self.handle_incoming_session_initiation(pseudonym, session_req).await?;
            }
            HoprStartProtocol::SessionEstablished(est) => {
                trace!(
                    session_id = ?est.session_id,
                    "received session establishment confirmation"
                );
                let challenge = est.orig_challenge;
                let session_id = est.session_id;
                if let Some(tx_est) = self.session_initiations.remove(&est.orig_challenge).await {
                    if let Err(e) = tx_est.unbounded_send(Ok(est)) {
                        return Err(SessionManagerError::Other(format!(
                            "could not notify session {session_id} establishment: {e}"
                        ))
                        .into());
                    }
                    debug!(?session_id, challenge, "session establishment complete");
                } else {
                    error!(%session_id, challenge, "unknown session establishment attempt or expired");
                }
            }
            HoprStartProtocol::SessionError(error) => {
                trace!(
                    challenge = error.challenge,
                    error = ?error.reason,
                    "failed to initialize a session",
                );
                // Currently, we do not distinguish between individual error types
                // and just discard the initiation attempt and pass on the error.
                if let Some(tx_est) = self.session_initiations.remove(&error.challenge).await {
                    if let Err(e) = tx_est.unbounded_send(Err(error)) {
                        return Err(SessionManagerError::Other(format!(
                            "could not notify session establishment error {error:?}: {e}"
                        ))
                        .into());
                    }
                    error!(
                        challenge = error.challenge,
                        ?error,
                        "session establishment error received"
                    );
                } else {
                    error!(
                        challenge = error.challenge,
                        ?error,
                        "session establishment attempt expired before error could be delivered"
                    );
                }

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_RECEIVED_SESSION_ERRS.increment(&[&error.reason.to_string()])
            }
            HoprStartProtocol::KeepAlive(msg) => {
                let session_id = msg.session_id;
                if let Some(session_slot) = self.sessions.get(&session_id).await {
                    trace!(?session_id, "received keep-alive request");
                    // If the session has a SURB flow estimator, increase the number of received SURBs.
                    // This applies to the incoming sessions currently
                    if let Some(estimator) = session_slot.surb_estimator.as_ref() {
                        // Two SURBs per Keep-Alive message
                        estimator.produced.fetch_add(
                            KeepAliveMessage::<SessionId>::MIN_SURBS_PER_MESSAGE as u64,
                            std::sync::atomic::Ordering::Relaxed,
                        );
                    }
                } else {
                    debug!(%session_id, "received keep-alive request for an unknown session");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use futures::{AsyncWriteExt, future::BoxFuture};
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use hopr_primitive_types::prelude::Address;
    use hopr_protocol_start::StartProtocolDiscriminants;
    use tokio::time::timeout;

    use super::*;
    use crate::{Capabilities, Capability, balancer::SurbBalancerConfig, types::SessionTarget};

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

    fn mock_packet_planning(sender: MockMsgSender) -> UnboundedSender<(DestinationRouting, ApplicationDataOut)> {
        let (tx, rx) = futures::channel::mpsc::unbounded();
        tokio::task::spawn(async move {
            pin_mut!(rx);
            while let Some((routing, data)) = rx.next().await {
                sender
                    .send_message(routing, data)
                    .await
                    .expect("send message must not fail in mock");
            }
        });
        tx
    }

    fn msg_type(data: &ApplicationDataOut, expected: StartProtocolDiscriminants) -> bool {
        HoprStartProtocol::decode(data.data.application_tag, &data.data.plain_text)
            .map(|d| StartProtocolDiscriminants::from(d) == expected)
            .unwrap_or(false)
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
                        .await?;
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
                    alice_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        .await?;
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
                        .await?;
                    Ok(())
                })
            });

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        ahs.extend(alice_mgr.start(mock_packet_planning(alice_transport), new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        ahs.extend(bob_mgr.start(mock_packet_planning(bob_transport), new_session_tx_bob)?);

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

        assert_eq!(vec![*alice_session.id()], alice_mgr.active_sessions().await);
        assert_eq!(None, alice_mgr.get_surb_balancer_config(alice_session.id()).await?);
        assert!(
            alice_mgr
                .update_surb_balancer_config(alice_session.id(), SurbBalancerConfig::default())
                .await
                .is_err()
        );

        assert_eq!(vec![*bob_session.session.id()], bob_mgr.active_sessions().await);
        assert_eq!(None, bob_mgr.get_surb_balancer_config(bob_session.session.id()).await?);
        assert!(
            bob_mgr
                .update_surb_balancer_config(bob_session.session.id(), SurbBalancerConfig::default())
                .await
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
                        .await?;
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
                    alice_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        .await?;
                    Ok(())
                })
            });

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        ahs.extend(alice_mgr.start(mock_packet_planning(alice_transport), new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        ahs.extend(bob_mgr.start(mock_packet_planning(bob_transport), new_session_tx_bob)?);

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

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn session_manager_should_update_surb_balancer_config() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let session_id = SessionId::new(16u64, alice_pseudonym);
        let balancer_cfg = SurbBalancerConfig {
            target_surb_buffer_size: 1000,
            max_surbs_per_sec: 100,
            ..Default::default()
        };

        let alice_mgr = SessionManager::<
            UnboundedSender<(DestinationRouting, ApplicationDataOut)>,
            futures::channel::mpsc::Sender<IncomingSession>,
        >::new(Default::default());

        let (dummy_tx, _) = futures::channel::mpsc::unbounded();
        alice_mgr
            .sessions
            .insert(
                session_id,
                SessionSlot {
                    session_tx: Arc::new(dummy_tx),
                    routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(alice_pseudonym)),
                    abort_handles: Vec::new(),
                    surb_mgmt: Some(balancer_cfg.clone()),
                    surb_estimator: None,
                    #[cfg(feature = "stats")]
                    stats: Arc::new(SessionStats::new(session_id, Default::default())),
                },
            )
            .await;

        let actual_cfg = alice_mgr
            .get_surb_balancer_config(&session_id)
            .await?
            .ok_or(anyhow!("session must have a surb balancer config"))?;
        assert_eq!(actual_cfg, balancer_cfg);

        let new_cfg = SurbBalancerConfig {
            target_surb_buffer_size: 2000,
            max_surbs_per_sec: 200,
            ..Default::default()
        };
        alice_mgr.update_surb_balancer_config(&session_id, new_cfg).await?;

        let actual_cfg = alice_mgr
            .get_surb_balancer_config(&session_id)
            .await?
            .ok_or(anyhow!("session must have a surb balancer config"))?;
        assert_eq!(actual_cfg, new_cfg);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn session_manager_should_not_allow_establish_session_when_tag_range_is_used_up() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let cfg = SessionManagerConfig {
            session_tag_range: 16u64..17u64, // Slot for exactly one session
            ..Default::default()
        };

        let alice_mgr = SessionManager::new(Default::default());
        let bob_mgr = SessionManager::new(cfg);

        // Occupy the only free slot with tag 16
        let (dummy_tx, _) = futures::channel::mpsc::unbounded();
        bob_mgr
            .sessions
            .insert(
                SessionId::new(16u64, alice_pseudonym),
                SessionSlot {
                    session_tx: Arc::new(dummy_tx),
                    routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(alice_pseudonym)),
                    abort_handles: Vec::new(),
                    stats: Arc::new(SessionStats::new(
                        SessionId::new(16u64, alice_pseudonym),
                        Default::default(),
                    )),
                    surb_mgmt: None,
                    surb_estimator: None,
                },
            )
            .await;

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
                        .await?;
                    Ok(())
                })
            });

        // Bob sends the SessionError message
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::SessionError)
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
                        .await?;
                    Ok(())
                })
            });

        let mut jhs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        jhs.extend(alice_mgr.start(mock_packet_planning(alice_transport), new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
        jhs.extend(bob_mgr.start(mock_packet_planning(bob_transport), new_session_tx_bob)?);

        let result = alice_mgr
            .new_session(
                bob_peer,
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: Capabilities::empty(),
                    pseudonym: alice_pseudonym.into(),
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await;

        assert!(
            matches!(result, Err(TransportSessionError::Rejected(reason)) if reason == StartErrorReason::NoSlotsAvailable)
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn session_manager_should_not_allow_establish_session_when_maximum_number_of_session_is_reached()
    -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let cfg = SessionManagerConfig {
            maximum_sessions: 1,
            ..Default::default()
        };

        let alice_mgr = SessionManager::new(Default::default());
        let bob_mgr = SessionManager::new(cfg);

        // Occupy the only free slot with tag 16
        let (dummy_tx, _) = futures::channel::mpsc::unbounded();
        bob_mgr
            .sessions
            .insert(
                SessionId::new(16u64, alice_pseudonym),
                SessionSlot {
                    session_tx: Arc::new(dummy_tx),
                    routing_opts: DestinationRouting::Return(alice_pseudonym.into()),
                    abort_handles: Vec::new(),
                    surb_mgmt: None,
                    surb_estimator: None,
                    #[cfg(feature = "stats")]
                    stats: Arc::new(SessionStats::new(
                        SessionId::new(16u64, alice_pseudonym),
                        Default::default(),
                    )),
                },
            )
            .await;

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
                        .await?;
                    Ok(())
                })
            });

        // Bob sends the SessionError message
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::SessionError)
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
                        .await?;
                    Ok(())
                })
            });

        let mut jhs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        jhs.extend(alice_mgr.start(mock_packet_planning(alice_transport), new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
        jhs.extend(bob_mgr.start(mock_packet_planning(bob_transport), new_session_tx_bob)?);

        let result = alice_mgr
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

        assert!(
            matches!(result, Err(TransportSessionError::Rejected(reason)) if reason == StartErrorReason::NoSlotsAvailable)
        );

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
                        .await?;
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
                    alice_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        .await?;
                    Ok(())
                })
            });

        // Start Alice
        let (new_session_tx_alice, new_session_rx_alice) = futures::channel::mpsc::channel(1024);
        alice_mgr.start(mock_packet_planning(alice_transport), new_session_tx_alice)?;

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
        alice_mgr.start(mock_packet_planning(alice_transport), new_session_tx_alice)?;

        // Start Bob
        let (new_session_tx_bob, _) = futures::channel::mpsc::channel(1024);
        bob_mgr.start(mock_packet_planning(bob_transport), new_session_tx_bob)?;

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

    #[test_log::test(tokio::test)]
    async fn session_manager_should_send_keep_alives_via_surb_balancer() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let bob_cfg = SessionManagerConfig::default();
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
                        .await?;
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
                    alice_mgr_clone
                        .dispatch_message(
                            alice_pseudonym,
                            ApplicationDataIn {
                                data: data.data,
                                packet_info: Default::default(),
                            },
                        )
                        .await?;
                    Ok(())
                })
            });

        // Alice sends the KeepAlive messages
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .times(5..)
            //.in_sequence(&mut sequence)
            .withf(move |peer, data| {
                msg_type(data, StartProtocolDiscriminants::KeepAlive)
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
                        .await?;
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
                        .await?;
                    Ok(())
                })
            });

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::channel(1024);
        ahs.extend(alice_mgr.start(mock_packet_planning(alice_transport), new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::channel(1024);
        ahs.extend(bob_mgr.start(mock_packet_planning(bob_transport), new_session_tx_bob)?);

        let target = SealedHost::Plain("127.0.0.1:80".parse()?);

        let balancer_cfg = SurbBalancerConfig {
            target_surb_buffer_size: 10,
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
            alice_mgr.get_surb_balancer_config(alice_session.id()).await?
        );

        let remote_cfg = bob_mgr
            .get_surb_balancer_config(bob_session.session.id())
            .await?
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
        alice_session.close().await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(matches!(
            alice_mgr.ping_session(alice_session.id()).await,
            Err(TransportSessionError::Manager(SessionManagerError::NonExistingSession))
        ));

        futures::stream::iter(ahs)
            .for_each(|ah| async move { ah.abort() })
            .await;

        Ok(())
    }
}
