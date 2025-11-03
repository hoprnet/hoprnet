use std::time::Duration;
use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use futures::future::Either;
use futures_concurrency::stream::Merge;
use futures_time::future::FutureExt as FuturesTimeExt;
use petgraph::prelude::DiGraphMap;
use hopr_api::chain::HoprKeyIdent;
use hopr_async_runtime::AbortHandle;
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::Address;

use crate::backend::Backend;
use crate::connector::keys::HoprKeyMapper;
use crate::connector::utils::{model_to_account_entry, model_to_graph_entry, process_account_changes_into_events, process_channel_changes_into_events};
use crate::errors::ConnectorError;

mod channels;
mod accounts;
mod keys;
mod utils;
mod values;
mod events;
mod tickets;

type EventsChannel = (async_broadcast::Sender<ChainEvent>, async_broadcast::InactiveReceiver<ChainEvent>);

pub struct HoprBlockchainConnector<B, C, P> {
    payload_generator: P,
    chain_key: ChainKeypair,
    safe_address: Address,
    client: std::sync::Arc<C>,
    graph: std::sync::Arc<parking_lot::RwLock<DiGraphMap<HoprKeyIdent, ChannelId, ahash::RandomState>>>,
    backend: std::sync::Arc<B>,
    connection_handle: Option<AbortHandle>,
    events: EventsChannel,

    // KeyId <-> OffchainPublicKey mapping
    mapper: keys::HoprKeyMapper<B>,
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
    // Holds all the tickets which were submitted for redeeming
    redeeming_tickets: moka::future::Cache<TicketId, Box<VerifiedTicket>, ahash::RandomState>,
}

impl<B, C, P> HoprBlockchainConnector<B, C, P>
    where
        B: Backend + Send + Sync + 'static,
        C: BlokliSubscriptionClient + BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
        P: PayloadGenerator + Send + Sync + 'static
{
    pub fn new(chain_key: ChainKeypair, safe_address: Address, client: C, backend: B, payload_generator: P) -> Self {
        let backend = std::sync::Arc::new(backend);
        let (mut events_tx, events_rx) = async_broadcast::broadcast(1024);
        events_tx.set_overflow(true);
        events_tx.set_await_active(false);
        let events_rx = events_rx.deactivate();

        Self {
            payload_generator,
            chain_key,
            safe_address,
            client: std::sync::Arc::new(client),
            graph: std::sync::Arc::new(parking_lot::RwLock::new(
                DiGraphMap::with_capacity_and_hasher(
                    10_000,
                    100_000,
                    ahash::RandomState::default()))
            ),
            backend: backend.clone(),
            connection_handle: None,
            events: (events_tx, events_rx),
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
            redeeming_tickets: moka::future::CacheBuilder::new(2048)
                .time_to_live(Duration::from_secs(600))
                .build_with_hasher(ahash::RandomState::default()),
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
        let redeeming_tickets = self.redeeming_tickets.clone();

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
                        .and_then(|res| futures::future::ready(res.map(Either::Left).map_err(|e| ConnectorError::BackendError(e.into()))))
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
                        backend.insert_channel(channel.clone())
                            .map(|old| (channel, old.map(|old| old.diff(&channel))))
                    })
                        .map_err(|e| ConnectorError::BackendError(e.into()))
                        .and_then(|res| futures::future::ready(res.map(Either::Right).map_err(|e| ConnectorError::BackendError(e.into()))))
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
                    let redeeming_tickets = redeeming_tickets.clone();
                    let event_tx = event_tx.clone();
                    async move {
                        match account_or_channel {
                            Ok(Either::Left((new_account, maybe_old_account))) => {
                                tracing::debug!(%new_account, "account inserted");
                                process_account_changes_into_events(new_account, maybe_old_account, &event_tx).await;
                            },
                            Ok(Either::Right((new_channel, Some(changes)))) => {
                                tracing::debug!(id = %new_channel.get_id(), num_changes = changes.len(), "channel updated");
                                process_channel_changes_into_events(new_channel, changes, &me, &event_tx, &redeeming_tickets).await;
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
                },
                Ok(Err(_)) => {
                    abort_handle.abort();
                    Err(ConnectorError::InvalidState("failed to determine connection state"))
                },
                Err(_) => {
                    abort_handle.abort();
                    Err(ConnectorError::ConnectionTimeout)
                },

            })
            .await
    }


    pub async fn connect(&mut self, timeout: Duration) -> Result<(), ConnectorError> {
        if self.connection_handle
            .as_ref()
            .filter(|handle| !handle.is_aborted())
            .is_some() {
            return Err(ConnectorError::InvalidState("connector is already connected"));
        }

        let abort_handle = self.do_connect(timeout).await?;
        self.connection_handle = Some(abort_handle);
        Ok(())
    }
}

impl<B,C,P> HoprBlockchainConnector<B,C,P> {
    pub(crate) fn check_connection_state(&self) -> Result<(), ConnectorError> {
        self.connection_handle
            .as_ref()
            .filter(|handle| !handle.is_aborted()) // Do a safety check
            .ok_or(ConnectorError::InvalidState("connector is not connected"))
            .map(|_| ())
    }
}

impl<B,C,P> Drop for HoprBlockchainConnector<B,C,P> {
    fn drop(&mut self) {
        if let Some(abort_handle) = self.connection_handle.take() {
            abort_handle.abort();
        }
    }
}

