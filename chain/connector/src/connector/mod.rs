use std::{str::FromStr, time::Duration};

use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use futures_time::future::FutureExt as FuturesTimeExt;
use hopr_api::chain::{ChainPathResolver, ChainReceipt, HoprKeyIdent};
use hopr_async_runtime::AbortHandle;
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{Address, HoprBalance};
use petgraph::prelude::DiGraphMap;

use crate::{
    backend::Backend,
    connector::{
        keys::HoprKeyMapper,
        sequencer::TransactionSequencer,
        utils::{model_to_account_entry, model_to_graph_entry, process_channel_changes_into_events},
    },
    errors::ConnectorError,
};

mod accounts;
mod channels;
mod events;
mod keys;
mod sequencer;
mod tickets;
mod utils;
mod values;

type EventsChannel = (
    async_broadcast::Sender<ChainEvent>,
    async_broadcast::InactiveReceiver<ChainEvent>,
);

/// Configuration of the [`HoprBlockchainConnector`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, smart_default::SmartDefault)]
pub struct BlockchainConnectorConfig {
    /// Default time to wait until a transaction is confirmed.
    #[default(Duration::from_secs(30))]
    pub tx_confirm_timeout: Duration,
    /// Fee to use for new key bindings.
    #[default(HoprBalance::from_str("0.01 wxHOPR").unwrap())]
    pub new_key_binding_fee: HoprBalance,
}

