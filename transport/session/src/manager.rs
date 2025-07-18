use std::{
    ops::Range,
    sync::{Arc, OnceLock, atomic::AtomicU64},
    time::Duration,
};

use futures::{
    FutureExt, StreamExt, TryStreamExt,
    channel::mpsc::UnboundedSender,
    future::{AbortHandle, Either},
    pin_mut,
};
use hopr_crypto_packet::prelude::HoprPacket;
use hopr_crypto_random::Randomizable;
use hopr_internal_types::prelude::HoprPseudonym;
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::Address;
use hopr_transport_packet::prelude::{ApplicationData, ReservedTag, Tag};
use tracing::{debug, error, info, trace, warn};

use crate::{
    IncomingSession, Session, SessionClientConfig, SessionId, SessionTarget,
    balancer::{RateController, RateLimitExt, SurbBalancer, SurbFlowController},
    errors::{SessionManagerError, TransportSessionError},
    initiation::{StartChallenge, StartErrorReason, StartErrorType, StartEstablished, StartInitiation, StartProtocol},
    traits::SendMsg,
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

/// Configuration for the [`SessionManager`].
#[derive(Clone, Debug, PartialEq, Eq, smart_default::SmartDefault)]
pub struct SessionManagerConfig {
    /// Ranges of tags available for Sessions.
    ///
    /// **NOTE**: If the range starts lower than [`MIN_SESSION_TAG_RANGE_RESERVATION`],
    /// it will be automatically transformed to start at this value.
    /// This is due to the reserved range by the Start sub-protocol.
    ///
    /// Default is 16..1024.
    #[doc(hidden)]
    #[default(_code = "16u64..1024u64")]
    pub session_tag_range: Range<u64>,

    /// The base timeout for initiation of Session initiation.
    ///
    /// The actual timeout is adjusted according to the number of hops for that Session:
    /// `t = initiation_time_out_base * (num_forward_hops + num_return_hops + 2)`
    ///
    /// Default is 5 seconds.
    #[default(Duration::from_secs(5))]
    pub initiation_timeout_base: Duration,

    /// Timeout for Session to be closed due to inactivity.
    ///
    /// Default is 180 seconds.
    #[default(Duration::from_secs(180))]
    pub idle_timeout: Duration,

    /// The sampling interval for SURB balancer.
    /// It will make SURB control decisions regularly at this interval.
    ///
    /// Default is 500 milliseconds.
    #[default(Duration::from_millis(500))]
    pub balancer_sampling_interval: Duration,
}

fn close_session_after_eviction<S: SendMsg + Send + Sync + 'static>(
    msg_sender: Arc<OnceLock<S>>,
    session_id: SessionId,
    session_data: CachedSession,
    cause: moka::notification::RemovalCause,
) -> moka::notification::ListenerFuture {
    // When a Session is removed from the cache, we notify the other party only
    // if this removal was due to expiration or cache size limit.
    match cause {
        r @ moka::notification::RemovalCause::Expired | r @ moka::notification::RemovalCause::Size
            if msg_sender.get().is_some() =>
        {
            trace!(
                ?session_id,
                reason = ?r,
                "session termination due to eviction from the cache"
            );
            let data = match ApplicationData::try_from(StartProtocol::CloseSession(session_id)) {
                Ok(data) => data,
                Err(error) => {
                    error!(%session_id, %error, "failed to serialize CloseSession");
                    return futures::future::ready(()).boxed();
                }
            };

            let msg_sender = msg_sender.clone();
            async move {
                // Unwrap cannot panic, since the value's existence is checked on L72
                if let Err(error) = msg_sender
                    .get()
                    .unwrap()
                    .send_message(data, session_data.routing_opts)
                    .await
                {
                    error!(
                        %session_id,
                        %error,
                        "could not send notification of session closure after cache eviction"
                    );
                }

                session_data.session_tx.close_channel();
                debug!(
                    ?session_id,
                    reason = ?r,
                    "session has been closed due to cache eviction"
                );

                // Terminate any additional tasks spawned by the Session
                session_data.abort_handles.into_iter().for_each(|h| h.abort());

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_ACTIVE_SESSIONS.decrement(1.0);
            }
            .boxed()
        }
        _ => futures::future::ready(()).boxed(),
    }
}

