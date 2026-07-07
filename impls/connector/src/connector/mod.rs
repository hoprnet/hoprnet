use std::{cmp::Ordering, str::FromStr, sync::atomic::Ordering as AtomicOrdering, time::Duration};

use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use futures_time::future::FutureExt as FuturesTimeExt;
use hopr_api::{
    chain::{ChainPathResolver, ChainReceipt, HoprKeyIdent},
    types::{chain::prelude::*, crypto::prelude::*, internal::prelude::*, primitive::prelude::*},
};
use hopr_utils::runtime::AbortHandle;
use petgraph::prelude::DiGraphMap;

use crate::{
    backend::Backend,
    connector::{keys::HoprKeyMapper, sequencer::TransactionSequencer, values::CHAIN_INFO_CACHE_KEY},
    errors::ConnectorError,
    utils::{
        ParsedChainInfo, model_to_account_entry, model_to_graph_entry, model_to_ticket_params,
        process_channel_changes_into_events,
    },
};

mod accounts;
mod channels;
mod events;
mod keys;
mod safe;
mod sequencer;
mod tickets;
mod values;

type EventsChannel = (
    async_broadcast::Sender<ChainEvent>,
    async_broadcast::InactiveReceiver<ChainEvent>,
);

const MIN_CONNECTION_TIMEOUT: Duration = Duration::from_millis(100);
const MIN_TX_CONFIRM_TIMEOUT: Duration = Duration::from_secs(1);
const TX_TIMEOUT_MULTIPLIER: u32 = 2;
const DEFAULT_SYNC_TOLERANCE_PCT: usize = 90;

/// Connector health states.
///
/// Each value maps to a `ComponentStatus` variant plus a fixed detail message.
/// Storage and updates are lock-free via [`AtomicChainHealthState`]; reads
/// convert to `ComponentStatus` which allocates for non-`Ready` variants.
#[atomic_enum::atomic_enum]
#[derive(PartialEq, Eq)]
enum ChainHealthState {
    Ready = 0,
    WaitingForConnection = 1,
    Connecting = 2,
    SubscriptionEnded = 3,
    SyncTimedOut = 4,
    ServerNotHealthy = 5,
    ConnectionFailed = 6,
    Dropped = 7,
}

impl From<ChainHealthState> for hopr_api::node::ComponentStatus {
    fn from(state: ChainHealthState) -> Self {
        match state {
            ChainHealthState::Ready => Self::Ready,
            ChainHealthState::WaitingForConnection => Self::Initializing("waiting for chain connection".into()),
            ChainHealthState::Connecting => Self::Initializing("connecting to blokli".into()),
            ChainHealthState::SubscriptionEnded => Self::Degraded("chain subscription ended".into()),
            ChainHealthState::SyncTimedOut => Self::Degraded("connection sync timed out".into()),
            ChainHealthState::ServerNotHealthy => Self::Unavailable("blokli server not healthy".into()),
            ChainHealthState::ConnectionFailed => Self::Unavailable("chain connection failed".into()),
            ChainHealthState::Dropped => Self::Unavailable("connector dropped".into()),
        }
    }
}

/// Configuration of the [`HoprBlockchainConnector`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault)]
pub struct BlockchainConnectorConfig {
    /// Maximum time to wait for [connection](HoprBlockchainConnector::connect) to complete.
    ///
    /// Default is 30 seconds, minimum is 100 milliseconds.
    #[default(Duration::from_secs(30))]
    pub connection_sync_timeout: Duration,
    /// Percentage of the total number of accounts and opened channels that must
    /// be received during a [connection attempt](HoprBlockchainConnector::connect)
    /// to be successful.
    ///
    /// Default is 90%, minimum is 1, maximum is 100.
    #[default(DEFAULT_SYNC_TOLERANCE_PCT)]
    pub sync_tolerance: usize,
    /// Transaction waits for confirmation by multiplying chain's blocktime, finality, and this multiplier.
    /// Set it to higher values if transactions are failing due to timeout at the client.
    ///
    /// Default is 2, minimum is 1.
    #[default(TX_TIMEOUT_MULTIPLIER)]
    pub tx_timeout_multiplier: u32,
}

/// A connector acting as middleware between the HOPR APIs (see the [`hopr_api`] crate) and the Blokli Client API (see
/// the [`blokli_client`] crate).
///
/// The connector object cannot be cloned and shall be used inside an `Arc` if cloning is needed.
pub struct HoprBlockchainConnector<C, B, P, R> {
    payload_generator: P,
    chain_key: ChainKeypair,
    client: std::sync::Arc<C>,
    graph: std::sync::Arc<parking_lot::RwLock<DiGraphMap<HoprKeyIdent, ChannelId, ahash::RandomState>>>,
    backend: std::sync::Arc<B>,
    connection_handle: Option<AbortHandle>,
    sequencer: TransactionSequencer<C, R>,
    events: EventsChannel,
    cfg: BlockchainConnectorConfig,
    health: std::sync::Arc<AtomicChainHealthState>,

