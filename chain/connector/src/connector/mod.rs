use std::{str::FromStr, time::Duration};

use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt, future::Either};
use futures_concurrency::stream::Merge;
use futures_time::future::FutureExt as FuturesTimeExt;
use hopr_api::chain::{ChainPathResolver, ChainReceipt, HoprKeyIdent};
use hopr_async_runtime::AbortHandle;
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::Address;
use petgraph::prelude::DiGraphMap;

use crate::{
    TempDbBackend,
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

pub(crate) const DEFAULT_TX_TIMEOUT: Duration = Duration::from_secs(10);

/// A connector acting as a middleware between the HOPR APIs (see the [`hopr_api`] crate) and the Blokli Client API (see
/// the [`blokli_client`] crate).
///
/// The connector object cannot be cloned, and shall be used inside an `Arc` if cloning is needed.
pub struct HoprBlockchainConnector<C, R, B = TempDbBackend, P = SafePayloadGenerator> {
    payload_generator: P,
    chain_key: ChainKeypair,
    client: std::sync::Arc<C>,
    graph: std::sync::Arc<parking_lot::RwLock<DiGraphMap<HoprKeyIdent, ChannelId, ahash::RandomState>>>,
    backend: std::sync::Arc<B>,
    connection_handle: Option<AbortHandle>,
    sequencer: TransactionSequencer<C, R>,
    events: EventsChannel,

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

impl<B, C, P> HoprBlockchainConnector<C, P::TxRequest, B, P>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliSubscriptionClient + BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    /// Creates a new instance.
    pub fn new(chain_key: ChainKeypair, client: C, backend: B, payload_generator: P) -> Self {
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
        let num_accounts = self.client.count_accounts(None).await? - 1;
        let num_channels = self.client.count_channels(None).await? - 1;

        let (abort_handle, abort_reg) = AbortHandle::new_pair();

        let (connection_ready_tx, connection_ready_rx) = futures::channel::oneshot::channel();
        let mut connection_ready_tx = Some(connection_ready_tx);

        let client = self.client.clone();
        let mapper = self.mapper.clone();
        let backend = self.backend.clone();
        let graph = self.graph.clone();
        let event_tx = self.events.0.clone();
        let me = self.chain_key.public().to_address();

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
                .map_err(ConnectorError::from)
                .try_filter_map(|account| futures::future::ready(model_to_account_entry(account).map(Some)))
                .and_then(move |account| {
                    let mapper = mapper.clone();
                    hopr_async_runtime::prelude::spawn_blocking(move || {
                        mapper.key_to_id.insert(account.public_key, Some(account.key_id));
                        mapper.id_to_key.insert(account.key_id, Some(account.public_key));
                        mapper.backend.insert_account(account.clone()).map(|old| (account, old))
                    })
                    .map_err(|e| ConnectorError::BackendError(e.into()))
                    .and_then(|res| {
                        futures::future::ready(
                            res.map(Either::Left)
                                .map_err(|e| ConnectorError::BackendError(e.into())),
                        )
                    })
                })
                .fuse();

            let channel_stream = channel_stream
                .map_err(ConnectorError::from)
                .try_filter_map(|graph_event| futures::future::ready(model_to_graph_entry(graph_event).map(Some)))
                .and_then(move |(src, dst, channel)| {
                    let graph = graph.clone();
                    let backend = backend.clone();
                    hopr_async_runtime::prelude::spawn_blocking(move || {
                        graph.write().add_edge(src.key_id, dst.key_id, channel.get_id());
                        backend
                            .insert_channel(channel.clone())
                            .map(|old| (channel, old.map(|old| old.diff(&channel))))
                    })
                    .map_err(|e| ConnectorError::BackendError(e.into()))
                    .and_then(|res| {
                        futures::future::ready(
                            res.map(Either::Right)
                                .map_err(|e| ConnectorError::BackendError(e.into())),
                        )
                    })
                })
                .fuse();

            let mut account_counter = 0;
            let mut channel_counter = 0;

            futures::stream::Abortable::new((account_stream, channel_stream).merge(), abort_reg)
                .inspect_ok(move |account_or_channel| {
                    match account_or_channel {
                        Either::Left(_) => if account_counter < num_accounts {
                            account_counter += 1;
                        },
                        Either::Right(_) => if channel_counter < num_channels {
                            channel_counter += 1;
                        }
                    }
                    if account_counter >= num_accounts && channel_counter >= num_channels {
                        tracing::debug!("on-chain graph has been synced");
                        if let Some(connection_ready_tx) = connection_ready_tx.take() {
                            let _ = connection_ready_tx.send(Ok(()));
                        }
                    }
                })
                .for_each(|account_or_channel| {
                    let event_tx = event_tx.clone();
                    async move {
                        match account_or_channel {
                            Ok(Either::Left((new_account, old_account))) => {
                                tracing::debug!(%new_account, "account inserted");
                                // We only track public accounts as events and also
                                // broadcast announcements of already existing accounts (old_account == None).
                                if new_account.has_announced() && old_account.is_none_or(|a| !a.has_announced()) {
                                    tracing::debug!(account = %new_account, "new announcement");
                                    let _ = event_tx.broadcast_direct(ChainEvent::Announcement(new_account.clone())).await;
                                }
                            },
                            Ok(Either::Right((new_channel, Some(changes)))) => {
                                tracing::debug!(id = %new_channel.get_id(), num_changes = changes.len(), "channel updated");
                                process_channel_changes_into_events(new_channel, changes, &me, &event_tx).await;
                            },
                            Ok(Either::Right((new_channel, None))) => {
                                tracing::debug!(id = %new_channel.get_id(), "channel opened");
                                let _ = event_tx.broadcast_direct(ChainEvent::ChannelOpened(new_channel)).await;
                            }
                            Err(error) => {
                                tracing::error!(%error, "error processing account/graph subscription");
                            }
                        }
                    }
                }).await;
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

        self.sequencer.start().await?;

        let abort_handle = self.do_connect(timeout).await?;
        self.connection_handle = Some(abort_handle);

        Ok(())
    }
}

impl<B, C, P> HoprBlockchainConnector<C, P::TxRequest, B, P>
where
    C: BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync,
    P::TxRequest: Send + Sync,
{
    async fn send_tx(
        &self,
        tx_req: P::TxRequest,
    ) -> Result<impl Future<Output = Result<ChainReceipt, ConnectorError>> + Send, ConnectorError> {
        Ok(self
            .sequencer
            .enqueue_transaction(tx_req, DEFAULT_TX_TIMEOUT)
            .await?
            .and_then(|tx| {
                futures::future::ready(
                    tx.transaction_hash
                        .ok_or(ConnectorError::InvalidState("transaction hash missing"))
                        .and_then(|tx| {
                            ChainReceipt::from_str(&tx.0)
                                .map_err(|_| ConnectorError::TypeConversion("invalid tx hash".into()))
                        }),
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

impl<B, C, P, R> HoprBlockchainConnector<C, R, B, P>
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
