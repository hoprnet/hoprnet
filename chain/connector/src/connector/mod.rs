use std::{cmp::Ordering, str::FromStr, time::Duration};

use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use futures_time::future::FutureExt as FuturesTimeExt;
use hopr_api::chain::{ChainPathResolver, ChainReceipt, HoprKeyIdent};
use hopr_async_runtime::AbortHandle;
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
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

    // KeyId <-> OffchainPublicKey mapping
    mapper: HoprKeyMapper<B>,
    // Fast retrieval of chain keys by address
    chain_to_packet: moka::future::Cache<Address, Option<OffchainPublicKey>, ahash::RandomState>,
    // Fast retrieval of packet keys by chain key
    packet_to_chain: moka::future::Cache<OffchainPublicKey, Option<Address>, ahash::RandomState>,
    // Fast retrieval of channel entries by id
    channel_by_id: moka::future::Cache<ChannelId, Option<ChannelEntry>, ahash::RandomState>,
    // Fast retrieval of channel entries by parties
    channel_by_parties: moka::future::Cache<ChannelParties, Option<ChannelEntry>, ahash::RandomState>,
    // Contains chain info (no TTL - kept fresh by subscription handler)
    values: moka::future::Cache<u32, ParsedChainInfo>,
}

const EXPECTED_NUM_NODES: usize = 10_000;
const EXPECTED_NUM_CHANNELS: usize = 100_000;

const DEFAULT_CACHE_TIMEOUT: Duration = Duration::from_mins(10);
const SUBSCRIPTION_RETRY_INITIAL_DELAY: Duration = Duration::from_secs(2);
const SUBSCRIPTION_RETRY_MAX_DELAY: Duration = Duration::from_secs(30);

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum SubscribedEventType {
    Account((AccountEntry, Option<AccountEntry>)),
    Channel((ChannelEntry, Option<Vec<ChannelChange>>)),
    WinningProbability((WinningProbability, Option<WinningProbability>)),
    TicketPrice((HoprBalance, Option<HoprBalance>)),
}