    // KeyId <-> OffchainPublicKey mapping
    mapper: HoprKeyMapper<B>,
    // Fast retrieval of chain keys by address
    chain_to_packet: moka::sync::Cache<Address, Option<OffchainPublicKey>, ahash::RandomState>,
    // Fast retrieval of packet keys by chain key
    packet_to_chain: moka::sync::Cache<OffchainPublicKey, Option<Address>, ahash::RandomState>,
    // Fast retrieval of channel entries by id
    channel_by_id: moka::sync::Cache<ChannelId, Option<ChannelEntry>, ahash::RandomState>,
    // Fast retrieval of channel entries by parties
    channel_by_parties: moka::sync::Cache<ChannelParties, Option<ChannelEntry>, ahash::RandomState>,
    // Contains chain info (no TTL - kept fresh by subscription handler)
    values: moka::future::Cache<u32, ParsedChainInfo>,
    // Ticket values (winning probability, price), kept fresh by subscription handler
    // Set only when connected
    ticket_values: std::sync::Arc<parking_lot::RwLock<Option<(WinningProbability, HoprBalance)>>>,
}

const EXPECTED_NUM_NODES: usize = 10_000;
const EXPECTED_NUM_CHANNELS: usize = 100_000;

const DEFAULT_CACHE_TIMEOUT: Duration = Duration::from_mins(10);

