//! HOPR library creating a unified [`Hopr`] object that can be used on its own,
//! as well as integrated into other systems and libraries.
//!
//! The [`Hopr`] object is standalone, meaning that once it is constructed and run,
//! it will perform its functionality autonomously. The API it offers serves as a
//! high-level integration point for other applications and utilities, but offers
//! a complete and fully featured HOPR node stripped from top level functionality
//! such as the REST API, key management...
//!
//! The intended way to use hopr_lib is for a specific tool to be built on top of it;
//! should the default `hoprd` implementation not be acceptable.
//!
//! For most of the practical use cases, the `hoprd` application should be a preferable
//! choice.

/// Helper functions.
mod helpers;

/// Configuration-related public types
pub mod config;
/// Various public constants.
pub mod constants;
/// Lists all errors thrown from this library.
pub mod errors;

/// Utility module with helper types and functionality over hopr-lib behavior.
pub mod utils;

/// Public traits for interactions with this library.
pub mod traits;

/// Functionality related to the HOPR node state.
pub mod state;

#[cfg(any(feature = "testing", test))]
pub mod testing;

/// Exports of libraries necessary for API and interface operations.
#[doc(hidden)]
pub mod exports {
    pub use hopr_api as api;

    pub mod types {
        pub use hopr_internal_types as internal;
        pub use hopr_primitive_types as primitive;
    }

    pub mod crypto {
        pub use hopr_crypto_keypair as keypair;
        pub use hopr_crypto_types as types;
    }

    pub mod network {
        pub use hopr_network_types as types;
    }

    pub use hopr_transport as transport;
}

/// Export of relevant types for easier integration.
#[doc(hidden)]
pub mod prelude {
    pub use super::exports::{
        crypto::{
            keypair::key_pair::HoprKeys,
            types::prelude::{ChainKeypair, Hash, OffchainKeypair},
        },
        network::types::{
            prelude::ForeignDataMode,
            udp::{ConnectedUdpStream, UdpStreamParallelism},
        },
        transport::{OffchainPublicKey, socket::HoprSocket},
        types::primitive::prelude::Address,
    };
}

use std::{
    convert::identity,
    num::NonZeroUsize,
    sync::{Arc, OnceLock, atomic::Ordering},
    time::Duration,
};

use futures::{FutureExt, SinkExt, StreamExt, TryFutureExt, channel::mpsc::channel};
use hopr_api::{
    chain::{AccountSelector, AnnouncementError, ChannelSelector, *},
    db::{HoprNodeDbApi, PeerStatus, TicketMarker, TicketSelector},
};
use hopr_async_runtime::prelude::spawn;
pub use hopr_async_runtime::{Abortable, AbortableList};
pub use hopr_crypto_keypair::key_pair::{HoprKeys, IdentityRetrievalModes};
pub use hopr_crypto_types::prelude::*;
pub use hopr_internal_types::prelude::*;
pub use hopr_network_types::prelude::*;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_platform::time::native::current_time;
pub use hopr_primitive_types::prelude::*;
#[cfg(feature = "runtime-tokio")]
pub use hopr_transport::transfer_session;
pub use hopr_transport::*;
use tracing::{debug, error, info, trace, warn};

pub use crate::{
    config::SafeModule,
    constants::{MIN_NATIVE_BALANCE, SUGGESTED_NATIVE_BALANCE},
    errors::{HoprLibError, HoprStatusError},
    state::{HoprLibProcess, HoprState},
    traits::chain::{CloseChannelResult, OpenChannelResult},
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_PROCESS_START_TIME:  hopr_metrics::SimpleGauge =  hopr_metrics::SimpleGauge::new(
        "hopr_start_time",
        "The unix timestamp in seconds at which the process was started"
    ).unwrap();
    static ref METRIC_HOPR_LIB_VERSION:  hopr_metrics::MultiGauge =  hopr_metrics::MultiGauge::new(
        "hopr_lib_version",
        "Executed version of hopr-lib",
        &["version"]
    ).unwrap();
    static ref METRIC_HOPR_NODE_INFO:  hopr_metrics::MultiGauge =  hopr_metrics::MultiGauge::new(
        "hopr_node_addresses",
        "Node on-chain and off-chain addresses",
        &["peerid", "address", "safe_address", "module_address"]
    ).unwrap();
}

pub struct DummyCoverTrafficType {
    #[allow(dead_code)]
    _unconstructable: (),
}

impl TrafficGeneration for DummyCoverTrafficType {
    fn build(
        self,
    ) -> (
        impl futures::Stream<Item = DestinationRouting> + Send,
        impl futures::Sink<
            std::result::Result<hopr_transport::Telemetry, hopr_transport::ProbeError>,
            Error = impl std::error::Error,
        > + Send
        + Sync
        + Clone
        + 'static,
    ) {
        (
            futures::stream::empty(),
            futures::sink::drain::<std::result::Result<hopr_transport::Telemetry, hopr_transport::ProbeError>>(),
        )
    }
}