#[derive(Debug)]
enum SubscriptionUpdate {
    Data(SubscribedEventType),
    Error(ConnectorError),
    StreamEnded(&'static str),
}

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
            chain_to_packet: moka::future::CacheBuilder::new(EXPECTED_NUM_NODES as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            packet_to_chain: moka::future::CacheBuilder::new(EXPECTED_NUM_NODES as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            channel_by_id: moka::future::CacheBuilder::new(EXPECTED_NUM_CHANNELS as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            channel_by_parties: moka::future::CacheBuilder::new(EXPECTED_NUM_CHANNELS as u64)
                .time_to_idle(DEFAULT_CACHE_TIMEOUT)
                .build_with_hasher(ahash::RandomState::default()),
            // No TTL: kept fresh by the Blokli subscription handler
            values: moka::future::CacheBuilder::new(1).build(),
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
            .count_channels(blokli_client::api::ChannelSelector {
                filter: None,
                status: Some(blokli_client::api::types::ChannelStatus::Open),
            })
            .await? as f64
            * sync_quota)
            .round() as u32;
        tracing::debug!(min_accounts, min_channels, "connection thresholds");

        let server_health = self.client.query_health().await?;
        if !server_health.eq_ignore_ascii_case("OK") {
            tracing::error!(server_health, "blokli server not healthy");
            return Err(ConnectorError::ServerNotHealthy);
        }

        let (abort_handle, abort_reg) = AbortHandle::new_pair();

        let (connection_ready_tx, connection_ready_rx) =
            futures::channel::oneshot::channel::<std::result::Result<(), ConnectorError>>();
        let connection_ready_tx = Some(connection_ready_tx);

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
        self.query_cached_chain_info().await?;

        hopr_async_runtime::prelude::spawn(async move {
            let sync_started = std::time::Instant::now();
            let mut connection_ready_tx = connection_ready_tx;
            let mut account_counter = 0;
            let mut channel_counter = 0;
            let mut retry_delay = SUBSCRIPTION_RETRY_INITIAL_DELAY;

            let subscription_task = async {
                loop {
                    let connections = client
                        .subscribe_accounts(blokli_client::api::AccountSelector::Any)
                        .and_then(|accounts| Ok((accounts, client.subscribe_graph()?)))
                        .and_then(|(accounts, channels)| Ok((accounts, channels, client.subscribe_ticket_params()?)));

                    let (account_stream, channel_stream, ticket_params_stream) = match connections {
                        Ok(streams) => streams,
                        Err(error) => {
                            tracing::error!(%error, "failed to establish account/graph/ticket subscriptions");
                            tracing::warn!(?retry_delay, "retrying account/graph/ticket subscriptions");
                            hopr_async_runtime::prelude::sleep(retry_delay).await;
                            retry_delay = (retry_delay + retry_delay).min(SUBSCRIPTION_RETRY_MAX_DELAY);
                            continue;
                        }
                    };

                    tracing::debug!("account/graph/ticket subscriptions established");
                    retry_delay = SUBSCRIPTION_RETRY_INITIAL_DELAY;

                    if min_accounts == 0 && min_channels == 0 && connection_ready_tx.is_some() {
                        tracing::info!(
                            account_counter,
                            channel_counter,
                            time = ?sync_started.elapsed(),
                            "on-chain graph has been synced"
                        );
                        let _ = connection_ready_tx.take().unwrap().send(Ok(()));
                    }

                    let graph_for_accounts = graph.clone();
                    let mapper_for_accounts = mapper.clone();
                    let chain_to_packet_for_accounts = chain_to_packet.clone();
                    let packet_to_chain_for_accounts = packet_to_chain.clone();
                    let account_stream = account_stream
                        .inspect_ok(|entry| tracing::trace!(?entry, "new account event"))
                        .map_err(ConnectorError::from)
                        .try_filter_map(|account| futures::future::ready(model_to_account_entry(account).map(Some)))
                        .and_then(move |account| {
                            let graph = graph_for_accounts.clone();
                            let mapper = mapper_for_accounts.clone();
                            let chain_to_packet = chain_to_packet_for_accounts.clone();
                            let packet_to_chain = packet_to_chain_for_accounts.clone();
                            hopr_async_runtime::prelude::spawn_blocking(move || {
                                mapper.key_to_id.insert(account.public_key, Some(account.key_id));
                                mapper.id_to_key.insert(account.key_id, Some(account.public_key));
                                graph.write().add_node(account.key_id);
                                mapper.backend.insert_account(account.clone()).map(|old| (account, old))
                            })
                            .map_err(ConnectorError::backend)
                            .and_then(move |res| {
                                let chain_to_packet = chain_to_packet.clone();
                                let packet_to_chain = packet_to_chain.clone();
                                async move {
                                    if let Ok((upserted_account, _)) = &res {
                                        chain_to_packet
                                            .insert(upserted_account.chain_addr, Some(upserted_account.public_key))
                                            .await;
                                        packet_to_chain
                                            .insert(upserted_account.public_key, Some(upserted_account.chain_addr))
                                            .await;
                                    }
                                    res.map(SubscribedEventType::Account).map_err(ConnectorError::backend)
                                }
                            })
                        })
                        .map(|res| match res {
                            Ok(v) => SubscriptionUpdate::Data(v),
                            Err(e) => SubscriptionUpdate::Error(e),
                        })
                        .chain(futures::stream::once(futures::future::ready(
                            SubscriptionUpdate::StreamEnded("accounts"),
                        )))
                        .fuse();

                    let graph_for_channels = graph.clone();
                    let backend_for_channels = backend.clone();
                    let channel_by_id_for_channels = channel_by_id.clone();
                    let channel_by_parties_for_channels = channel_by_parties.clone();
                    let channel_stream = channel_stream
                        .map_err(ConnectorError::from)
                        .inspect_ok(|entry| tracing::trace!(?entry, "new graph event"))
                        .try_filter_map(|graph_event| {
                            futures::future::ready(model_to_graph_entry(graph_event).map(Some))
                        })
                        .and_then(move |(src, dst, channel)| {
                            let graph = graph_for_channels.clone();
                            let backend = backend_for_channels.clone();
                            let channel_by_id = channel_by_id_for_channels.clone();
                            let channel_by_parties = channel_by_parties_for_channels.clone();
                            hopr_async_runtime::prelude::spawn_blocking(move || {
                                graph.write().add_edge(src.key_id, dst.key_id, *channel.get_id());
                                backend
                                    .insert_channel(channel)
                                    .map(|old| (channel, old.map(|old| old.diff(&channel))))
                            })
                            .map_err(ConnectorError::backend)
                            .and_then(move |res| {
                                let channel_by_id = channel_by_id.clone();
                                let channel_by_parties = channel_by_parties.clone();
                                async move {
                                    if let Ok((upserted_channel, _)) = &res {
                                        channel_by_id
                                            .insert(*upserted_channel.get_id(), Some(*upserted_channel))
                                            .await;
                                        channel_by_parties
                                            .insert(ChannelParties::from(upserted_channel), Some(*upserted_channel))
                                            .await;
                                    }
                                    res.map(SubscribedEventType::Channel).map_err(ConnectorError::backend)
                                }
                            })
                        })
                        .map(|res| match res {
                            Ok(v) => SubscriptionUpdate::Data(v),
                            Err(e) => SubscriptionUpdate::Error(e),
                        })
                        .chain(futures::stream::once(futures::future::ready(
                            SubscriptionUpdate::StreamEnded("graph"),
                        )))
                        .fuse();

                    let values_cache_for_tickets = values_cache.clone();
                    let ticket_params_stream = ticket_params_stream
                        .map_err(ConnectorError::from)
                        .inspect_ok(|entry| tracing::trace!(?entry, "new ticket params"))
                        .try_filter_map(|ticket_value_event| {
                            futures::future::ready(model_to_ticket_params(ticket_value_event).map(Some))
                        })
                        .and_then(|(new_ticket_price, new_win_prob)| {
                            let values_cache = values_cache_for_tickets.clone();
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
                                                events.push(SubscribedEventType::WinningProbability((
                                                    new_win_prob,
                                                    None,
                                                )));
                                                moka::ops::compute::Op::Nop
                                            }
                                        })
                                    })
                                    .await;
                                Ok(futures::stream::iter(events).map(Ok::<_, ConnectorError>))
                            }
                        })
                        .try_flatten()
                        .map(|res| match res {
                            Ok(v) => SubscriptionUpdate::Data(v),
                            Err(e) => SubscriptionUpdate::Error(e),
                        })
                        .chain(futures::stream::once(futures::future::ready(
                            SubscriptionUpdate::StreamEnded("ticket_params"),
                        )))
                        .fuse();