/// This function will use the given generator to generate an initial seeding key.
/// It will check whether the given cache already contains a value for that key, and if not,
/// calls the generator (with the previous value) to generate a new seeding key and retry.
/// The function either finds a suitable free slot, inserting the `value` and returns the found key,
/// or terminates with `None` when `gen` returns the initial seed again.
async fn insert_into_next_slot<K, V, F>(cache: &moka::future::Cache<K, V>, generator: F, value: V) -> Option<K>
where
    K: Copy + std::hash::Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(Option<K>) -> K,
{
    let initial = generator(None);
    let mut next = initial;
    loop {
        let insertion_result = cache
            .entry(next)
            .and_try_compute_with(|e| {
                if e.is_none() {
                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Put(value.clone()))
                } else {
                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Nop)
                }
            })
            .await;

        // If we inserted successfully, break the loop and return the insertion key
        if let Ok(moka::ops::compute::CompResult::Inserted(_)) = insertion_result {
            return Some(next);
        }

        // Otherwise, generate the next key
        next = generator(Some(next));

        // If generated keys made it to full loop, return failure
        if next == initial {
            return None;
        }
    }
}

/// The first challenge value used in Start protocol to initiate a session.
pub(crate) const MIN_CHALLENGE: StartChallenge = 1;

/// Maximum time to wait for counterparty to receive the target amount of SURBs.
const SESSION_READINESS_TIMEOUT: Duration = Duration::from_secs(10);

// Needs to use an UnboundedSender instead of oneshot
// because Moka cache requires the value to be Clone, which oneshot Sender is not.
// It also cannot be enclosed in an Arc, since calling `send` consumes the oneshot Sender.
type SessionInitiationCache =
    moka::future::Cache<StartChallenge, UnboundedSender<Result<StartEstablished<SessionId>, StartErrorType>>>;

#[derive(Clone)]
struct CachedSession {
    // Sender needs to be put in Arc, so that no clones are made by `moka`.
    // This makes sure that the entire channel closes once the one and only sender is closed.
    session_tx: Arc<UnboundedSender<Box<[u8]>>>,
    routing_opts: DestinationRouting,
    abort_handles: Vec<AbortHandle>,
}

/// Indicates the result of processing a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchResult {
    /// Session or Start protocol message has been processed successfully.
    Processed,
    /// The message was not related to Start or Session protocol.
    Unrelated(ApplicationData),
}

pub(crate) struct KeepAliveController(pub(crate) RateController);

impl SurbFlowController for KeepAliveController {
    fn adjust_surb_flow(&self, surbs_per_sec: usize) {
        // Currently, a keep-alive message can bear `HoprPacket::MAX_SURBS_IN_PACKET` SURBs,
        // so the correction by this factor is applied.
        self.0.set_rate_per_unit(
            surbs_per_sec,
            HoprPacket::MAX_SURBS_IN_PACKET as u32 * Duration::from_secs(1),
        );
    }
}

pub(crate) struct CountingSendMsg<T>(T, Arc<AtomicU64>);

impl<T: SendMsg> CountingSendMsg<T> {
    pub fn new(msg: T, counter: Arc<AtomicU64>) -> Self {
        Self(msg, counter)
    }
}

#[async_trait::async_trait]
impl<T: SendMsg + Send + Sync> SendMsg for CountingSendMsg<T> {
    async fn send_message(
        &self,
        data: ApplicationData,
        destination: DestinationRouting,
    ) -> Result<(), TransportSessionError> {
        let num_surbs = HoprPacket::max_surbs_with_message(data.len()) as u64;
        self.0.send_message(data, destination).await.inspect(|_| {
            self.1.fetch_add(num_surbs, std::sync::atomic::Ordering::Relaxed);
        })
    }
}

/// Manages lifecycles of Sessions.
///
/// Once the manager is [started](SessionManager::start), the [`SessionManager::dispatch_message`]
/// should be called for each [`ApplicationData`] received by the node.
/// This way, the `SessionManager` takes care of proper Start sub-protocol message processing
/// and correct dispatch of Session-related packets to individual existing Sessions.
///
/// Secondly, the manager can initiate new outgoing sessions via [`SessionManager::new_session`].
///
/// Since the `SessionManager` operates over the HOPR protocol,
/// the [message transport `S`](SendMsg) is required.
/// Such transport must also be `Clone`, since it will be cloned into the created [`Session`] objects.
///
/// The manager also can take care of [SURB balancing](SurbBalancerConfig) per Session. When enabled,
/// a desired target level of SURBs at the Session counterparty is set. According to measured
/// inflow and outflow of SURBS to/from the counterparty, the production of non-organic SURBs
/// through keep-alive messages (sent to counterparty) is controlled to maintain that target level.
pub struct SessionManager<S> {
    session_initiations: SessionInitiationCache,
    session_notifiers: Arc<OnceLock<(UnboundedSender<IncomingSession>, UnboundedSender<SessionId>)>>,
    sessions: moka::future::Cache<SessionId, CachedSession>,
    msg_sender: Arc<OnceLock<S>>,
    cfg: SessionManagerConfig,
}