/// Prepare an optimized version of the tokio runtime setup for hopr-lib specifically.
///
/// Divide the available CPU parallelism by 2, since half of the available threads are
/// to be used for IO-bound and half for CPU-bound tasks.
#[cfg(feature = "runtime-tokio")]
pub fn prepare_tokio_runtime(
    num_cpu_threads: Option<NonZeroUsize>,
    num_io_threads: Option<NonZeroUsize>,
) -> anyhow::Result<tokio::runtime::Runtime> {
    use std::str::FromStr;
    let avail_parallelism = std::thread::available_parallelism().ok().map(|v| v.get() / 2);

    hopr_parallelize::cpu::init_thread_pool(
        num_cpu_threads
            .map(|v| v.get())
            .or(avail_parallelism)
            .ok_or(anyhow::anyhow!(
                "Could not determine the number of CPU threads to use. Please set the HOPRD_NUM_CPU_THREADS \
                 environment variable."
            ))?
            .max(1),
    )?;

    Ok(tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(
            num_io_threads
                .map(|v| v.get())
                .or(avail_parallelism)
                .ok_or(anyhow::anyhow!(
                    "Could not determine the number of IO threads to use. Please set the HOPRD_NUM_IO_THREADS \
                     environment variable."
                ))?
                .max(1),
        )
        .thread_name("hoprd")
        .thread_stack_size(
            std::env::var("HOPRD_THREAD_STACK_SIZE")
                .ok()
                .and_then(|v| usize::from_str(&v).ok())
                .unwrap_or(10 * 1024 * 1024)
                .max(2 * 1024 * 1024),
        )
        .build()?)
}

/// Type alias used to send and receive transport data via a running HOPR node.
pub type HoprTransportIO = socket::HoprSocket<
    futures::channel::mpsc::Receiver<ApplicationDataIn>,
    futures::channel::mpsc::Sender<(DestinationRouting, ApplicationDataOut)>,
>;

/// HOPR main object providing the entire HOPR node functionality
///
/// Instantiating this object creates all processes and objects necessary for
/// running the HOPR node. Once created, the node can be started using the
/// `run()` method.
///
/// Externally offered API should be enough to perform all necessary tasks
/// with the HOPR node manually, but it is advised to create such a configuration
/// that manual interaction is unnecessary.
///
/// As such, the `hopr_lib` serves mainly as an integration point into Rust programs.
pub struct Hopr<Chain, Db> {
    me: OffchainKeypair,
    cfg: config::HoprLibConfig,
    state: Arc<state::AtomicHoprState>,
    transport_api: HoprTransport<Db, Chain>,
    redeem_requests: OnceLock<futures::channel::mpsc::Sender<TicketSelector>>,
    node_db: Db,
    chain_api: Chain,
    processes: OnceLock<AbortableList<HoprLibProcess>>,
}