                    let mut merged = Box::pin((account_stream, channel_stream, ticket_params_stream).merge().fuse());
                    let mut should_retry = false;

                    while let Some(event_type) = merged.next().await {
                        match event_type {
                            SubscriptionUpdate::Data(event_type) => {
                                if connection_ready_tx.is_some() {
                                    match event_type {
                                        SubscribedEventType::Account(_) => account_counter += 1,
                                        SubscribedEventType::Channel(_) => channel_counter += 1,
                                        _ => {}
                                    }

                                    let pct_synced = ((account_counter + channel_counter) * 100
                                        / (min_accounts + min_channels))
                                        .clamp(0, 100);
                                    tracing::debug!(
                                        pct_synced,
                                        sync_quota,
                                        account_counter,
                                        channel_counter,
                                        "percentage of connection quota synced"
                                    );

                                    if account_counter >= min_accounts && channel_counter >= min_channels {
                                        tracing::info!(
                                            account_counter,
                                            channel_counter,
                                            time = ?sync_started.elapsed(),
                                            "on-chain graph has been synced"
                                        );
                                        let _ = connection_ready_tx.take().unwrap().send(Ok(()));
                                    }
                                }

                                match event_type {
                                    SubscribedEventType::Account((new_account, old_account)) => {
                                        tracing::debug!(%new_account, "account inserted");
                                        if new_account.has_announced_with_routing_info()
                                            && old_account.is_none_or(|a| !a.has_announced_with_routing_info())
                                        {
                                            tracing::debug!(account = %new_account, "new announcement");
                                            let _ = event_tx
                                                .broadcast_direct(ChainEvent::Announcement(new_account.clone()))
                                                .await;
                                        }
                                    }
                                    SubscribedEventType::Channel((new_channel, Some(changes))) => {
                                        tracing::debug!(
                                            id = %new_channel.get_id(),
                                            src = %new_channel.source, dst = %new_channel.destination,
                                            num_changes = changes.len(),
                                            "channel updated"
                                        );
                                        process_channel_changes_into_events(new_channel, changes, &me, &event_tx).await;
                                    }
                                    SubscribedEventType::Channel((new_channel, None)) => {
                                        tracing::debug!(
                                            id = %new_channel.get_id(),
                                            src = %new_channel.source, dst = %new_channel.destination,
                                            "channel opened"
                                        );
                                        let _ = event_tx.broadcast_direct(ChainEvent::ChannelOpened(new_channel)).await;
                                    }
                                    SubscribedEventType::WinningProbability((new, old)) => {
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
                                    SubscribedEventType::TicketPrice((new, old)) => {
                                        tracing::debug!(%new, ?old, "ticket price changed");
                                        let _ = event_tx.broadcast_direct(ChainEvent::TicketPriceChanged(new)).await;
                                    }
                                }
                            }
                            SubscriptionUpdate::Error(error) => {
                                tracing::error!(%error, "error processing account/graph/ticket params subscription");
                            }
                            SubscriptionUpdate::StreamEnded(stream_name) => {
                                should_retry = true;
                                tracing::warn!(
                                    stream = stream_name,
                                    "subscription stream ended, restarting all account/graph/ticket subscriptions"
                                );
                                break;
                            }
                        }
                    }

                    if !should_retry {
                        tracing::warn!("account/graph/ticket subscription loop stopped; restarting all subscriptions");
                    }

                    tracing::warn!(
                        ?retry_delay,
                        "retrying account/graph/ticket subscriptions after stream stop"
                    );
                    hopr_async_runtime::prelude::sleep(retry_delay).await;
                    retry_delay = (retry_delay + retry_delay).min(SUBSCRIPTION_RETRY_MAX_DELAY);
                }
            };

            match futures::future::Abortable::new(subscription_task, abort_reg).await {
                Ok(()) => tracing::warn!("subscription supervisor task stopped unexpectedly"),
                Err(_) => tracing::debug!("subscription supervisor task aborted"),
            }
        });

        connection_ready_rx
            .timeout(futures_time::time::Duration::from(timeout))
            .map(|res| match res {
                Ok(Ok(Ok(_))) => Ok(abort_handle),
                Ok(Ok(Err(error))) => {
                    tracing::error!(%error, "connection failed while establishing subscriptions");
                    abort_handle.abort();
                    Err(ConnectorError::from(error))
                }
                Ok(Err(_)) => {
                    tracing::error!("subscription task ended before signaling readiness");
                    abort_handle.abort();
                    Err(ConnectorError::InvalidState("failed to determine connection state"))
                }
                Err(_) => {
                    abort_handle.abort();
                    tracing::error!(min_accounts, min_channels, "connection timeout when syncing");
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

        let abort_handle = self
            .do_connect(self.cfg.connection_sync_timeout.max(MIN_CONNECTION_TIMEOUT))
            .await?;

        self.connection_handle = Some(abort_handle);

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
    ) -> Result<impl Future<Output = Result<ChainReceipt, ConnectorError>> + Send + 'a, ConnectorError> {
        let chain_info = self.query_cached_chain_info().await?;
        let tx_timeout = self.cfg.tx_timeout_multiplier.max(1) * chain_info.finality * chain_info.expected_block_time;
        Ok(self
            .sequencer
            .enqueue_transaction(tx_req, tx_timeout.max(MIN_TX_CONFIRM_TIMEOUT))
            .await?
            .and_then(|tx| {
                futures::future::ready(
                    ChainReceipt::from_str(&tx.transaction_hash.0)
                        .map_err(|_| ConnectorError::TypeConversion("invalid tx hash".into())),
                )
            }))
    }
}

impl<B, C, P, R> HoprBlockchainConnector<C, R, B, P> {
    pub(crate) fn check_connection_state(&self) -> Result<(), ConnectorError> {
        self.connection_handle
            .as_ref()
            .filter(|handle| !handle.is_aborted()) // Do a safety check
            .ok_or(ConnectorError::InvalidState("connector is not connected"))
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
        tracing::debug!("connector dropped, stopping background chain subscriptions");
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
    use hopr_chain_types::contract_addresses_for_network;

    use super::*;
    use crate::{InMemoryBackend, testing::BlokliTestStateBuilder};

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
}