/// A connector acting as a middleware between the HOPR APIs (see the [`hopr_api`] crate) and the Blokli Client API (see
/// the [`blokli_client`] crate).
///
/// The connector object cannot be cloned, and shall be used inside an `Arc` if cloning is needed.
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
    // Contains only the chain info structure
    values: moka::future::Cache<u32, blokli_client::api::types::ChainInfo>,
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
                10_000,
                100_000,
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
                id_to_key: moka::sync::CacheBuilder::new(10_000)
                    .time_to_idle(Duration::from_secs(600))
                    .build_with_hasher(ahash::RandomState::default()),
                key_to_id: moka::sync::CacheBuilder::new(10_000)
                    .time_to_idle(Duration::from_secs(600))
                    .build_with_hasher(ahash::RandomState::default()),
                backend,
            },
            chain_to_packet: moka::future::CacheBuilder::new(10_000)
                .time_to_idle(Duration::from_secs(600))
                .build_with_hasher(ahash::RandomState::default()),
            packet_to_chain: moka::future::CacheBuilder::new(10_000)
                .time_to_idle(Duration::from_secs(600))
                .build_with_hasher(ahash::RandomState::default()),
            channel_by_id: moka::future::CacheBuilder::new(100_000)
                .time_to_idle(Duration::from_secs(600))
                .build_with_hasher(ahash::RandomState::default()),
            channel_by_parties: moka::future::CacheBuilder::new(100_000)
                .time_to_idle(Duration::from_secs(600))
                .build_with_hasher(ahash::RandomState::default()),
            values: moka::future::CacheBuilder::new(1)
                .time_to_live(Duration::from_secs(600))
                .build(),
        }
    }

    async fn do_connect(&self, timeout: Duration) -> Result<AbortHandle, ConnectorError> {
        const TOLERANCE: f64 = 0.01;
        let min_accounts = (self.client.count_accounts(None).await? as f64 * (1.0 - TOLERANCE)).round() as u32;
        let min_channels = (self.client.count_channels(None).await? as f64 * (1.0 - TOLERANCE)).round() as u32;
        tracing::debug!(min_accounts, min_channels, "connection thresholds");

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

        #[allow(unused, clippy::large_enum_variant)]
        enum SubscribedEventType {
            Account((AccountEntry, Option<AccountEntry>)),
            Channel((ChannelEntry, Option<Vec<ChannelChange>>)),
            WinningProbability((WinningProbability, Option<WinningProbability>)),
            TicketPrice((HoprBalance, Option<HoprBalance>)),
        }

        hopr_async_runtime::prelude::spawn(async move {
            let connections = client
                .subscribe_accounts(None)
                .and_then(|accounts| Ok((accounts, client.subscribe_graph()?)));

            if let Err(error) = connections {
                if let Some(connection_ready_tx) = connection_ready_tx.take() {
                    let _ = connection_ready_tx.send(Err(error));
                }
                return;
            }

            let (account_stream, channel_stream) = connections.unwrap();
            let account_stream = account_stream
                .inspect_ok(|entry| tracing::trace!(?entry, "new account entry"))
                .map_err(ConnectorError::from)
                .try_filter_map(|account| futures::future::ready(model_to_account_entry(account).map(Some)))
                .and_then(move |account| {
                    let mapper = mapper.clone();
                    let chain_to_packet = chain_to_packet.clone();
                    let packet_to_chain = packet_to_chain.clone();
                    hopr_async_runtime::prelude::spawn_blocking(move || {
                        mapper.key_to_id.insert(account.public_key, Some(account.key_id));
                        mapper.id_to_key.insert(account.key_id, Some(account.public_key));
                        mapper.backend.insert_account(account.clone()).map(|old| (account, old))
                    })
                    .map_err(|e| ConnectorError::BackendError(e.into()))
                    .and_then(move |res| {
                        let chain_to_packet = chain_to_packet.clone();
                        let packet_to_chain = packet_to_chain.clone();
                        async move {
                            if let Ok((account, _)) = &res {
                                // Rather update the cached entry than invalidating it
                                chain_to_packet
                                    .insert(account.chain_addr, Some(account.public_key))
                                    .await;
                                packet_to_chain
                                    .insert(account.public_key, Some(account.chain_addr))
                                    .await;
                            }
                            res.map(SubscribedEventType::Account)
                                .map_err(|e| ConnectorError::BackendError(e.into()))
                        }
                    })
                })
                .fuse();

            let channel_stream = channel_stream
                .map_err(ConnectorError::from)
                .inspect_ok(|entry| tracing::trace!(?entry, "new graph entry"))
                .try_filter_map(|graph_event| futures::future::ready(model_to_graph_entry(graph_event).map(Some)))
                .and_then(move |(src, dst, channel)| {
                    let graph = graph.clone();
                    let backend = backend.clone();
                    let channel_by_id = channel_by_id.clone();
                    let channel_by_parties = channel_by_parties.clone();
                    hopr_async_runtime::prelude::spawn_blocking(move || {
                        graph.write().add_edge(src.key_id, dst.key_id, channel.get_id());
                        backend
                            .insert_channel(channel)
                            .map(|old| (channel, old.map(|old| old.diff(&channel))))
                    })
                    .map_err(|e| ConnectorError::BackendError(e.into()))
                    .and_then(move |res| {
                        let channel_by_id = channel_by_id.clone();
                        let channel_by_parties = channel_by_parties.clone();
                        async move {
                            if let Ok((channel, _)) = &res {
                                // Rather update the cached entry than invalidating it
                                channel_by_id.insert(channel.get_id(), Some(channel.clone())).await;
                                channel_by_parties
                                    .insert(ChannelParties::from(channel), Some(channel.clone()))
                                    .await;
                            }
                            res.map(SubscribedEventType::Channel)
                                .map_err(|e| ConnectorError::BackendError(e.into()))
                        }
                    })
                })
                .fuse();

            let mut account_counter = 0;
            let mut channel_counter = 0;
            if min_accounts == 0 && min_channels == 0 {
                tracing::debug!(account_counter, channel_counter, "on-chain graph has been synced");
                let _ = connection_ready_tx.take().unwrap().send(Ok(()));
            }

            futures::stream::Abortable::new((account_stream, channel_stream).merge(), abort_reg)
                .inspect_ok(move |event_type| {
                    if connection_ready_tx.is_some() {
                        match event_type {
                            SubscribedEventType::Account(_) => account_counter += 1,
                            SubscribedEventType::Channel(_) => channel_counter += 1,
                            _ => {}
                        }

                        // Send the completion notification
                        // once we reach the expected number of accounts and channels with
                        // the given tolerance
                        if account_counter >= min_accounts && channel_counter >= min_channels {
                            tracing::debug!(account_counter, channel_counter, "on-chain graph has been synced");
                            let _ = connection_ready_tx.take().unwrap().send(Ok(()));
                        }
                    }
                })
                .for_each(|event_type| {
                    let event_tx = event_tx.clone();
                    let values_cache = values_cache.clone();
                    async move {
                        match event_type {
                            Ok(SubscribedEventType::Account((new_account, old_account))) => {
                                tracing::debug!(%new_account, "account inserted");
                                // We only track public accounts as events and also
                                // broadcast announcements of already existing accounts (old_account == None).
                                if new_account.has_announced() && old_account.is_none_or(|a| !a.has_announced()) {
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
                            // TODO: update the values in values_cache instead of invalidating it (make them separate
                            // cache entries?)
                            Ok(SubscribedEventType::WinningProbability((new, old))) => {
                                let old = old.unwrap_or_default();
                                if new.approx_cmp(&old).is_gt() {
                                    tracing::debug!(%new, %old, "winning probability increased");
                                    values_cache.invalidate_all();
                                    let _ = event_tx
                                        .broadcast_direct(ChainEvent::WinningProbabilityIncreased(new))
                                        .await;
                                } else if new.approx_cmp(&old).is_lt() {
                                    tracing::debug!(%new, %old, "winning probability decreased");
                                    values_cache.invalidate_all();
                                    let _ = event_tx
                                        .broadcast_direct(ChainEvent::WinningProbabilityDecreased(new))
                                        .await;
                                }
                            }
                            Ok(SubscribedEventType::TicketPrice((new, old))) => {
                                tracing::debug!(%new, ?old, "ticket price changed");
                                values_cache.invalidate_all();
                                let _ = event_tx.broadcast_direct(ChainEvent::TicketPriceChanged(new)).await;
                            }
                            Err(error) => {
                                tracing::error!(%error, "error processing account/graph subscription");
                            }
                        }
                    }
                })
                .await;
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
                    tracing::error!(min_accounts, min_channels, "connection timeout when syncing");
                    Err(ConnectorError::ConnectionTimeout)
                }
            })
            .await
    }

    /// Connects to the chain using the underlying client, syncs all on-chain data
    /// and subscribes for all future updates.
    ///
    /// If the sync of the current state does not happen within `timeout`, a [`ConnectorError::ConnectionTimeout`]
    /// error is returned.
    pub async fn connect(&mut self, timeout: Duration) -> Result<(), ConnectorError> {
        if self
            .connection_handle
            .as_ref()
            .filter(|handle| !handle.is_aborted())
            .is_some()
        {
            return Err(ConnectorError::InvalidState("connector is already connected"));
        }

        let abort_handle = self.do_connect(timeout).await?;

        if let Err(error) = self.sequencer.start().await {
            abort_handle.abort();
            return Err(error);
        }

        self.connection_handle = Some(abort_handle);

        tracing::info!(node = %self.chain_key.public().to_address(), "connected to chain as node");
        Ok(())
    }

    /// Returns the reference to the underlying client.
    pub fn client(&self) -> &C {
        self.client.as_ref()
    }
}

impl<B, C, P> HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    C: BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync,
    P::TxRequest: Send + Sync,
{
    async fn send_tx<'a>(
        &'a self,
        tx_req: P::TxRequest,
    ) -> Result<impl Future<Output = Result<ChainReceipt, ConnectorError>> + Send + 'a, ConnectorError> {
        Ok(self
            .sequencer
            .enqueue_transaction(tx_req, self.cfg.tx_confirm_timeout)
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
}

impl<B, C, P, R> Drop for HoprBlockchainConnector<C, R, B, P> {
    fn drop(&mut self) {
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