impl<Chain, Db> Hopr<Chain, Db>
where
    Chain: HoprChainApi + Clone + Send + Sync + 'static,
    Db: HoprNodeDbApi + Clone + Send + Sync + 'static,
{
    pub async fn new(
        cfg: config::HoprLibConfig,
        hopr_chain_api: Chain,
        hopr_node_db: Db,
        me: &OffchainKeypair,
        me_onchain: &ChainKeypair,
    ) -> errors::Result<Self> {
        if hopr_crypto_random::is_rng_fixed() {
            warn!("!! FOR TESTING ONLY !! THIS BUILD IS USING AN INSECURE FIXED RNG !!")
        }

        let multiaddress: Multiaddr = (&cfg.host).try_into().map_err(HoprLibError::TransportError)?;

        let my_multiaddresses = vec![multiaddress];

        let hopr_transport_api = HoprTransport::new(
            me,
            me_onchain,
            HoprTransportConfig {
                transport: cfg.transport.clone(),
                network: cfg.network_options.clone(),
                protocol: cfg.protocol,
                probe: cfg.probe,
                session: cfg.session,
            },
            hopr_node_db.clone(),
            hopr_chain_api.clone(),
            my_multiaddresses,
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            METRIC_PROCESS_START_TIME.set(current_time().as_unix_timestamp().as_secs_f64());
            METRIC_HOPR_LIB_VERSION.set(
                &[const_format::formatcp!("{}", constants::APP_VERSION)],
                const_format::formatcp!(
                    "{}.{}",
                    env!("CARGO_PKG_VERSION_MAJOR"),
                    env!("CARGO_PKG_VERSION_MINOR")
                )
                .parse()
                .unwrap_or(0.0),
            );

            // Calling get_ticket_statistics will initialize the respective metrics on tickets
            if let Err(error) = hopr_node_db.get_ticket_statistics(None).await {
                error!(%error, "failed to initialize ticket statistics metrics");
            }
        }

        Ok(Self {
            me: me.clone(),
            cfg,
            state: Arc::new(state::AtomicHoprState::new(HoprState::Uninitialized)),
            transport_api: hopr_transport_api,
            chain_api: hopr_chain_api,
            node_db: hopr_node_db,
            redeem_requests: OnceLock::new(),
            processes: OnceLock::new(),
        })
    }

    fn error_if_not_in_state(&self, state: HoprState, error: String) -> errors::Result<()> {
        if self.status() == state {
            Ok(())
        } else {
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(state, error)))
        }
    }

    pub fn status(&self) -> HoprState {
        self.state.load(Ordering::Relaxed)
    }

    pub async fn get_balance<C: Currency + Send>(&self) -> errors::Result<Balance<C>> {
        self.chain_api
            .get_balance(self.me_onchain())
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn get_safe_balance<C: Currency + Send>(&self) -> errors::Result<Balance<C>> {
        self.chain_api
            .get_balance(self.cfg.safe_module.safe_address)
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn chain_info(&self) -> errors::Result<ChainInfo> {
        self.chain_api.chain_info().await.map_err(HoprLibError::chain)
    }

    pub fn get_safe_config(&self) -> SafeModule {
        self.cfg.safe_module.clone()
    }

    pub fn config(&self) -> &config::HoprLibConfig {
        &self.cfg
    }

    #[inline]
    fn is_public(&self) -> bool {
        self.cfg.publish
    }

    pub async fn run<
        Ct,
        #[cfg(feature = "session-server")] T: traits::session::HoprSessionServer + Clone + Send + 'static,
    >(
        &self,
        cover_traffic: Option<Ct>,
        #[cfg(feature = "session-server")] serve_handler: T,
    ) -> errors::Result<HoprTransportIO>
    where
        Ct: TrafficGeneration + Send + Sync + 'static,
    {
        self.error_if_not_in_state(
            HoprState::Uninitialized,
            "cannot start the hopr node multiple times".into(),
        )?;

        #[cfg(feature = "testing")]
        warn!("!! FOR TESTING ONLY !! Node is running with some safety checks disabled!");

        info!(
            address = %self.me_onchain(), minimum_balance = %*SUGGESTED_NATIVE_BALANCE,
            "node is not started, please fund this node",
        );

        helpers::wait_for_funds(
            *MIN_NATIVE_BALANCE,
            *SUGGESTED_NATIVE_BALANCE,
            Duration::from_secs(200),
            self.me_onchain(),
            &self.chain_api,
        )
        .await?;

        let mut processes = AbortableList::<HoprLibProcess>::default();

        info!("starting HOPR node...");
        self.state.store(HoprState::Initializing, Ordering::Relaxed);

        let balance: XDaiBalance = self.get_balance().await?;
        let minimum_balance = *constants::MIN_NATIVE_BALANCE;

        info!(
            address = %self.me_onchain(),
            %balance,
            %minimum_balance,
            "node information"
        );

        if balance.le(&minimum_balance) {
            return Err(HoprLibError::GeneralError(
                "cannot start the node without a sufficiently funded wallet".into(),
            ));
        }

        // Once we are able to query the chain,
        // check if the ticket price is configured correctly.
        let network_min_ticket_price = self
            .chain_api
            .minimum_ticket_price()
            .await
            .map_err(HoprLibError::chain)?;
        let configured_ticket_price = self.cfg.protocol.outgoing_ticket_price;
        if configured_ticket_price.is_some_and(|c| c < network_min_ticket_price) {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing ticket price is lower than the network minimum ticket price: \
                 {configured_ticket_price:?} < {network_min_ticket_price}"
            )));
        }
        // Once we are able to query the chain,
        // check if the winning probability is configured correctly.
        let network_min_win_prob = self
            .chain_api
            .minimum_incoming_ticket_win_prob()
            .await
            .map_err(HoprLibError::chain)?;
        let configured_win_prob = self.cfg.protocol.outgoing_ticket_winning_prob;
        if !std::env::var("HOPR_TEST_DISABLE_CHECKS").is_ok_and(|v| v.to_lowercase() == "true")
            && configured_win_prob
                .and_then(|c| WinningProbability::try_from(c).ok())
                .is_some_and(|c| c.approx_cmp(&network_min_win_prob).is_lt())
        {
            return Err(HoprLibError::GeneralError(format!(
                "configured outgoing ticket winning probability is lower than the network minimum winning \
                 probability: {configured_win_prob:?} < {network_min_win_prob}"
            )));
        }

        // Calculate the minimum capacity based on accounts (each account can generate 2 messages),
        // plus 100 as an additional buffer
        let minimum_capacity = self
            .chain_api
            .count_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .await
            .map_err(HoprLibError::chain)?
            .saturating_mul(2)
            .saturating_add(100);

        let chain_discovery_events_capacity = std::env::var("HOPR_INTERNAL_CHAIN_DISCOVERY_CHANNEL_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(2048)
            .max(minimum_capacity);

        debug!(
            capacity = chain_discovery_events_capacity,
            minimum_required = minimum_capacity,
            "creating chain discovery events channel"
        );
        let (indexer_peer_update_tx, indexer_peer_update_rx) =
            channel::<PeerDiscovery>(chain_discovery_events_capacity);

        // Stream all the existing announcements and also subscribe to all future on-chain
        // announcements
        let (announcements_stream, announcements_handle) = futures::stream::abortable(
            self.chain_api
                .subscribe_with_state_sync([StateSyncOptions::PublicAccounts])
                .map_err(HoprLibError::chain)?,
        );
        processes.insert(HoprLibProcess::AccountAnnouncements, announcements_handle);

        spawn(
            announcements_stream
                .filter_map(|event| {
                    futures::future::ready(event.try_as_announcement().map(|account| {
                        PeerDiscovery::Announce(account.public_key.into(), account.get_multiaddrs().to_vec())
                    }))
                })
                .map(Ok)
                .forward(indexer_peer_update_tx)
                .inspect(
                    |_| warn!(task = %HoprLibProcess::AccountAnnouncements,"long-running background task finished"),
                ),
        );

        info!(peer_id = %self.me_peer_id(), address = %self.me_onchain(), version = constants::APP_VERSION, "Node information");

        let safe_addr = self.cfg.safe_module.safe_address;

        if self.me_onchain() == safe_addr {
            return Err(HoprLibError::GeneralError("cannot self as staking safe address".into()));
        }

        info!(%safe_addr, "registering safe with this node");
        match self.chain_api.register_safe(&safe_addr).await {
            Ok(awaiter) => {
                // Wait until the registration is confirmed on-chain, otherwise we cannot proceed.
                awaiter.await.map_err(HoprLibError::chain)?;
                info!(%safe_addr, "safe successfully registered with this node");
            }
            Err(SafeRegistrationError::AlreadyRegistered(registered_safe)) if registered_safe == safe_addr => {
                info!(%safe_addr, "this safe is already registered with this node");
            }
            Err(error) => {
                error!(%safe_addr, %error, "safe registration failed");
                return Err(HoprLibError::chain(error));
            }
        }

        // Only public nodes announce multiaddresses
        let multiaddresses_to_announce = if self.is_public() {
            // The multiaddresses are filtered for the non-private ones,
            // unless `announce_local_addresses` is set to `true`.
            self.transport_api.announceable_multiaddresses()
        } else {
            Vec::with_capacity(0)
        };

        // Warn when announcing a private multiaddress, which is acceptable in certain scenarios
        multiaddresses_to_announce
            .iter()
            .filter(|a| !is_public_address(a))
            .for_each(|multi_addr| tracing::warn!(?multi_addr, "announcing private multiaddress"));

        // At this point the node is already registered with Safe, so
        // we can announce via Safe-compliant TX
        info!(?multiaddresses_to_announce, "announcing node on chain");
        match self.chain_api.announce(&multiaddresses_to_announce, &self.me).await {
            Ok(awaiter) => {
                // Wait until the announcement is confirmed on-chain, otherwise we cannot proceed.
                awaiter.await.map_err(HoprLibError::chain)?;
                info!(?multiaddresses_to_announce, "node has been successfully announced");
            }
            Err(AnnouncementError::AlreadyAnnounced) => {
                info!(multiaddresses_announced = ?multiaddresses_to_announce, "node already announced on chain")
            }
            Err(error) => {
                error!(%error, ?multiaddresses_to_announce, "failed to transmit node announcement");
                return Err(HoprLibError::chain(error));
            }
        }

        let incoming_session_channel_capacity = std::env::var("HOPR_INTERNAL_SESSION_INCOMING_CAPACITY")
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(256);

        let (session_tx, _session_rx) = channel::<IncomingSession>(incoming_session_channel_capacity);
        #[cfg(feature = "session-server")]
        {
            debug!(capacity = incoming_session_channel_capacity, "creating session server");
            processes.insert(
                HoprLibProcess::SessionServer,
                hopr_async_runtime::spawn_as_abortable!(
                    _session_rx
                        .for_each_concurrent(None, move |session| {
                            let serve_handler = serve_handler.clone();
                            async move {
                                let session_id = *session.session.id();
                                match serve_handler.process(session).await {
                                    Ok(_) => debug!(?session_id, "client session processed successfully"),
                                    Err(error) => error!(
                                        ?session_id,
                                        %error,
                                        "client session processing failed"
                                    ),
                                }
                            }
                        })
                        .inspect(|_| tracing::warn!(
                            task = %HoprLibProcess::SessionServer,
                            "long-running background task finished"
                        ))
                ),
            );
        }

        info!("starting transport");

        let (hopr_socket, transport_processes) = self
            .transport_api
            .run(cover_traffic, indexer_peer_update_rx, session_tx)
            .await?;
        processes.flat_map_extend_from(transport_processes, HoprLibProcess::Transport);

        info!("starting outgoing ticket flush process");
        let (index_flush_stream, index_flush_handle) =
            futures::stream::abortable(futures_time::stream::interval(Duration::from_secs(5).into()));
        processes.insert(HoprLibProcess::TicketIndexFlush, index_flush_handle);
        let node_db = self.node_db.clone();
        spawn(
            index_flush_stream
                .for_each(move |_| {
                    let node_db = node_db.clone();
                    async move {
                        match node_db.persist_outgoing_ticket_indices().await {
                            Ok(count) => trace!(count, "successfully flushed states of outgoing ticket indices"),
                            Err(error) => error!(%error, "Failed to flush ticket indices"),
                        }
                    }
                })
                .inspect(|_| {
                    tracing::warn!(
                        task = %HoprLibProcess::TicketIndexFlush,
                        "long-running background task finished"
                    )
                }),
        );

        // Start a queue that takes care of redeeming tickets via given TicketSelectors
        let (redemption_req_tx, redemption_req_rx) = channel::<TicketSelector>(1024);
        let _ = self.redeem_requests.set(redemption_req_tx);
        let (redemption_req_rx, redemption_req_handle) = futures::stream::abortable(redemption_req_rx);
        processes.insert(HoprLibProcess::TicketRedemptions, redemption_req_handle);
        let chain = self.chain_api.clone();
        let node_db = self.node_db.clone();
        spawn(redemption_req_rx
            .for_each(move |selector| {
                let chain = chain.clone();
                let db = node_db.clone();
                async move {
                    match chain.redeem_tickets_via_selectors(&db, [selector]).await {
                        Ok(res) => debug!(%res, "redemption complete"),
                        Err(error) => error!(%error, "redemption failed"),
                    }
                }
            })
            .inspect(|_| tracing::warn!(task = %HoprLibProcess::TicketRedemptions, "long-running background task finished"))
        );

        let (chain_events_sub_handle, chain_events_sub_reg) = hopr_async_runtime::AbortHandle::new_pair();
        processes.insert(HoprLibProcess::ChannelEvents, chain_events_sub_handle);
        let chain = self.chain_api.clone();
        let node_db = self.node_db.clone();
        let events = chain.subscribe().map_err(HoprLibError::chain)?;
        spawn(
            futures::stream::Abortable::new(
                events
                    .filter_map(move |event|
                        futures::future::ready(event.try_as_channel_closed())
                    ),
                chain_events_sub_reg
            )
            .for_each(move |closed_channel| {
                let node_db = node_db.clone();
                let chain = chain.clone();
                async move {
                    match closed_channel.direction(chain.me()) {
                        Some(ChannelDirection::Incoming) => {
                            match node_db.mark_tickets_as([&closed_channel], TicketMarker::Neglected).await {
                                Ok(num_neglected) if num_neglected > 0 => {
                                    warn!(%num_neglected, %closed_channel, "tickets on incoming closed channel were neglected");
                                },
                                Ok(_) => {
                                    debug!(%closed_channel, "no neglected tickets on incoming closed channel");
                                },
                                Err(error) => {
                                    error!(%error, %closed_channel, "failed to mark tickets on incoming closed channel as neglected");
                                }
                            }
                        },
                        Some(ChannelDirection::Outgoing) => {
                            if let Err(error) = node_db.reset_outgoing_ticket_index(closed_channel.get_id()).await {
                                error!(%error, %closed_channel, "failed to reset ticket index on closed outgoing channel");
                            } else {
                                debug!(%closed_channel, "outgoing ticket index has been resets on outgoing channel closure");
                            }
                        }
                        _ => {} // Event for a channel that is not our own
                    }
                }
            })
            .inspect(|_| tracing::warn!(task = %HoprLibProcess::ChannelEvents, "long-running background task finished"))
        );

        // NOTE: after the chain is synced, we can reset tickets which are considered
        // redeemed but on-chain state does not align with that. This implies there was a problem
        // right when the transaction was sent on-chain. In such cases, we simply let it retry and
        // handle errors appropriately.
        let mut channels = self
            .chain_api
            .stream_channels(ChannelSelector {
                destination: self.me_onchain().into(),
                ..Default::default()
            })
            .map_err(HoprLibError::chain)
            .await?;

        while let Some(channel) = channels.next().await {
            self.node_db
                .update_ticket_states_and_fetch(
                    [TicketSelector::from(&channel)
                        .with_state(AcknowledgedTicketStatus::BeingRedeemed)
                        .with_index_range(channel.ticket_index.as_u64()..)],
                    AcknowledgedTicketStatus::Untouched,
                )
                .map_err(HoprLibError::db)
                .await?
                .for_each(|ticket| {
                    info!(%ticket, "fixed next out-of-sync ticket");
                    futures::future::ready(())
                })
                .await;
        }

        self.state.store(HoprState::Running, Ordering::Relaxed);

        info!(
            id = %self.me_peer_id(),
            version = constants::APP_VERSION,
            "NODE STARTED AND RUNNING"
        );

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_HOPR_NODE_INFO.set(
            &[
                &self.me.public().to_peerid_str(),
                &self.me_onchain().to_string(),
                &self.cfg.safe_module.safe_address.to_string(),
                &self.cfg.safe_module.module_address.to_string(),
            ],
            1.0,
        );

        let _ = self.processes.set(processes);
        Ok(hopr_socket)
    }

    /// Used to practically shut down all node's processes without dropping the instance.
    ///
    /// This means that the instance can be used to retrieve some information, but all
    /// active operations will stop and new will be impossible to perform.
    /// Such operations will return [`HoprStatusError::NotThereYet`].
    ///
    /// This is the final state and cannot be reversed by calling [`HoprLib::run`] again.
    pub fn shutdown(&self) -> Result<(), HoprLibError> {
        self.error_if_not_in_state(HoprState::Running, "node is not running".into())?;
        if let Some(processes) = self.processes.get() {
            processes.abort_all();
        }
        self.state.store(HoprState::Terminated, Ordering::Relaxed);
        info!("NODE SHUTDOWN COMPLETE");
        Ok(())
    }

    // p2p transport =========
    /// Own PeerId used in the libp2p transport layer
    pub fn me_peer_id(&self) -> PeerId {
        (*self.me.public()).into()
    }

    /// Get the list of all announced public nodes in the network
    pub async fn get_public_nodes(&self) -> errors::Result<Vec<(PeerId, Address, Vec<Multiaddr>)>> {
        Ok(self
            .chain_api
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .map_err(HoprLibError::chain)
            .await?
            .map(|entry| {
                (
                    PeerId::from(entry.public_key),
                    entry.chain_addr,
                    entry.get_multiaddrs().to_vec(),
                )
            })
            .collect()
            .await)
    }

    /// Ping another node in the network based on the PeerId
    ///
    /// Returns the RTT (round trip time), i.e. how long it took for the ping to return.
    pub async fn ping(&self, peer: &PeerId) -> errors::Result<(std::time::Duration, PeerStatus)> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        Ok(self.transport_api.ping(peer).await?)
    }

    /// Create a client session connection returning a session object that implements
    /// [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`] and can bu used as a read/write binary session.
    #[cfg(feature = "session-client")]
    pub async fn connect_to(
        &self,
        destination: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
    ) -> errors::Result<HoprSession> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let backoff = backon::ConstantBuilder::default()
            .with_max_times(self.cfg.session.establish_max_retries as usize)
            .with_delay(self.cfg.session.establish_retry_timeout)
            .with_jitter();

        use backon::Retryable;

        Ok((|| {
            let cfg = cfg.clone();
            let target = target.clone();
            async { self.transport_api.new_session(destination, target, cfg).await }
        })
        .retry(backoff)
        .sleep(backon::FuturesTimerSleeper)
        .await?)
    }

    /// Sends keep-alive to the given [`HoprSessionId`], making sure the session is not
    /// closed due to inactivity.
    #[cfg(feature = "session-client")]
    pub async fn keep_alive_session(&self, id: &SessionId) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;
        Ok(self.transport_api.probe_session(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn get_session_surb_balancer_config(&self, id: &SessionId) -> errors::Result<Option<SurbBalancerConfig>> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;
        Ok(self.transport_api.session_surb_balancing_cfg(id).await?)
    }

    #[cfg(feature = "session-client")]
    pub async fn update_session_surb_balancer_config(
        &self,
        id: &SessionId,
        cfg: SurbBalancerConfig,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;
        Ok(self.transport_api.update_session_surb_balancing_cfg(id, cfg).await?)
    }

    /// List all multiaddresses announced by this node
    pub fn local_multiaddresses(&self) -> Vec<Multiaddr> {
        self.transport_api.local_multiaddresses()
    }

    /// List all multiaddresses on which the node is listening
    pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
        self.transport_api.listening_multiaddresses().await
    }

    /// List all multiaddresses observed for a PeerId
    pub async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
        self.transport_api.network_observed_multiaddresses(peer).await
    }

    /// List all multiaddresses announced on-chain for the given node.
    pub async fn multiaddresses_announced_on_chain(&self, peer: &PeerId) -> errors::Result<Vec<Multiaddr>> {
        let peer = *peer;
        // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
        let pubkey = hopr_parallelize::cpu::spawn_blocking(move || OffchainPublicKey::from_peerid(&peer)).await?;

        match self
            .chain_api
            .stream_accounts(AccountSelector {
                public_only: false,
                offchain_key: Some(pubkey),
                ..Default::default()
            })
            .map_err(HoprLibError::chain)
            .await?
            .next()
            .await
        {
            Some(entry) => Ok(entry.get_multiaddrs().to_vec()),
            None => {
                error!(%peer, "no information");
                Ok(vec![])
            }
        }
    }

    // Network =========

    /// Get measured network health
    pub async fn network_health(&self) -> Health {
        self.transport_api.network_health().await
    }

    /// List all peers connected to this
    pub async fn network_connected_peers(&self) -> errors::Result<Vec<PeerId>> {
        Ok(self.transport_api.network_connected_peers().await?)
    }

    /// Get all data collected from the network relevant for a PeerId
    pub async fn network_peer_info(&self, peer: &PeerId) -> errors::Result<Option<PeerStatus>> {
        Ok(self.transport_api.network_peer_info(peer).await?)
    }

    /// Get peers connected peers with quality higher than some value
    pub async fn all_network_peers(
        &self,
        minimum_quality: f64,
    ) -> errors::Result<Vec<(Option<Address>, PeerId, PeerStatus)>> {
        Ok(
            futures::stream::iter(self.transport_api.network_connected_peers().await?)
                .filter_map(|peer| async move {
                    if let Ok(Some(info)) = self.transport_api.network_peer_info(&peer).await {
                        if info.get_average_quality() >= minimum_quality {
                            Some((peer, info))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .filter_map(|(peer_id, info)| async move {
                    let address = self.peerid_to_chain_key(&peer_id).await.ok().flatten();
                    Some((address, peer_id, info))
                })
                .collect::<Vec<_>>()
                .await,
        )
    }

    // Ticket ========
    /// Get all tickets in a channel specified by [`prelude::Hash`]
    pub async fn tickets_in_channel(&self, channel: &prelude::Hash) -> errors::Result<Option<Vec<RedeemableTicket>>> {
        Ok(self.transport_api.tickets_in_channel(channel).await?)
    }

    /// Get all tickets
    pub async fn all_tickets(&self) -> errors::Result<Vec<Ticket>> {
        Ok(self.transport_api.all_tickets().await?)
    }

    /// Get statistics for all tickets
    pub async fn ticket_statistics(&self) -> errors::Result<TicketStatistics> {
        Ok(self.transport_api.ticket_statistics().await?)
    }

    /// Reset the ticket metrics to zero
    pub async fn reset_ticket_statistics(&self) -> errors::Result<()> {
        self.node_db
            .reset_ticket_statistics()
            .await
            .map_err(HoprLibError::chain)
    }

    // Chain =========
    pub fn me_onchain(&self) -> Address {
        *self.chain_api.me()
    }

    /// Get ticket price
    pub async fn get_ticket_price(&self) -> errors::Result<HoprBalance> {
        self.chain_api.minimum_ticket_price().await.map_err(HoprLibError::chain)
    }

    /// Get minimum incoming ticket winning probability
    pub async fn get_minimum_incoming_ticket_win_probability(&self) -> errors::Result<WinningProbability> {
        self.chain_api
            .minimum_incoming_ticket_win_prob()
            .await
            .map_err(HoprLibError::chain)
    }

    /// List of all accounts announced on the chain
    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self
            .chain_api
            .stream_accounts(AccountSelector {
                public_only: true,
                ..Default::default()
            })
            .map_err(HoprLibError::chain)
            .await?
            .collect()
            .await)
    }

    /// Get the channel entry from Hash.
    /// @returns the channel entry of those two nodes
    pub async fn channel_from_hash(&self, channel_id: &Hash) -> errors::Result<Option<ChannelEntry>> {
        self.chain_api
            .channel_by_id(channel_id)
            .await
            .map_err(HoprLibError::chain)
    }

    /// Get the channel entry between source and destination node.
    /// @param src Address
    /// @param dest Address
    /// @returns the channel entry of those two nodes
    pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<Option<ChannelEntry>> {
        self.chain_api
            .channel_by_parties(src, dest)
            .await
            .map_err(HoprLibError::chain)
    }

    /// List all channels open from a specified Address
    pub async fn channels_from(&self, src: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self
            .chain_api
            .stream_channels(ChannelSelector::default().with_source(*src).with_allowed_states(&[
                ChannelStatusDiscriminants::Closed,
                ChannelStatusDiscriminants::Open,
                ChannelStatusDiscriminants::PendingToClose,
            ]))
            .map_err(HoprLibError::chain)
            .await?
            .collect()
            .await)
    }

    /// List all channels open to a specified address
    pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self
            .chain_api
            .stream_channels(
                ChannelSelector::default()
                    .with_destination(*dest)
                    .with_allowed_states(&[
                        ChannelStatusDiscriminants::Closed,
                        ChannelStatusDiscriminants::Open,
                        ChannelStatusDiscriminants::PendingToClose,
                    ]),
            )
            .map_err(HoprLibError::chain)
            .await?
            .collect()
            .await)
    }

    /// List all channels
    pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self
            .chain_api
            .stream_channels(ChannelSelector::default().with_allowed_states(&[
                ChannelStatusDiscriminants::Closed,
                ChannelStatusDiscriminants::Open,
                ChannelStatusDiscriminants::PendingToClose,
            ]))
            .map_err(HoprLibError::chain)
            .await?
            .collect()
            .await)
    }

    /// Current safe allowance balance
    pub async fn safe_allowance(&self) -> errors::Result<HoprBalance> {
        self.chain_api
            .safe_allowance(self.cfg.safe_module.safe_address)
            .await
            .map_err(HoprLibError::chain)
    }

    /// Withdraw on-chain assets to a given address
    /// @param recipient the account where the assets should be transferred to
    /// @param amount how many tokens to be transferred
    pub async fn withdraw_tokens(&self, recipient: Address, amount: HoprBalance) -> errors::Result<prelude::Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.chain_api
            .withdraw(amount, &recipient)
            .and_then(identity)
            .map_err(HoprLibError::chain)
            .await
    }

    /// Withdraw on-chain native assets to a given address
    /// @param recipient the account where the assets should be transferred to
    /// @param amount how many tokens to be transferred
    pub async fn withdraw_native(&self, recipient: Address, amount: XDaiBalance) -> errors::Result<prelude::Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.chain_api
            .withdraw(amount, &recipient)
            .and_then(identity)
            .map_err(HoprLibError::chain)
            .await
    }

    pub async fn open_channel(&self, destination: &Address, amount: HoprBalance) -> errors::Result<OpenChannelResult> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let (channel_id, tx_hash) = self
            .chain_api
            .open_channel(destination, amount)
            .and_then(identity)
            .map_err(HoprLibError::chain)
            .await?;

        Ok(OpenChannelResult { tx_hash, channel_id })
    }

    pub async fn fund_channel(&self, channel_id: &prelude::Hash, amount: HoprBalance) -> errors::Result<prelude::Hash> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.chain_api
            .fund_channel(channel_id, amount)
            .and_then(identity)
            .map_err(HoprLibError::chain)
            .await
    }

    pub async fn close_channel_by_id(&self, channel_id: &ChannelId) -> errors::Result<CloseChannelResult> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let tx_hash = self
            .chain_api
            .close_channel(channel_id)
            .and_then(identity)
            .map_err(HoprLibError::chain)
            .await?;

        Ok(CloseChannelResult { tx_hash })
    }

    pub async fn get_channel_closure_notice_period(&self) -> errors::Result<Duration> {
        self.chain_api
            .channel_closure_notice_period()
            .await
            .map_err(HoprLibError::chain)
    }

    pub fn redemption_requests(
        &self,
    ) -> errors::Result<impl futures::Sink<TicketSelector, Error = HoprLibError> + Clone> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        // TODO: add universal timeout sink here
        Ok(self
            .redeem_requests
            .get()
            .cloned()
            .expect("redeem_requests is not initialized")
            .sink_map_err(HoprLibError::other))
    }

    pub async fn redeem_all_tickets<B: Into<HoprBalance>>(&self, min_value: B) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let min_value = min_value.into();

        self.chain_api
            .stream_channels(
                ChannelSelector::default()
                    .with_destination(self.me_onchain())
                    .with_allowed_states(&[
                        ChannelStatusDiscriminants::Open,
                        ChannelStatusDiscriminants::PendingToClose,
                    ]),
            )
            .map_err(HoprLibError::chain)
            .await?
            .map(|channel| {
                Ok(TicketSelector::from(&channel)
                    .with_amount(min_value..)
                    .with_index_range(channel.ticket_index.as_u64()..)
                    .with_state(AcknowledgedTicketStatus::Untouched))
            })
            .forward(self.redemption_requests()?)
            .await?;

        Ok(())
    }

    pub async fn redeem_tickets_with_counterparty<B: Into<HoprBalance>>(
        &self,
        counterparty: &Address,
        min_value: B,
    ) -> errors::Result<()> {
        self.redeem_tickets_in_channel(&generate_channel_id(counterparty, &self.me_onchain()), min_value)
            .await
    }

    pub async fn redeem_tickets_in_channel<B: Into<HoprBalance>>(
        &self,
        channel_id: &Hash,
        min_value: B,
    ) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        let channel = self
            .chain_api
            .channel_by_id(channel_id)
            .await
            .map_err(HoprLibError::chain)?
            .ok_or(HoprLibError::GeneralError("Channel not found".into()))?;

        self.redemption_requests()?
            .send(
                TicketSelector::from(channel)
                    .with_amount(min_value.into()..)
                    .with_index_range(channel.ticket_index.as_u64()..)
                    .with_state(AcknowledgedTicketStatus::Untouched),
            )
            .await?;

        Ok(())
    }

    pub async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> errors::Result<()> {
        self.error_if_not_in_state(HoprState::Running, "Node is not ready for on-chain operations".into())?;

        self.redemption_requests()?
            .send(TicketSelector::from(&ack_ticket).with_state(AcknowledgedTicketStatus::Untouched))
            .await?;

        Ok(())
    }

    pub async fn peerid_to_chain_key(&self, peer_id: &PeerId) -> errors::Result<Option<Address>> {
        let peer_id = *peer_id;
        // PeerId -> OffchainPublicKey is a CPU-intensive blocking operation
        let pubkey = hopr_parallelize::cpu::spawn_blocking(move || prelude::OffchainPublicKey::from_peerid(&peer_id))
            .await
            .map_err(|e| HoprLibError::GeneralError(format!("failed to convert peer id to off-chain key: {}", e)))?;

        self.chain_api
            .packet_key_to_chain_key(&pubkey)
            .await
            .map_err(HoprLibError::chain)
    }

    pub async fn chain_key_to_peerid(&self, address: &Address) -> errors::Result<Option<PeerId>> {
        self.chain_api
            .chain_key_to_packet_key(address)
            .await
            .map(|pk| pk.map(|v| v.into()))
            .map_err(HoprLibError::chain)
    }
}

impl<Chain, Db> Hopr<Chain, Db> {
    // === telemetry
    /// Prometheus formatted metrics collected by the hopr-lib components.
    pub fn collect_hopr_metrics() -> errors::Result<String> {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "prometheus", not(test)))] {
                hopr_metrics::gather_all_metrics().map_err(|e| HoprLibError::Other(e.into()))
            } else {
                Err(HoprLibError::GeneralError("BUILT WITHOUT METRICS SUPPORT".into()))
            }
        }
    }
}
