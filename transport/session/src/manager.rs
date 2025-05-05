use crate::balancer::{keep_alive_stream, CountingSendMsg, KeepAliveController, SurbBalancer};
use crate::errors::{SessionManagerError, TransportSessionError};
use crate::initiation::{
    StartChallenge, StartErrorReason, StartErrorType, StartEstablished, StartInitiation, StartProtocol,
};
use crate::traits::SendMsg;
use crate::{IncomingSession, Session, SessionClientConfig, SessionId, SessionTarget};
use futures::channel::mpsc::UnboundedSender;
use futures::future::Either;
use futures::stream::AbortHandle;
use futures::{pin_mut, FutureExt, StreamExt, TryStreamExt};
use hopr_internal_types::prelude::{ApplicationData, HoprPseudonym, Tag};
use hopr_network_types::prelude::*;
use hopr_primitive_types::prelude::Address;
use std::ops::Range;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

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
    #[default(_code = "16..1024")]
    pub session_tag_range: Range<Tag>,

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
async fn insert_into_next_slot<K, V, F>(cache: &moka::future::Cache<K, V>, gen: F, value: V) -> Option<K>
where
    K: Copy + std::hash::Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    F: Fn(Option<K>) -> K,
{
    let initial = gen(None);
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
        next = gen(Some(next));

        // If generated keys made it to full loop, return failure
        if next == initial {
            return None;
        }
    }
}

/// The first challenge value used in Start protocol to initiate a session.
pub(crate) const MIN_CHALLENGE: StartChallenge = 1;

/// How often the SURB rate sampling and SURB balancer control decisions are done.
pub(crate) const BALANCER_SAMPLE_INTERVAL: Duration = Duration::from_secs(1);

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
}

/// Indicates the result of processing a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DispatchResult {
    /// Session or Start protocol message has been processed successfully.
    Processed,
    /// The message was not related to Start or Session protocol.
    Unrelated(ApplicationData),
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

/// The Minimum Session tag due to Start-protocol messages.
pub const MIN_SESSION_TAG_RANGE_RESERVATION: Tag = 16;