impl<B, C, P> HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliSubscriptionClient + BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    /// Creates a new instance.
    pub fn new(
        chain_key: ChainKeypair,
        cfg: BlockchainConnectorConfig,
        client: C,
        backend: B,
        payload_generator: P,
    ) -> Self {
        let backend = std::sync::Arc::new(backend);
        let (mut events_tx, events_rx) = async_broadcast::broadcast(1024);
        events_tx.set_overflow(true);
        events_tx.set_await_active(false);

        let client = std::sync::Arc::new(client);
        Self {
            payload_generator,
            health: std::sync::Arc::new(AtomicChainHealthState::new(ChainHealthState::WaitingForConnection)),
            graph: std::sync::Arc::new(parking_lot::RwLock::new(DiGraphMap::with_capacity_and_hasher(
                EXPECTED_NUM_NODES,
                EXPECTED_NUM_CHANNELS,
                ahash::RandomState::default(),
            ))),
            backend: backend.clone(),
            connection_handle: None,
            sequencer: TransactionSequencer::new(chain_key.clone(), client.clone()),
            events: (events_tx, events_rx.deactivate()),
            client,
            chain_key,
            cfg,
            mapper: HoprKeyMapper {
                id_to_key: moka::sync::CacheBuilder::new(EXPECTED_NUM_NODES as u64)
                    .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                    .build_with_hasher(ahash::RandomState::default()),
                key_to_id: moka::sync::CacheBuilder::new(EXPECTED_NUM_NODES as u64)
                    .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                    .build_with_hasher(ahash::RandomState::default()),
                backend,
            },
            chain_to_packet: moka::sync::CacheBuilder::new(EXPECTED_NUM_NODES as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            packet_to_chain: moka::sync::CacheBuilder::new(EXPECTED_NUM_NODES as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            channel_by_id: moka::sync::CacheBuilder::new(EXPECTED_NUM_CHANNELS as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            channel_by_parties: moka::sync::CacheBuilder::new(EXPECTED_NUM_CHANNELS as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            // No TTL: kept fresh by the Blokli subscription handler
            values: moka::future::CacheBuilder::new(1).build(),
            ticket_values: Default::default(),
        }
    }

    async fn do_connect(&self, timeout: Duration) -> Result<AbortHandle, ConnectorError> {
        let sync_quota = self.cfg.sync_tolerance.clamp(1, 100) as f64 / 100.0;
        let min_accounts = (self
            .client
            .count_accounts(blokli_client::api::AccountSelector::Any)
            .await? as f64
            * sync_quota)
            .round() as u32;
        let min_channels = (self
            .client
            .query_channel_stats(blokli_client::api::ChannelSelector {
                filter: None,
                status: Some(blokli_client::api::types::ChannelStatus::Open),
                ..Default::default()
            })
            .await?
            .count as f64
            * sync_quota)
            .round() as u32;
        tracing::debug!(min_accounts, min_channels, "connection thresholds");

        let server_health = self.client.query_health().await?;
        if !server_health.eq_ignore_ascii_case("OK") {
            tracing::warn!(server_health, "blokli server not healthy");
            return Err(ConnectorError::ServerNotHealthy);
        }

        let (abort_handle, abort_reg) = AbortHandle::new_pair();

        let (connection_ready_tx, connection_ready_rx) = futures::channel::oneshot::channel();
        let mut connection_ready_tx = Some(connection_ready_tx);

        let client = self.client.clone();
        let mapper = self.mapper.clone();
        let backend = self.backend.clone();
        let graph = self.graph.clone();
        let event_tx = self.events.0.clone();
        let me = self.chain_key.public().to_address();
        let values_cache = self.values.clone();

        let chain_to_packet = self.chain_to_packet.clone();
        let packet_to_chain = self.packet_to_chain.clone();

        let channel_by_id = self.channel_by_id.clone();
        let channel_by_parties = self.channel_by_parties.clone();

        // Query chain info to populate the cache
        let initial_chain_values = self.query_cached_chain_info().await?;
        self.ticket_values
            .write()
            .replace((initial_chain_values.ticket_win_prob, initial_chain_values.ticket_price));

        #[allow(clippy::large_enum_variant)]
        #[derive(Debug)]
        enum SubscribedEventType {
            Account((AccountEntry, Option<AccountEntry>)),
            Channel((ChannelEntry, Option<Vec<ChannelChange>>)),
            WinningProbability((WinningProbability, Option<WinningProbability>)),
            TicketPrice((HoprBalance, Option<HoprBalance>)),
        }

        let ticket_values = self.ticket_values.clone();
        let health = self.health.clone();
        hopr_utils::runtime::prelude::spawn(async move {
            let sync_started = std::time::Instant::now();

            let connections = client
                .subscribe_accounts(blokli_client::api::AccountSelector::Any)
                .and_then(|accounts| Ok((accounts, client.subscribe_graph()?)))
                .and_then(|(accounts, channels)| Ok((accounts, channels, client.subscribe_ticket_params()?)));

            if let Err(error) = connections {
                if let Some(connection_ready_tx) = connection_ready_tx.take() {
                    let _ = connection_ready_tx.send(Err(error));
                }
                return;
            }

            let (account_stream, channel_stream, ticket_params_stream) = connections.unwrap();

            // Stream of Account events (Announcements)
            let graph_clone = graph.clone();
            let account_stream = account_stream
                .inspect_ok(|entry| tracing::trace!(?entry, "new account event"))
                .map_err(ConnectorError::from)
                .try_filter_map(|account| futures::future::ready(model_to_account_entry(account).map(Some)))
                .and_then(move |account| {
                    let graph = graph_clone.clone();
                    let mapper = mapper.clone();
                    let chain_to_packet = chain_to_packet.clone();
                    let packet_to_chain = packet_to_chain.clone();
                    hopr_utils::runtime::prelude::spawn_blocking(move || {
                        mapper.key_to_id.insert(account.public_key, Some(account.key_id));
                        mapper.id_to_key.insert(account.key_id, Some(account.public_key));
                        chain_to_packet.insert(account.chain_addr, Some(account.public_key));
                        packet_to_chain.insert(account.public_key, Some(account.chain_addr));
                        graph.write().add_node(account.key_id);
                        let old = mapper
                            .backend
                            .insert_account(account.clone())
                            .map_err(ConnectorError::backend)?;
                        if let Some(old_account) = &old {
                            if old_account.chain_addr != account.chain_addr {
                                chain_to_packet.invalidate(&old_account.chain_addr);
                            }
                            if old_account.public_key != account.public_key {
                                packet_to_chain.invalidate(&old_account.public_key);
                            }
                        }
                        Ok::<_, ConnectorError>((account, old))
                    })
                    .map(|result| {
                        result
                            .map_err(ConnectorError::backend)?
                            .map(SubscribedEventType::Account)
                    })
                })
                .fuse();

            // Stream of channel graph updates
            let channel_stream = channel_stream
                .map_err(ConnectorError::from)
                .inspect_ok(|entry| tracing::trace!(?entry, "new graph event"))
                .try_filter_map(|graph_event| futures::future::ready(model_to_graph_entry(graph_event).map(Some)))
                .and_then(move |(src, dst, channel)| {
                    let graph = graph.clone();
                    let backend = backend.clone();
                    let channel_by_id = channel_by_id.clone();
                    let channel_by_parties = channel_by_parties.clone();
                    hopr_utils::runtime::prelude::spawn_blocking(move || {
                        graph.write().add_edge(src.key_id, dst.key_id, *channel.get_id());
                        backend
                            .insert_channel(channel)
                            .map(|old| (channel, old.map(|old| old.diff(&channel))))
                    })
                    .map_err(ConnectorError::backend)
                    .and_then(move |res| {
                        let channel_by_id = channel_by_id.clone();
                        let channel_by_parties = channel_by_parties.clone();
                        if let Ok((upserted_channel, _)) = &res {
                            // Rather update the cached entry than invalidating it
                            channel_by_id.insert(*upserted_channel.get_id(), Some(*upserted_channel));
                            channel_by_parties.insert(ChannelParties::from(upserted_channel), Some(*upserted_channel));
                        }
                        futures::future::ready(res.map(SubscribedEventType::Channel).map_err(ConnectorError::backend))
                    })
                })
                .fuse();

            // Stream of ticket parameter updates (ticket price, minimum winning probability)
            let ticket_params_stream = ticket_params_stream
                .map_err(ConnectorError::from)
                .inspect_ok(|entry| tracing::trace!(?entry, "new ticket params"))
                .try_filter_map(|ticket_value_event| {
                    futures::future::ready(model_to_ticket_params(ticket_value_event).map(Some))
                })
                .inspect_ok(|(new_ticket_price, new_win_prob)| {
                    // This cannot block, because there are no other concurrent writers/upgradeable readers
                    let tv = ticket_values.upgradable_read();
                    if let Some((current_win_prob, current_ticket_price)) = tv.as_ref().copied() {
                        if &current_ticket_price != new_ticket_price && !current_win_prob.approx_eq(new_win_prob) {
                            parking_lot::RwLockUpgradableReadGuard::upgrade(tv)
                                .replace((*new_win_prob, *new_ticket_price));
                        } else if &current_ticket_price != new_ticket_price {
                            parking_lot::RwLockUpgradableReadGuard::upgrade(tv)
                                .replace((current_win_prob, *new_ticket_price));
                        } else if !current_win_prob.approx_eq(new_win_prob) {
                            parking_lot::RwLockUpgradableReadGuard::upgrade(tv)
                                .replace((*new_win_prob, current_ticket_price));
                        }
                    }
                })
                .and_then(|(new_ticket_price, new_win_prob)| {
                    let values_cache = values_cache.clone();
                    async move {
                        let mut events = Vec::<SubscribedEventType>::new();
                        values_cache
                            .entry(CHAIN_INFO_CACHE_KEY)
                            .and_compute_with(|cached_entry| {
                                futures::future::ready(match cached_entry {
                                    Some(chain_info) => {
                                        let mut chain_info = chain_info.into_value();
                                        if chain_info.ticket_price != new_ticket_price {
                                            events.push(SubscribedEventType::TicketPrice((
                                                new_ticket_price,
                                                Some(chain_info.ticket_price),
                                            )));
                                            chain_info.ticket_price = new_ticket_price;
                                        }
                                        if !chain_info.ticket_win_prob.approx_eq(&new_win_prob) {
                                            events.push(SubscribedEventType::WinningProbability((
                                                new_win_prob,
                                                Some(chain_info.ticket_win_prob),
                                            )));
                                            chain_info.ticket_win_prob = new_win_prob;
                                        }

                                        if !events.is_empty() {
                                            moka::ops::compute::Op::Put(chain_info)
                                        } else {
                                            moka::ops::compute::Op::Nop
                                        }
                                    }
                                    None => {
                                        tracing::warn!(
                                            "chain info not present in the cache before ticket params update"
                                        );
                                        events.push(SubscribedEventType::TicketPrice((new_ticket_price, None)));
                                        events.push(SubscribedEventType::WinningProbability((new_win_prob, None)));
                                        moka::ops::compute::Op::Nop
                                    }
                                })
                            })
                            .await;
                        Ok(futures::stream::iter(events).map(Ok::<_, ConnectorError>))
                    }
                })
                .try_flatten()
                .fuse();

            let mut account_counter = 0;
            let mut channel_counter = 0;
            if min_accounts == 0 && min_channels == 0 {
                tracing::info!(account_counter, channel_counter, time = ?sync_started.elapsed(), "on-chain graph has been synced");
                let _ = connection_ready_tx.take().unwrap().send(Ok(()));
            }

            futures::stream::Abortable::new(
                (account_stream, channel_stream, ticket_params_stream).merge(),
                abort_reg,
            )
            .inspect_ok(move |event_type| {
                if connection_ready_tx.is_some() {
                    match event_type {
                        SubscribedEventType::Account(_) => account_counter += 1,
                        SubscribedEventType::Channel(_) => channel_counter += 1,
                        _ => {}
                    }

                    let pct_synced =
                        ((account_counter + channel_counter) * 100 / (min_accounts + min_channels)).clamp(0, 100);
                    tracing::debug!(
                        pct_synced,
                        sync_quota,
                        account_counter,
                        channel_counter,
                        "percentage of connection quota synced"
                    );

                    // Send the completion notification
                    // once we reach the expected number of accounts and channels with
                    // the given tolerance
                    if account_counter >= min_accounts && channel_counter >= min_channels {
                        tracing::info!(account_counter, channel_counter, time = ?sync_started.elapsed(), "on-chain graph has been synced");
                        let _ = connection_ready_tx.take().unwrap().send(Ok(()));
                    }
                }
            })
            .for_each(|event_type| {
                let event_tx = event_tx.clone();
                async move {
                    match event_type {
                        Ok(SubscribedEventType::Account((new_account, old_account))) => {
                            tracing::debug!(%new_account, "account inserted");
                            // We only track public accounts as events and also
                            // broadcast announcements of already existing accounts (old_account == None).
                            if new_account.has_announced_with_routing_info()
                                && old_account.is_none_or(|a| !a.has_announced_with_routing_info())
                            {
                                tracing::debug!(account = %new_account, "new announcement");
                                let _ = event_tx
                                    .broadcast_direct(ChainEvent::Announcement(new_account.clone()))
                                    .await;
                            }
                        }
                        Ok(SubscribedEventType::Channel((new_channel, Some(changes)))) => {
                            tracing::debug!(
                                id = %new_channel.get_id(),
                                src = %new_channel.source, dst = %new_channel.destination,
                                num_changes = changes.len(),
                                "channel updated"
                            );
                            process_channel_changes_into_events(new_channel, changes, &me, &event_tx).await;
                        }
                        Ok(SubscribedEventType::Channel((new_channel, None))) => {
                            tracing::debug!(
                                id = %new_channel.get_id(),
                                src = %new_channel.source, dst = %new_channel.destination,
                                "channel opened"
                            );
                            let _ = event_tx.broadcast_direct(ChainEvent::ChannelOpened(new_channel)).await;
                        }
                        Ok(SubscribedEventType::WinningProbability((new, old))) => {
                            let old = old.unwrap_or_default();
                            match new.approx_cmp(&old) {
                                Ordering::Less => {
                                    tracing::debug!(%new, %old, "winning probability decreased");
                                    let _ = event_tx
                                        .broadcast_direct(ChainEvent::WinningProbabilityDecreased(new))
                                        .await;
                                }
                                Ordering::Greater => {
                                    tracing::debug!(%new, %old, "winning probability increased");
                                    let _ = event_tx
                                        .broadcast_direct(ChainEvent::WinningProbabilityIncreased(new))
                                        .await;
                                }
                                Ordering::Equal => {}
                            }
                        }
                        Ok(SubscribedEventType::TicketPrice((new, old))) => {
                            tracing::debug!(%new, ?old, "ticket price changed");
                            let _ = event_tx.broadcast_direct(ChainEvent::TicketPriceChanged(new)).await;
                        }
                        Err(error) => {
                            tracing::error!(%error, "error processing account/graph/ticket params subscription");
                        }
                    }
                }
            })
            .await;

            // Only transition to SubscriptionEnded if currently Ready or Connecting —
            // don't overwrite terminal error states (ServerNotHealthy, ConnectionFailed, etc.)
            tracing::warn!("chain subscription stream ended, marking chain health as degraded");
            let current = health.load(AtomicOrdering::Relaxed);
            if matches!(current, ChainHealthState::Connecting | ChainHealthState::Ready) {
                let _ = health.compare_exchange(
                    current,
                    ChainHealthState::SubscriptionEnded,
                    AtomicOrdering::Relaxed,
                    AtomicOrdering::Relaxed,
                );
            }
        });

        connection_ready_rx
            .timeout(futures_time::time::Duration::from(timeout))
            .map(|res| match res {
                Ok(Ok(Ok(_))) => Ok(abort_handle),
                Ok(Ok(Err(error))) => {
                    abort_handle.abort();
                    Err(ConnectorError::from(error))
                }
                Ok(Err(_)) => {
                    abort_handle.abort();
                    Err(ConnectorError::InvalidState("failed to determine connection state"))
                }
                Err(_) => {
                    abort_handle.abort();
                    tracing::warn!(min_accounts, min_channels, "connection timeout when syncing");
                    Err(ConnectorError::ConnectionTimeout)
                }
            })
            .await
    }

    /// Connects to the chain using the underlying client, syncs all on-chain data,
    /// and subscribes for all future updates.
    ///
    /// If the connection does not finish within
    /// [`BlockchainConnectorConfig::connection_timeout`](BlockchainConnectorConfig)
    /// the [`ConnectorError::ConnectionTimeout`] error is returned.
    ///
    /// Most of the operations with the Connector will fail if it is not connected first.
    ///
    /// There are some notable exceptions that DO NOT require a prior call to `connect`:
    /// - all the [`ChainValues`](hopr_api::chain::ChainValues) methods,
    /// - all the [`ChainReadSafeOperations`](hopr_api::chain::ChainReadSafeOperations) methods,
    /// - all the [`ChainWriteSafeOperations`](hopr_api::chain::ChainWriteSafeOperations) methods,
    /// - [`me`](hopr_api::chain::ChainReadChannelOperations::me)
    ///
    /// If you wish to only call operations from the above Chain APIs, consider constructing
    /// the [`HoprBlockchainReader`](crate::HoprBlockchainReader) instead.
    pub async fn connect(&mut self) -> Result<(), ConnectorError> {
        if self
            .connection_handle
            .as_ref()
            .filter(|handle| !handle.is_aborted())
            .is_some()
        {
            return Err(ConnectorError::InvalidState("connector is already connected"));
        }

        self.health.store(ChainHealthState::Connecting, AtomicOrdering::Relaxed);

        let abort_handle = match self
            .do_connect(self.cfg.connection_sync_timeout.max(MIN_CONNECTION_TIMEOUT))
            .await
        {
            Ok(handle) => handle,
            Err(e @ ConnectorError::ServerNotHealthy) => {
                self.health
                    .store(ChainHealthState::ServerNotHealthy, AtomicOrdering::Relaxed);
                return Err(e);
            }
            Err(e @ ConnectorError::ConnectionTimeout) => {
                self.health
                    .store(ChainHealthState::SyncTimedOut, AtomicOrdering::Relaxed);
                return Err(e);
            }
            Err(e) => {
                self.health
                    .store(ChainHealthState::ConnectionFailed, AtomicOrdering::Relaxed);
                return Err(e);
            }
        };

        self.connection_handle = Some(abort_handle);
        // Only transition to Ready if still Connecting — the subscription task
        // may have already set SubscriptionEnded in a race.
        let _ = self.health.compare_exchange(
            ChainHealthState::Connecting,
            ChainHealthState::Ready,
            AtomicOrdering::Relaxed,
            AtomicOrdering::Relaxed,
        );

        tracing::info!(node = %self.chain_key.public().to_address(), "connected to chain as node");
        Ok(())
    }

    /// Returns the reference to the underlying client.
    pub fn client(&self) -> &C {
        self.client.as_ref()
    }

    /// Checks if the connector is [connected](HoprBlockchainConnector::connect) to the chain.
    pub fn is_connected(&self) -> bool {
        self.check_connection_state().is_ok()
    }
}

impl<B, C, P> HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync,
    P::TxRequest: Send + Sync,
{
    async fn send_tx<'a>(
        &'a self,
        tx_req: P::TxRequest,
        custom_tx_multiplier: Option<u32>,
        custom_signer: Option<ChainKeypair>,
    ) -> Result<impl Future<Output = Result<ChainReceipt, ConnectorError>> + Send + 'a, ConnectorError> {
        let chain_info = self.query_cached_chain_info().await?;
        let tx_timeout = custom_tx_multiplier.unwrap_or(self.cfg.tx_timeout_multiplier).max(1)
            * chain_info.finality
            * chain_info.expected_block_time;
        Ok(self
            .sequencer
            .enqueue_transaction(tx_req, tx_timeout.max(MIN_TX_CONFIRM_TIMEOUT), custom_signer)
            .await?
            .and_then(|tx| {
                if let Some(tx_exec) = tx.safe_execution
                    && !tx_exec.success
                {
                    return futures::future::err(ConnectorError::InnerTxFailed(
                        tx_exec.revert_reason.unwrap_or("n/a".into()),
                    ));
                }
                futures::future::ready(
                    ChainReceipt::from_str(&tx.transaction_hash.0)
                        .map_err(|_| ConnectorError::TypeConversion("invalid tx hash".into())),
                )
            }))
    }
}

impl<B, C, P, R> hopr_api::node::ComponentStatusReporter for HoprBlockchainConnector<C, B, P, R> {
    fn component_status(&self) -> hopr_api::node::ComponentStatus {
        self.health.load(AtomicOrdering::Relaxed).into()
    }
}

impl<B, C, P, R> HoprBlockchainConnector<C, R, B, P> {
    #[inline]
    pub(crate) fn check_connection_state(&self) -> Result<(), ConnectorError> {
        self.connection_handle
            .as_ref()
            .filter(|handle| !handle.is_aborted()) // Do a safety check
            .ok_or_else(|| ConnectorError::InvalidState("connector is not connected"))
            .map(|_| ())
    }

    /// Invalidates all cached on-chain data.
    pub fn invalidate_caches(&self) {
        self.channel_by_parties.invalidate_all();
        self.channel_by_id.invalidate_all();
        self.packet_to_chain.invalidate_all();
        self.chain_to_packet.invalidate_all();
        self.values.invalidate_all();
    }
}

impl<B, C, P, R> Drop for HoprBlockchainConnector<C, R, B, P> {
    fn drop(&mut self) {
        self.health.store(ChainHealthState::Dropped, AtomicOrdering::Relaxed);
        self.events.0.close();
        if let Some(abort_handle) = self.connection_handle.take() {
            abort_handle.abort();
        }
    }
}

impl<B, C, P, R> HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
    C: Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    /// Returns a [`PathAddressResolver`] using this connector.
    pub fn as_path_resolver(&self) -> ChainPathResolver<'_, Self> {
        self.into()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use blokli_client::BlokliTestState;
    use hex_literal::hex;
    use hopr_api::{chain::ChainWriteTicketOperations, types::chain::contract_addresses_for_network};

    use super::*;
    use crate::{
        InMemoryBackend,
        testing::{BlokliTestStateBuilder, ChainMutator, FullStateEmulator},
    };

    pub const PRIVATE_KEY_1: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    pub const PRIVATE_KEY_2: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    pub const MODULE_ADDR: [u8; 20] = hex!("1111111111111111111111111111111111111111");

    pub type TestConnector<C> = HoprBlockchainConnector<
        C,
        InMemoryBackend,
        SafePayloadGenerator,
        <SafePayloadGenerator as PayloadGenerator>::TxRequest,
    >;

    pub fn create_connector<C>(blokli_client: C) -> anyhow::Result<TestConnector<C>>
    where
        C: BlokliQueryClient + BlokliTransactionClient + BlokliSubscriptionClient + Send + Sync + 'static,
    {
        let ckp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        Ok(HoprBlockchainConnector::new(
            ckp.clone(),
            Default::default(),
            blokli_client,
            InMemoryBackend::default(),
            SafePayloadGenerator::new(
                &ckp,
                contract_addresses_for_network("rotsee").unwrap().1,
                MODULE_ADDR.into(),
            ),
        ))
    }

    #[tokio::test]
    async fn connector_should_connect() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default().build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        assert!(connector.is_connected());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_not_connect_when_blokli_not_healthy() -> anyhow::Result<()> {
        let state = BlokliTestState {
            health: "DOWN".into(),
            ..Default::default()
        };

        let blokli_client = BlokliTestStateBuilder::from(state).build_static_client();

        let mut connector = create_connector(blokli_client)?;

        let res = connector.connect().await;

        assert!(matches!(res, Err(ConnectorError::ServerNotHealthy)));
        assert!(!connector.is_connected());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_handle_inner_tx_failure_during_redemption() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!(
            "60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"
        ))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!(
            "71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"
        ))?;
        let account_2 = AccountEntry {
            public_key: *offchain_key_2.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 2.into(),
        };

        let channel_1 = ChannelEntry::builder()
            .between(
                &ChainKeypair::from_secret(&PRIVATE_KEY_2)?,
                &ChainKeypair::from_secret(&PRIVATE_KEY_1)?,
            )
            .amount(10)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(1)
            .build()?;

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1])
            .with_hopr_network_chain_info("rotsee")
            .build_dynamic_client_with_mutator(ChainMutator::new(
                move |_: &[u8], state: &mut BlokliTestState| -> Result<(), blokli_client::errors::BlokliClientError> {
                    // Update the channel ticket index, without the client noticing the change
                    // This will cause the transaction to be rejected in the Emulator, and
                    // not by the checks performed by the Connector before the redemption.
                    if let Some(c) = state.get_channel_by_id_mut(&(*channel_1.get_id()).into()) {
                        c.ticket_index = blokli_client::api::types::Uint64("2".into());
                        Ok(())
                    } else {
                        Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "channel unexpectedly not found"
                        ))
                        .into())
                    }
                },
                FullStateEmulator(MODULE_ADDR.into(), None),
            ))
            .with_tx_simulation_delay(Duration::from_millis(100))
            .with_use_internal_txs(true);

        let mut connector = create_connector(blokli_client.clone())?;
        connector.connect().await?;

        let hkc1 = ChainKeypair::from_secret(&hex!(
            "e17fe86ce6e99f4806715b0c9412f8dad89334bf07f72d5834207a9d8f19d7f8"
        ))?;
        let hkc2 = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))?;

        let ticket = TicketBuilder::default()
            .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?)
            .amount(1)
            .index(1)
            .channel_epoch(1)
            .eth_challenge(
                Challenge::from_hint_and_share(
                    &HalfKeyChallenge::new(hkc1.public().as_ref()),
                    &HalfKeyChallenge::new(hkc2.public().as_ref()),
                )?
                .to_ethereum_challenge(),
            )
            .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(
                &HalfKey::try_from(hkc1.secret().as_ref())?,
                &HalfKey::try_from(hkc2.secret().as_ref())?,
            )?)
            .into_redeemable(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Hash::default())?;

        let res = connector.redeem_ticket(ticket).await?;
        let err = res.await;
        assert!(matches!(err, Err(hopr_api::chain::TicketRedeemError::Rejected(_, _))));

        Ok(())
    }

    #[test]
    fn chain_health_all_variants_convert_to_component_status() {
        use hopr_api::node::ComponentStatus;
        let variants = [
            ChainHealthState::Ready,
            ChainHealthState::WaitingForConnection,
            ChainHealthState::Connecting,
            ChainHealthState::SubscriptionEnded,
            ChainHealthState::SyncTimedOut,
            ChainHealthState::ServerNotHealthy,
            ChainHealthState::ConnectionFailed,
            ChainHealthState::Dropped,
        ];
        for state in variants {
            let _: ComponentStatus = state.into();
        }
    }

    #[test]
    fn chain_health_ready_maps_to_component_ready() {
        use hopr_api::node::ComponentStatus;
        let status: ComponentStatus = ChainHealthState::Ready.into();
        assert!(status.is_ready());
    }

    #[test]
    fn chain_health_degraded_states() {
        use hopr_api::node::ComponentStatus;
        let s: ComponentStatus = ChainHealthState::SubscriptionEnded.into();
        assert!(s.is_degraded());
        let s: ComponentStatus = ChainHealthState::SyncTimedOut.into();
        assert!(s.is_degraded());
    }

    #[test]
    fn chain_health_unavailable_states() {
        use hopr_api::node::ComponentStatus;
        let s: ComponentStatus = ChainHealthState::ServerNotHealthy.into();
        assert!(s.is_unavailable());
        let s: ComponentStatus = ChainHealthState::ConnectionFailed.into();
        assert!(s.is_unavailable());
        let s: ComponentStatus = ChainHealthState::Dropped.into();
        assert!(s.is_unavailable());
    }

    #[test]
    fn chain_health_initializing_states() {
        use hopr_api::node::ComponentStatus;
        let s: ComponentStatus = ChainHealthState::WaitingForConnection.into();
        assert!(s.is_initializing());
        let s: ComponentStatus = ChainHealthState::Connecting.into();
        assert!(s.is_initializing());
    }

    #[tokio::test]
    async fn connector_health_starts_as_initializing() -> anyhow::Result<()> {
        use hopr_api::node::ComponentStatusReporter;
        let blokli_client = BlokliTestStateBuilder::default().build_static_client();
        let connector = create_connector(blokli_client)?;
        assert!(connector.component_status().is_initializing());
        Ok(())
    }

    #[tokio::test]
    async fn connector_health_ready_after_connect() -> anyhow::Result<()> {
        use hopr_api::node::ComponentStatusReporter;
        let blokli_client = BlokliTestStateBuilder::default().build_static_client();
        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;
        assert!(connector.component_status().is_ready());
        Ok(())
    }

    #[tokio::test]
    async fn connector_health_unavailable_when_server_not_healthy() -> anyhow::Result<()> {
        use hopr_api::node::ComponentStatusReporter;
        let state = BlokliTestState {
            health: "DOWN".into(),
            ..Default::default()
        };
        let blokli_client = BlokliTestStateBuilder::from(state).build_static_client();
        let mut connector = create_connector(blokli_client)?;
        let _ = connector.connect().await;
        assert!(connector.component_status().is_unavailable());
        Ok(())
    }

    #[test]
    fn health_cas_ready_only_from_connecting() {
        let health = AtomicChainHealthState::new(ChainHealthState::Connecting);
        let result = health.compare_exchange(
            ChainHealthState::Connecting,
            ChainHealthState::Ready,
            AtomicOrdering::Relaxed,
            AtomicOrdering::Relaxed,
        );
        assert!(result.is_ok());
        assert_eq!(health.load(AtomicOrdering::Relaxed), ChainHealthState::Ready);
    }

    #[test]
    fn health_cas_ready_fails_from_subscription_ended() {
        let health = AtomicChainHealthState::new(ChainHealthState::SubscriptionEnded);
        let result = health.compare_exchange(
            ChainHealthState::Connecting,
            ChainHealthState::Ready,
            AtomicOrdering::Relaxed,
            AtomicOrdering::Relaxed,
        );
        assert!(result.is_err());
        assert_eq!(
            health.load(AtomicOrdering::Relaxed),
            ChainHealthState::SubscriptionEnded
        );
    }

    #[test]
    fn health_subscription_ended_preserves_terminal_state() {
        let health = AtomicChainHealthState::new(ChainHealthState::ServerNotHealthy);
        let current = health.load(AtomicOrdering::Relaxed);
        // ServerNotHealthy is a terminal state — should NOT transition to SubscriptionEnded
        assert!(!matches!(
            current,
            ChainHealthState::Connecting | ChainHealthState::Ready
        ));
        // The conditional store would skip this
    }

    #[test]
    fn health_subscription_ended_from_ready() {
        let health = AtomicChainHealthState::new(ChainHealthState::Ready);
        let current = health.load(AtomicOrdering::Relaxed);
        if matches!(current, ChainHealthState::Connecting | ChainHealthState::Ready) {
            let _ = health.compare_exchange(
                current,
                ChainHealthState::SubscriptionEnded,
                AtomicOrdering::Relaxed,
                AtomicOrdering::Relaxed,
            );
        }
        assert_eq!(
            health.load(AtomicOrdering::Relaxed),
            ChainHealthState::SubscriptionEnded
        );
    }

    #[test]
    fn health_drop_overwrites_any_state() {
        for initial in [
            ChainHealthState::Ready,
            ChainHealthState::Connecting,
            ChainHealthState::SubscriptionEnded,
        ] {
            let health = AtomicChainHealthState::new(initial);
            health.store(ChainHealthState::Dropped, AtomicOrdering::Relaxed);
            assert_eq!(health.load(AtomicOrdering::Relaxed), ChainHealthState::Dropped);
        }
    }
}