impl<S> Clone for SessionManager<S> {
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

fn initiation_timeout_max_one_way(base: Duration, hops: usize) -> Duration {
    base * (hops as u32)
}

/// Smallest possible interval for balancer sampling.
pub const MIN_BALANCER_SAMPLING_INTERVAL: Duration = Duration::from_millis(100);

impl<S: SendMsg + Clone + Send + Sync + 'static> SessionManager<S> {
    /// Creates a new instance given the `PeerId` and [config](SessionManagerConfig).
    pub fn new(mut cfg: SessionManagerConfig) -> Self {
        let min_session_tag_range_reservation = ReservedTag::range().end;
        debug_assert!(
            min_session_tag_range_reservation > StartProtocol::<SessionId>::START_PROTOCOL_MESSAGE_TAG.as_u64(),
            "invalid tag reservation range"
        );

        // Accommodate the lower bound if too low.
        if cfg.session_tag_range.start < min_session_tag_range_reservation {
            let diff = min_session_tag_range_reservation - cfg.session_tag_range.start;
            cfg.session_tag_range = min_session_tag_range_reservation..cfg.session_tag_range.end + diff;
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_ACTIVE_SESSIONS.set(0.0);

        let msg_sender = Arc::new(OnceLock::new());
        Self {
            msg_sender: msg_sender.clone(),
            session_initiations: moka::future::Cache::builder()
                .max_capacity(
                    std::ops::Range {
                        start: cfg.session_tag_range.start,
                        end: cfg.session_tag_range.end,
                    }
                    .count() as u64,
                )
                .time_to_live(
                    2 * initiation_timeout_max_one_way(
                        cfg.initiation_timeout_base,
                        RoutingOptions::MAX_INTERMEDIATE_HOPS,
                    ),
                )
                .build(),
            sessions: moka::future::Cache::builder()
                .max_capacity(u16::MAX as u64)
                .time_to_idle(cfg.idle_timeout)
                .async_eviction_listener(move |k, v, c| {
                    let msg_sender = msg_sender.clone();
                    close_session_after_eviction(msg_sender, *k, v, c)
                })
                .build(),
            session_notifiers: Arc::new(OnceLock::new()),
            cfg,
        }
    }

    /// Starts the instance with the given [transport](SendMsg) implementation
    /// and a channel that is used to notify when a new incoming session is opened to us.
    ///
    /// This method must be called prior to any calls to [`SessionManager::new_session`] or
    /// [`SessionManager::dispatch_message`].
    pub fn start(
        &self,
        msg_sender: S,
        new_session_notifier: UnboundedSender<IncomingSession>,
    ) -> crate::errors::Result<Vec<hopr_async_runtime::AbortHandle>> {
        self.msg_sender
            .set(msg_sender)
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        let (session_close_tx, session_close_rx) = futures::channel::mpsc::unbounded();
        self.session_notifiers
            .set((new_session_notifier, session_close_tx))
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        let myself = self.clone();
        let ah_closure_notifications = hopr_async_runtime::spawn_as_abortable!(session_close_rx.for_each_concurrent(
            None,
            move |closed_session_id| {
                let myself = myself.clone();
                async move {
                    trace!(
                        session_id = ?closed_session_id,
                        "sending notification of session closure done by us"
                    );
                    match myself.close_session(closed_session_id, true).await {
                        Ok(closed) if closed => debug!(
                            session_id = ?closed_session_id,
                            "session has been closed by us"
                        ),
                        Err(e) => error!(
                            session_id = ?closed_session_id,
                            error = %e,
                            "cannot initiate session closure notification"
                        ),
                        _ => {}
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

    fn spawn_keep_alive_stream(
        &self,
        session_id: SessionId,
        sender: Arc<CountingSendMsg<S>>,
        routing: DestinationRouting,
    ) -> (KeepAliveController, AbortHandle) {
        let elem = StartProtocol::KeepAlive(session_id.into());

        // The stream is suspended until the caller sets a rate via the Controller
        let (ka_stream, controller) = futures::stream::repeat(elem).rate_limit_per_unit(0, Duration::from_secs(1));

        let (abort_handle, reg) = AbortHandle::new_pair();
        let sender_clone = sender.clone();
        let fwd_routing_clone = routing.clone();

        // This task will automatically terminate once the returned abort handle is used.
        debug!(%session_id, "spawning keep-alive stream");
        hopr_async_runtime::prelude::spawn(
            futures::stream::Abortable::new(ka_stream, reg)
                .map(ApplicationData::try_from)
                .try_for_each_concurrent(None, move |msg| {
                    let sender_clone = sender_clone.clone();
                    let fwd_routing_clone = fwd_routing_clone.clone();
                    async move { sender_clone.send_message(msg, fwd_routing_clone).await }
                })
                .then(move |res| {
                    match res {
                        Ok(_) => debug!(%session_id, "keep-alive stream done"),
                        Err(error) => error!(%session_id, %error, "keep-alive stream failed"),
                    }
                    futures::future::ready(())
                }),
        );

        (KeepAliveController(controller), abort_handle)
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
    ) -> crate::errors::Result<Session> {
        let msg_sender = self.msg_sender.get().ok_or(SessionManagerError::NotStarted)?;

        let (tx_initiation_done, rx_initiation_done) = futures::channel::mpsc::unbounded();
        let challenge = insert_into_next_slot(
            &self.session_initiations,
            |ch| {
                if let Some(challenge) = ch {
                    ((challenge + 1) % hopr_crypto_random::MAX_RANDOM_INTEGER).max(MIN_CHALLENGE)
                } else {
                    hopr_crypto_random::random_integer(MIN_CHALLENGE, None)
                }
            },
            tx_initiation_done,
        )
        .await
        .ok_or(SessionManagerError::NoChallengeSlots)?; // almost impossible with u64

        // Prepare the session initiation message in the Start protocol
        trace!(challenge, ?cfg, "initiating session with config");
        let start_session_msg = StartProtocol::<SessionId>::StartSession(StartInitiation {
            challenge,
            target,
            capabilities: cfg.capabilities,
        });

        let pseudonym = cfg.pseudonym.unwrap_or(HoprPseudonym::random());
        let forward_routing = DestinationRouting::Forward {
            destination,
            pseudonym: Some(pseudonym), // Session must use a fixed pseudonym already
            forward_options: cfg.forward_path_options.clone(),
            return_options: cfg.return_path_options.clone().into(),
        };

        // Send the Session initiation message
        info!(challenge, %pseudonym, %destination, "new session request");
        msg_sender
            .send_message(start_session_msg.try_into()?, forward_routing.clone())
            .await?;

        // Await session establishment response from the Exit node or timeout
        pin_mut!(rx_initiation_done);
        let initiation_done = TryStreamExt::try_next(&mut rx_initiation_done);

        // The timeout is given by the number of hops requested
        let timeout = hopr_async_runtime::prelude::sleep(initiation_timeout_max_one_way(
            self.cfg.initiation_timeout_base,
            cfg.forward_path_options.count_hops() + cfg.return_path_options.count_hops() + 2,
        ));
        pin_mut!(timeout);

        trace!(challenge, "awaiting session establishment");
        match futures::future::select(initiation_done, timeout).await {
            Either::Left((Ok(Some(est)), _)) => {
                // Session has been established, construct it
                let session_id = est.session_id;
                debug!(challenge = est.orig_challenge, ?session_id, "started a new session");

                let (tx, rx) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
                let mut abort_handles = Vec::new();
                let notifier = self
                    .session_notifiers
                    .get()
                    .map(|(_, notifier)| {
                        let notifier = notifier.clone();
                        Box::new(move |session_id: SessionId| {
                            let _ = notifier
                                .unbounded_send(session_id)
                                .inspect_err(|error| error!(%session_id, %error, "failed to notify session closure"));
                        })
                    })
                    .ok_or(SessionManagerError::NotStarted)?;

                let session = if let Some(balancer_config) = cfg.surb_management {
                    let surb_production_counter = Arc::new(AtomicU64::new(0));
                    let surb_consumption_counter = Arc::new(AtomicU64::new(0));

                    // Sender responsible for keep-alive and Session data is counting produced SURBs
                    let sender = Arc::new(CountingSendMsg::new(
                        msg_sender.clone(),
                        surb_production_counter.clone(),
                    ));

                    // Spawn the SURB-bearing keep alive stream
                    let (ka_controller, ka_abort_handle) =
                        self.spawn_keep_alive_stream(session_id, sender.clone(), forward_routing.clone());
                    abort_handles.push(ka_abort_handle);

                    // Spawn the SURB balancer, which will decide on the initial SURB rate.
                    debug!(%session_id, ?balancer_config, "spawning SURB balancer");
                    let mut balancer = SurbBalancer::new(
                        session_id,
                        surb_production_counter,
                        surb_consumption_counter.clone(),
                        ka_controller,
                        balancer_config,
                    );

                    let (surbs_ready_tx, surbs_ready_rx) = futures::channel::oneshot::channel();
                    let mut surbs_ready_tx = Some(surbs_ready_tx);
                    let (balancer_abort_handle, reg) = AbortHandle::new_pair();
                    hopr_async_runtime::prelude::spawn(
                        futures::stream::Abortable::new(
                            futures_time::stream::interval(
                                self.cfg
                                    .balancer_sampling_interval
                                    .max(MIN_BALANCER_SAMPLING_INTERVAL)
                                    .into(),
                            ),
                            reg,
                        )
                        .for_each(move |_| {
                            let level = balancer.update();
                            // We will wait until at least half of the target buffer has been sent
                            if surbs_ready_tx.is_some() && level >= balancer_config.target_surb_buffer_size / 2 {
                                let _ = surbs_ready_tx.take().unwrap().send(level);
                            }
                            futures::future::ready(())
                        })
                        .then(move |_| {
                            debug!(%session_id, "balancer done");
                            futures::future::ready(())
                        }),
                    );
                    abort_handles.push(balancer_abort_handle);

                    // Wait for enough SURBs to be sent to the counterparty
                    // TODO: consider making this interactive = other party reports the exact level periodically
                    let timeout = hopr_async_runtime::prelude::sleep(SESSION_READINESS_TIMEOUT);
                    pin_mut!(timeout);
                    match futures::future::select(surbs_ready_rx, timeout).await {
                        Either::Left((Ok(surb_level), _)) => {
                            info!(%session_id, surb_level, "session is ready");
                        }
                        Either::Left((Err(_), _)) => {
                            return Err(
                                SessionManagerError::Other("surb balancer was cancelled prematurely".into()).into(),
                            );
                        }
                        Either::Right(_) => {
                            warn!(%session_id, "session didn't reach target SURB buffer size");
                        }
                    }

                    Session::new(
                        session_id,
                        forward_routing.clone(),
                        cfg.capabilities,
                        sender,
                        Box::pin(rx.inspect(move |_| {
                            // Received packets = SURB consumption estimate
                            surb_consumption_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        })),
                        Some(notifier),
                    )
                } else {
                    warn!(%session_id, "session ready without SURB balancing");
                    Session::new(
                        session_id,
                        forward_routing.clone(),
                        cfg.capabilities,
                        Arc::new(msg_sender.clone()),
                        Box::pin(rx),
                        Some(notifier),
                    )
                };

                // We currently do not support loopback Sessions on ourselves.
                if let moka::ops::compute::CompResult::Inserted(_) = self
                    .sessions
                    .entry(session_id)
                    .and_compute_with(|entry| {
                        futures::future::ready(if entry.is_none() {
                            moka::ops::compute::Op::Put(CachedSession {
                                session_tx: Arc::new(tx),
                                routing_opts: forward_routing,
                                abort_handles,
                            })
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

                    Ok(session)
                } else {
                    // Session already exists; it means it is most likely a loopback attempt
                    error!(%session_id, "session already exists - loopback attempt");
                    Err(SessionManagerError::Loopback.into())
                }
            }
            Either::Left((Ok(None), _)) => Err(SessionManagerError::Other(
                "internal error: sender has been closed without completing the session establishment".into(),
            )
            .into()),
            Either::Left((Err(e), _)) => {
                // The other side did not allow us to establish a session
                error!(
                    challenge = e.challenge,
                    error = ?e,
                    "the other party rejected the session initiation with error"
                );
                Err(TransportSessionError::Rejected(e.reason))
            }
            Either::Right(_) => {
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
            Ok(self
                .msg_sender
                .get()
                .ok_or(SessionManagerError::NotStarted)?
                .send_message(
                    StartProtocol::KeepAlive((*id).into()).try_into()?,
                    session_data.routing_opts.clone(),
                )
                .await?)
        } else {
            Err(SessionManagerError::NonExistingSession.into())
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
        data: ApplicationData,
    ) -> crate::errors::Result<DispatchResult> {
        if data.application_tag == StartProtocol::<SessionId>::START_PROTOCOL_MESSAGE_TAG {
            // This is a Start protocol message, so we handle it
            trace!(tag = %data.application_tag, "dispatching Start protocol message");
            return self
                .handle_start_protocol_message(pseudonym, data)
                .await
                .map(|_| DispatchResult::Processed);
        } else if self.cfg.session_tag_range.contains(&data.application_tag.as_u64()) {
            let session_id = SessionId::new(data.application_tag, pseudonym);

            return if let Some(session_data) = self.sessions.get(&session_id).await {
                trace!(?session_id, "received data for a registered session");

                Ok(session_data
                    .session_tx
                    .unbounded_send(data.plain_text)
                    .map(|_| DispatchResult::Processed)
                    .map_err(|e| SessionManagerError::Other(e.to_string()))?)
            } else {
                error!(%session_id, "received data from an unestablished session");
                Err(TransportSessionError::UnknownData)
            };
        }

        trace!(%data.application_tag, "received data not associated with session protocol or any existing session");
        Ok(DispatchResult::Unrelated(data))
    }

    async fn handle_start_protocol_message(
        &self,
        pseudonym: HoprPseudonym,
        data: ApplicationData,
    ) -> crate::errors::Result<()> {
        match StartProtocol::<SessionId>::try_from(data)? {
            StartProtocol::StartSession(session_req) => {
                trace!(challenge = session_req.challenge, "received session initiation request");

                debug!(%pseudonym, "got new session request, searching for a free session slot");

                let msg_sender = self.msg_sender.get().ok_or(SessionManagerError::NotStarted)?;

                let (new_session_notifier, close_session_notifier) = self
                    .session_notifiers
                    .get()
                    .cloned()
                    .ok_or(SessionManagerError::NotStarted)?;

                // Reply routing uses SURBs only with the pseudonym of this Session's ID
                let reply_routing = DestinationRouting::Return(SurbMatcher::Pseudonym(pseudonym));

                // Construct the session
                let (tx_session_data, rx_session_data) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
                if let Some(session_id) = insert_into_next_slot(
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
                    CachedSession {
                        session_tx: Arc::new(tx_session_data),
                        routing_opts: reply_routing.clone(),
                        abort_handles: vec![],
                    },
                )
                .await
                {
                    debug!(?session_id, "assigning a new session");

                    let session = Session::new(
                        session_id,
                        reply_routing.clone(),
                        session_req.capabilities,
                        Arc::new(msg_sender.clone()),
                        Box::pin(rx_session_data),
                        Some(Box::new(move |session_id: SessionId| {
                            if let Err(error) = close_session_notifier.unbounded_send(session_id) {
                                error!(%session_id, %error, "failed to notify session closure");
                            }
                        })),
                    );

                    // Extract useful information about the session from the Start protocol message
                    let incoming_session = IncomingSession {
                        session,
                        target: session_req.target,
                    };

                    // Notify that a new incoming session has been created
                    if let Err(error) = new_session_notifier.unbounded_send(incoming_session) {
                        warn!(%error, "failed to send session to incoming session queue");
                    }

                    trace!(?session_id, "session notification sent");

                    // Notify the sender that the session has been established.
                    // Set our peer ID in the session ID sent back to them.
                    let data = StartProtocol::SessionEstablished(StartEstablished {
                        orig_challenge: session_req.challenge,
                        session_id,
                    });

                    msg_sender
                        .send_message(data.try_into()?, reply_routing)
                        .await
                        .map_err(|e| {
                            SessionManagerError::Other(format!("failed to send session establishment message: {e}"))
                        })?;

                    info!(%session_id, "new session established");

                    #[cfg(all(feature = "prometheus", not(test)))]
                    {
                        METRIC_NUM_ESTABLISHED_SESSIONS.increment();
                        METRIC_ACTIVE_SESSIONS.increment(1.0);
                    }
                } else {
                    error!(
                        %pseudonym,
                        "failed to reserve a new session slot"
                    );

                    // Notify the sender that the session could not be established
                    let reason = StartErrorReason::NoSlotsAvailable;
                    let data = StartProtocol::<SessionId>::SessionError(StartErrorType {
                        challenge: session_req.challenge,
                        reason,
                    });

                    msg_sender
                        .send_message(data.try_into()?, reply_routing.clone())
                        .await
                        .map_err(|e| {
                            SessionManagerError::Other(format!(
                                "failed to send session establishment error message: {e}"
                            ))
                        })?;

                    trace!(%pseudonym, "session establishment failure message sent");

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_SENT_SESSION_ERRS.increment(&[&reason.to_string()])
                }
            }
            StartProtocol::SessionEstablished(est) => {
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
                    error!(%session_id, challenge, "session establishment attempt expired");
                }
            }
            StartProtocol::SessionError(error) => {
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
            StartProtocol::CloseSession(session_id) => {
                trace!(?session_id, "received session close request");
                match self.close_session(session_id, false).await {
                    Ok(closed) if closed => debug!(?session_id, "session has been closed by the other party"),
                    Err(error) => error!(
                        %session_id,
                        %error,
                        "session could not be closed on other party's request"
                    ),
                    _ => {}
                }
            }
            StartProtocol::KeepAlive(msg) => {
                let session_id = msg.id;
                if self.sessions.get(&session_id).await.is_some() {
                    trace!(?session_id, "received keep-alive request");
                } else {
                    error!(%session_id, "received keep-alive request for an unknown session");
                }
            }
        }

        Ok(())
    }

    async fn close_session(&self, session_id: SessionId, notify_closure: bool) -> crate::errors::Result<bool> {
        if let Some(session_data) = self.sessions.remove(&session_id).await {
            // Notification is not sent only when closing in response to the other party's request
            if notify_closure {
                trace!(?session_id, "sending session termination");
                self.msg_sender
                    .get()
                    .ok_or(SessionManagerError::NotStarted)?
                    .send_message(
                        StartProtocol::CloseSession(session_id).try_into()?,
                        session_data.routing_opts,
                    )
                    .await?;
            }

            // Closing the data sender on the session will cause the Session to terminate
            session_data.session_tx.close_channel();
            trace!(?session_id, "data tx channel closed on session");

            // Terminate any additional tasks spawned by the Session
            session_data.abort_handles.into_iter().for_each(|h| h.abort());

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_ACTIVE_SESSIONS.decrement(1.0);
            Ok(true)
        } else {
            // Do not treat this as an error
            debug!(
                ?session_id,
                "could not find session id to close, maybe the session is already closed"
            );
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use futures::AsyncWriteExt;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use hopr_primitive_types::prelude::Address;
    use tokio::time::timeout;

    use super::*;
    use crate::{
        Capabilities, Capability, balancer::SurbBalancerConfig, initiation::StartProtocolDiscriminants,
        types::SessionTarget,
    };

    mockall::mock! {
        MsgSender {}
        impl Clone for MsgSender {
            fn clone(&self) -> Self;
        }
        impl SendMsg for MsgSender {
            fn send_message<'life0, 'async_trait>
            (
                &'life0 self,
                data: ApplicationData,
                routing: DestinationRouting,
            )
            -> std::pin::Pin<Box<dyn std::future::Future<Output=std::result::Result<(),TransportSessionError>> + Send + 'async_trait>>
            where
                'life0: 'async_trait,
                Self: Sync + 'async_trait;
        }
    }

    fn msg_type(data: &ApplicationData, expected: StartProtocolDiscriminants) -> bool {
        expected
            == StartProtocolDiscriminants::from(
                StartProtocol::<SessionId>::decode(data.application_tag, &data.plain_text)
                    .expect("failed to parse message"),
            )
    }

    #[tokio::test]
    async fn test_insert_into_next_slot() -> anyhow::Result<()> {
        let cache = moka::future::Cache::new(10);

        for i in 0..5 {
            let v = insert_into_next_slot(&cache, |prev| prev.map(|v| (v + 1) % 5).unwrap_or(0), "foo".to_string())
                .await
                .ok_or(anyhow!("should insert"))?;
            assert_eq!(v, i);
            assert_eq!(Some("foo".to_string()), cache.get(&i).await);
        }

        assert!(
            insert_into_next_slot(&cache, |prev| prev.map(|v| (v + 1) % 5).unwrap_or(0), "foo".to_string())
                .await
                .is_none(),
            "must not find slot when full"
        );

        Ok(())
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
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Bob clones transport for Session
        bob_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(MockMsgSender::new);

        // Bob sends the SessionEstablished message
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();

                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Alice clones transport for Session
        alice_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(MockMsgSender::new);

        // Alice sends the CloseSession message to initiate closure
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::CloseSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Bob sends the CloseSession message to confirm
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::CloseSession)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        ahs.extend(alice_mgr.start(alice_transport, new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::unbounded();
        ahs.extend(bob_mgr.start(bob_transport, new_session_tx_bob)?);

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
            alice_session.capabilities(),
            &Capabilities::from(Capability::Segmentation)
        );
        assert_eq!(alice_session.capabilities(), bob_session.session.capabilities());
        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        tokio::time::sleep(Duration::from_millis(100)).await;
        alice_session.close().await?;

        tokio::time::sleep(Duration::from_millis(100)).await;
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
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Bob clones transport for Session
        bob_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(MockMsgSender::new);

        // Bob sends the SessionEstablished message
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();

                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Alice clones transport for Session
        alice_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(MockMsgSender::new);

        // Alice sends the CloseSession message to initiate closure
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::CloseSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Bob sends the CloseSession message to confirm
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::CloseSession)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();

                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        ahs.extend(alice_mgr.start(alice_transport, new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::unbounded();
        ahs.extend(bob_mgr.start(bob_transport, new_session_tx_bob)?);

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
            alice_session.capabilities(),
            &Capabilities::from(Capability::Segmentation)
        );
        assert_eq!(alice_session.capabilities(), bob_session.session.capabilities());
        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        // Let the session timeout
        tokio::time::sleep(Duration::from_millis(300)).await;

        futures::stream::iter(ahs)
            .for_each(|ah| async move { ah.abort() })
            .await;

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
                CachedSession {
                    session_tx: Arc::new(dummy_tx),
                    routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(alice_pseudonym)),
                    abort_handles: Vec::new(),
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
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Bob sends the SessionError message
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::SessionError)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        let mut jhs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        jhs.extend(alice_mgr.start(alice_transport, new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, _) = futures::channel::mpsc::unbounded();
        jhs.extend(bob_mgr.start(bob_transport, new_session_tx_bob)?);

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
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                // But the message is again processed by Alice due to Loopback
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Alice clones transport for Session (as Bob)
        alice_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(MockMsgSender::new);

        // Alice sends the SessionEstablished message (as Bob)
        let alice_mgr_clone = alice_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();

                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Alice clones transport for Session
        alice_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(MockMsgSender::new);

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        alice_mgr.start(alice_transport, new_session_tx_alice)?;

        let alice_session = alice_mgr
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

        println!("{alice_session:?}");
        assert!(matches!(
            alice_session,
            Err(TransportSessionError::Manager(SessionManagerError::Loopback))
        ));

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
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(|_, _| Box::pin(async { Ok(()) }));

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        alice_mgr.start(alice_transport, new_session_tx_alice)?;

        // Start Bob
        let (new_session_tx_bob, _) = futures::channel::mpsc::unbounded();
        bob_mgr.start(bob_transport, new_session_tx_bob)?;

        let result = alice_mgr
            .new_session(
                bob_peer,
                SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:80".parse()?)),
                SessionClientConfig {
                    capabilities: Capabilities::empty(),
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
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::StartSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Bob clones transport for Session
        bob_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(MockMsgSender::new);

        // Bob sends the SessionEstablished message
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::SessionEstablished)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // On cloned SendMsg: Alice sends the KeepAlive messages
        let bob_mgr_clone = bob_mgr.clone();
        let mut alice_session_transport = MockMsgSender::new();
        alice_session_transport
            .expect_send_message()
            .times(5..)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::KeepAlive)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Alice clones transport for Session
        alice_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| alice_session_transport);

        // Alice sends the CloseSession message to initiate closure
        let bob_mgr_clone = bob_mgr.clone();
        alice_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::CloseSession)
                    && matches!(peer, DestinationRouting::Forward { destination, .. } if destination == &bob_peer)
            })
            .returning(move |data, _| {
                let bob_mgr_clone = bob_mgr_clone.clone();
                Box::pin(async move {
                    bob_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        // Bob sends the CloseSession message to confirm
        let alice_mgr_clone = alice_mgr.clone();
        bob_transport
            .expect_send_message()
            .once()
            .in_sequence(&mut sequence)
            .withf(move |data, peer| {
                msg_type(data, StartProtocolDiscriminants::CloseSession)
                    && matches!(peer, DestinationRouting::Return(SurbMatcher::Pseudonym(p)) if p == &alice_pseudonym)
            })
            .returning(move |data, _| {
                let alice_mgr_clone = alice_mgr_clone.clone();
                Box::pin(async move {
                    alice_mgr_clone.dispatch_message(alice_pseudonym, data).await?;
                    Ok(())
                })
            });

        let mut ahs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        ahs.extend(alice_mgr.start(alice_transport, new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::unbounded();
        ahs.extend(bob_mgr.start(bob_transport, new_session_tx_bob)?);

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
                        surb_management: Some(SurbBalancerConfig {
                            target_surb_buffer_size: 10,
                            max_surbs_per_sec: 100,
                        }),
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

        // Let the Surb balancer send enough KeepAlive messages
        tokio::time::sleep(Duration::from_millis(3000)).await;
        alice_session.close().await?;

        tokio::time::sleep(Duration::from_millis(300)).await;
        futures::stream::iter(ahs)
            .for_each(|ah| async move { ah.abort() })
            .await;

        Ok(())
    }
}