impl<S: SendMsg + Clone + Send + Sync + 'static> SessionManager<S> {
    /// Creates a new instance given the `PeerId` and [config](SessionManagerConfig).
    pub fn new(mut cfg: SessionManagerConfig) -> Self {
        // Accommodate the lower bound if too low.
        if cfg.session_tag_range.start < MIN_SESSION_TAG_RANGE_RESERVATION {
            let diff = MIN_SESSION_TAG_RANGE_RESERVATION - cfg.session_tag_range.start;
            cfg.session_tag_range = MIN_SESSION_TAG_RANGE_RESERVATION..cfg.session_tag_range.end + diff;
        }

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_ACTIVE_SESSIONS.set(0.0);

        let msg_sender = Arc::new(OnceLock::new());
        Self {
            msg_sender: msg_sender.clone(),
            session_initiations: moka::future::Cache::builder()
                .max_capacity(cfg.session_tag_range.clone().count() as u64)
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
    ) -> crate::errors::Result<Vec<hopr_async_runtime::prelude::JoinHandle<()>>> {
        self.msg_sender
            .set(msg_sender)
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        let (session_close_tx, session_close_rx) = futures::channel::mpsc::unbounded();
        self.session_notifiers
            .set((new_session_notifier, session_close_tx))
            .map_err(|_| SessionManagerError::AlreadyStarted)?;

        let myself = self.clone();
        let jh_closure_notifications =
            hopr_async_runtime::prelude::spawn(session_close_rx.for_each_concurrent(None, move |closed_session_id| {
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
            }));

        // This is necessary to evict expired entries from the caches if
        // no session-related operations happen at all.
        // This ensures the dangling expired sessions are properly closed
        // and their closure is timely notified to the other party.
        let myself = self.clone();
        let jh_session_expiration = hopr_async_runtime::prelude::spawn(async move {
            let jitter = hopr_crypto_random::random_float_in_range(1.0..1.5);
            let timeout = 2 * initiation_timeout_max_one_way(
                myself.cfg.initiation_timeout_base,
                RoutingOptions::MAX_INTERMEDIATE_HOPS,
            )
            .min(myself.cfg.idle_timeout)
            .mul_f64(jitter)
                / 2;
            loop {
                hopr_async_runtime::prelude::sleep(timeout).await;
                trace!("executing session cache evictions");
                futures::join!(
                    myself.sessions.run_pending_tasks(),
                    myself.session_initiations.run_pending_tasks()
                );
            }
        });

        Ok(vec![jh_closure_notifications, jh_session_expiration])
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
        // The stream is suspended until the caller sets a rate via the Controller
        let (ka_stream, ka_controller, abort_handle) = keep_alive_stream(session_id);
        let sender_clone = sender.clone();
        let fwd_routing_clone = routing.clone();

        // This task will automatically terminate once the returned abort handle is used.
        debug!(%session_id, "spawning keep-alive stream");
        hopr_async_runtime::prelude::spawn(
            ka_stream
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

        (ka_controller, abort_handle)
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
            capabilities: cfg.capabilities.iter().copied().collect(),
        });

        let forward_routing = DestinationRouting::Forward {
            destination,
            pseudonym: cfg.pseudonym,
            forward_options: cfg.forward_path_options.clone(),
            return_options: cfg.return_path_options.clone().into(),
        };

        // Send the Session initiation message
        trace!(challenge, "sending new session request");
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

                // Insert the Session object, forcibly overwrite any other session with the same ID
                let (tx, rx) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
                self.sessions
                    .insert(
                        session_id,
                        CachedSession {
                            session_tx: Arc::new(tx),
                            routing_opts: forward_routing.clone(),
                        },
                    )
                    .await;

                #[cfg(all(feature = "prometheus", not(test)))]
                {
                    METRIC_NUM_INITIATED_SESSIONS.increment();
                    METRIC_ACTIVE_SESSIONS.increment(1.0);
                }

                let notifier = self
                    .session_notifiers
                    .get()
                    .map(|(_, c)| c.clone())
                    .ok_or(SessionManagerError::NotStarted)?;
                if let Some(balancer_config) = cfg.surb_management {
                    let surb_production_counter = Arc::new(AtomicU64::new(0));
                    let surb_consumption_counter = Arc::new(AtomicU64::new(0));

                    // Sender responsible for keep-alive and Session data is counting produced SURBs
                    let sender = Arc::new(CountingSendMsg::new(
                        msg_sender.clone(),
                        surb_production_counter.clone(),
                    ));

                    // Spawn the SURB-bearing keep alive stream
                    let (ka_controller, ka_abort) =
                        self.spawn_keep_alive_stream(session_id, sender.clone(), forward_routing.clone());

                    let (balancer_abort, balancer_ar) = AbortHandle::new_pair();
                    let session = Session::new(
                        session_id,
                        forward_routing,
                        cfg.capabilities.into_iter().collect(),
                        sender,
                        rx,
                        Some(Box::new(move |session_id: SessionId| {
                            let _ = notifier
                                .unbounded_send(session_id)
                                .inspect_err(|error| error!(%session_id, %error, "failed to notify session closure"));

                            // Terminate also the SURB-bearing keep-alive sending and the Balancer
                            ka_abort.abort();
                            balancer_abort.abort();
                        })),
                        surb_consumption_counter.clone(), // Received packets = SURB consumption estimate
                    );

                    // Spawn the SURB balancer, which will decided on the initial SURB rate.
                    debug!(%session_id, ?balancer_config, "spawning SURB balancer");
                    let mut balancer = SurbBalancer::new(
                        session_id,
                        surb_production_counter,
                        surb_consumption_counter,
                        ka_controller,
                        balancer_config,
                    );
                    hopr_async_runtime::prelude::spawn(
                        futures::stream::Abortable::new(
                            futures_time::stream::interval(BALANCER_SAMPLE_INTERVAL.into()),
                            balancer_ar,
                        )
                        .for_each(move |_| {
                            balancer.update();
                            futures::future::ready(())
                        })
                        .then(move |_| {
                            debug!(%session_id, "balancer done");
                            futures::future::ready(())
                        }),
                    );

                    Ok(session)
                } else {
                    Ok(Session::new(
                        session_id,
                        forward_routing,
                        cfg.capabilities.into_iter().collect(),
                        Arc::new(msg_sender.clone()),
                        rx,
                        Some(Box::new(move |session_id: SessionId| {
                            let _ = notifier
                                .unbounded_send(session_id)
                                .inspect_err(|error| error!(%session_id, %error, "failed to notify session closure"));
                        })),
                        Arc::new(AtomicU64::new(0)),
                    ))
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
        if (0..self.cfg.session_tag_range.start).contains(&data.application_tag) {
            trace!(tag = data.application_tag, "dispatching Start protocol message");
            return self
                .handle_start_protocol_message(pseudonym, data)
                .await
                .map(|_| DispatchResult::Processed);
        } else if self.cfg.session_tag_range.contains(&data.application_tag) {
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
                        let next_tag = match sid {
                            Some(session_id) => ((session_id.tag() + 1) % self.cfg.session_tag_range.end)
                                .max(self.cfg.session_tag_range.start),
                            None => hopr_crypto_random::random_integer(
                                self.cfg.session_tag_range.start as u64,
                                Some(self.cfg.session_tag_range.end as u64),
                            ) as u16,
                        };
                        SessionId::new(next_tag, pseudonym)
                    },
                    CachedSession {
                        session_tx: Arc::new(tx_session_data),
                        routing_opts: reply_routing.clone(),
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
                        rx_session_data,
                        Some(Box::new(move |session_id: SessionId| {
                            if let Err(error) = close_session_notifier.unbounded_send(session_id) {
                                error!(%session_id, %error, "failed to notify session closure");
                            }
                        })),
                        Arc::new(AtomicU64::new(0)),
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
            StartProtocol::KeepAlive(session_id) => {
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
    use super::*;

    use crate::types::SessionTarget;
    use crate::Capability;
    use anyhow::anyhow;
    use async_std::prelude::FutureExt;
    use async_trait::async_trait;
    use futures::AsyncWriteExt;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::keypairs::ChainKeypair;
    use hopr_crypto_types::prelude::Keypair;
    use hopr_primitive_types::prelude::Address;

    use crate::balancer::SurbBalancerConfig;
    use crate::initiation::StartProtocolDiscriminants;

    mockall::mock! {
        MsgSender {}
        impl Clone for MsgSender {
            fn clone(&self) -> Self;
        }
        #[async_trait]
        impl SendMsg for MsgSender {
            async fn send_message(
                &self,
                data: ApplicationData,
                routing: DestinationRouting,
            ) -> std::result::Result<(), TransportSessionError>;
        }
    }

    fn msg_type(data: &ApplicationData, expected: StartProtocolDiscriminants) -> bool {
        expected
            == StartProtocolDiscriminants::from(
                StartProtocol::<SessionId>::decode(data.application_tag, &data.plain_text)
                    .expect("failed to parse message"),
            )
    }

    #[async_std::test]
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

    #[test_log::test(async_std::test)]
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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        // Bob clones transport for Session
        bob_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| MockMsgSender::new());

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
                async_std::task::block_on(alice_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        // Alice clones transport for Session
        alice_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| MockMsgSender::new());

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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
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
                async_std::task::block_on(alice_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        let mut jhs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        jhs.extend(alice_mgr.start(alice_transport, new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::unbounded();
        jhs.extend(bob_mgr.start(bob_transport, new_session_tx_bob)?);

        let target = SealedHost::Plain("127.0.0.1:80".parse()?);

        pin_mut!(new_session_rx_bob);
        let (alice_session, bob_session) = futures::future::join(
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
        )
        .timeout(Duration::from_secs(2))
        .await?;

        let mut alice_session = alice_session?;
        let bob_session = bob_session.ok_or(anyhow!("bob must get an incoming session"))?;

        assert!(
            alice_session.capabilities().len() == 1 && alice_session.capabilities().contains(&Capability::Segmentation)
        );
        assert_eq!(alice_session.capabilities(), bob_session.session.capabilities());
        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        async_std::task::sleep(Duration::from_millis(100)).await;
        alice_session.close().await?;

        async_std::task::sleep(Duration::from_millis(100)).await;
        futures::stream::iter(jhs)
            .for_each(hopr_async_runtime::prelude::cancel_join_handle)
            .await;

        Ok(())
    }

    #[test_log::test(async_std::test)]
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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        // Bob clones transport for Session
        bob_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| MockMsgSender::new());

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
                async_std::task::block_on(alice_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        // Alice clones transport for Session
        alice_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| MockMsgSender::new());

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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
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
                async_std::task::block_on(alice_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        let mut jhs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        jhs.extend(alice_mgr.start(alice_transport, new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::unbounded();
        jhs.extend(bob_mgr.start(bob_transport, new_session_tx_bob)?);

        let target = SealedHost::Plain("127.0.0.1:80".parse()?);

        pin_mut!(new_session_rx_bob);
        let (alice_session, bob_session) = futures::future::join(
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
        )
        .timeout(Duration::from_secs(2))
        .await?;

        let alice_session = alice_session?;
        let bob_session = bob_session.ok_or(anyhow!("bob must get an incoming session"))?;

        assert!(
            alice_session.capabilities().len() == 1 && alice_session.capabilities().contains(&Capability::Segmentation)
        );
        assert_eq!(alice_session.capabilities(), bob_session.session.capabilities());
        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        // Let the session timeout
        async_std::task::sleep(Duration::from_millis(300)).await;

        futures::stream::iter(jhs)
            .for_each(hopr_async_runtime::prelude::cancel_join_handle)
            .await;

        Ok(())
    }

    #[test_log::test(async_std::test)]
    async fn session_manager_should_not_allow_establish_session_when_tag_range_is_used_up() -> anyhow::Result<()> {
        let alice_pseudonym = HoprPseudonym::random();
        let bob_peer: Address = (&ChainKeypair::random()).into();

        let cfg = SessionManagerConfig {
            session_tag_range: 16..17, // Slot for exactly one session
            ..Default::default()
        };

        let alice_mgr = SessionManager::new(Default::default());
        let bob_mgr = SessionManager::new(cfg);

        // Occupy the only free slot with tag 16
        let (dummy_tx, _) = futures::channel::mpsc::unbounded();
        bob_mgr
            .sessions
            .insert(
                SessionId::new(16, alice_pseudonym),
                CachedSession {
                    session_tx: Arc::new(dummy_tx),
                    routing_opts: DestinationRouting::Return(SurbMatcher::Pseudonym(alice_pseudonym)),
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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
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
                async_std::task::block_on(alice_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
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
                    capabilities: vec![],
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

    #[test_log::test(async_std::test)]
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
            .returning(|_, _| Ok(()));

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
                    capabilities: vec![],
                    pseudonym: None,
                    surb_management: None,
                    ..Default::default()
                },
            )
            .await;

        assert!(matches!(result, Err(TransportSessionError::Timeout)));

        Ok(())
    }

    #[test_log::test(async_std::test)]
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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        // Bob clones transport for Session
        bob_transport
            .expect_clone()
            .once()
            .in_sequence(&mut sequence)
            .return_once(|| MockMsgSender::new());

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
                async_std::task::block_on(alice_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
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
                async_std::task::block_on(bob_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
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
                async_std::task::block_on(alice_mgr_clone.dispatch_message(alice_pseudonym, data))?;
                Ok(())
            });

        let mut jhs = Vec::new();

        // Start Alice
        let (new_session_tx_alice, _) = futures::channel::mpsc::unbounded();
        jhs.extend(alice_mgr.start(alice_transport, new_session_tx_alice)?);

        // Start Bob
        let (new_session_tx_bob, new_session_rx_bob) = futures::channel::mpsc::unbounded();
        jhs.extend(bob_mgr.start(bob_transport, new_session_tx_bob)?);

        let target = SealedHost::Plain("127.0.0.1:80".parse()?);

        pin_mut!(new_session_rx_bob);
        let (alice_session, bob_session) = futures::future::join(
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
        )
        .timeout(Duration::from_secs(2))
        .await?;

        let mut alice_session = alice_session?;
        let bob_session = bob_session.ok_or(anyhow!("bob must get an incoming session"))?;

        assert!(matches!(bob_session.target, SessionTarget::TcpStream(host) if host == target));

        // Let the Surb balancer send enough KeepAlive messages
        async_std::task::sleep(Duration::from_millis(3000)).await;
        alice_session.close().await?;

        async_std::task::sleep(Duration::from_millis(300)).await;
        futures::stream::iter(jhs)
            .for_each(hopr_async_runtime::prelude::cancel_join_handle)
            .await;

        Ok(())
    }
}
