use std::time::Duration;
use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient, BlokliTransactionClient};
use futures::{StreamExt, TryFutureExt, TryStreamExt};
use petgraph::prelude::DiGraphMap;
use hopr_api::chain::HoprKeyIdent;
use hopr_async_runtime::AbortHandle;
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::Address;

use crate::backend::Backend;
use crate::connector::keys::HoprKeyMapper;
use crate::connector::utils::{model_to_account_entry, model_to_graph_entry};
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
    payload_generator: std::sync::Arc<P>,
    chain_key: ChainKeypair,
    safe_address: Address,
    client: std::sync::Arc<C>,
    graph: std::sync::Arc<parking_lot::RwLock<DiGraphMap<HoprKeyIdent, ChannelId, ahash::RandomState>>>,
    backend: std::sync::Arc<B>,
    connection_handles: Vec<AbortHandle>,
    events: std::sync::Arc<EventsChannel>,

    // Caches
    mapper: keys::HoprKeyMapper<B>,
    chain_to_packet: moka::future::Cache<Address, Option<OffchainPublicKey>, ahash::RandomState>,
    packet_to_chain: moka::future::Cache<OffchainPublicKey, Option<Address>, ahash::RandomState>,
    channel_by_id: moka::future::Cache<ChannelId, Option<ChannelEntry>, ahash::RandomState>,
    channel_by_parties: moka::future::Cache<ChannelParties, Option<ChannelEntry>, ahash::RandomState>,
    values: moka::future::Cache<u32, blokli_client::api::types::ChainInfo>,
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
            payload_generator: std::sync::Arc::new(payload_generator),
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
            connection_handles: Vec::with_capacity(2),
            events: std::sync::Arc::new((events_tx, events_rx)),
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

    async fn connect_accounts(&self) -> Result<AbortHandle, ConnectorError> {
        let (accounts_connected_tx, accounts_connected_rx) = futures::channel::oneshot::channel();
        let client = self.client.clone();
        let mapper = self.mapper.clone();
        let event_tx = self.events.0.clone();
        hopr_async_runtime::prelude::spawn(async move {
            match client.subscribe_accounts(None) {
                Ok(stream) => {
                    let (stream, handle) = futures::stream::abortable(stream);
                    let _ = accounts_connected_tx.send(Ok(handle));
                    stream
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
                                .and_then(|res| futures::future::ready(res.map_err(|e| ConnectorError::BackendError(e.into()))))
                        })
                        .for_each(|res| {
                            let event_tx = event_tx.clone();
                            async move {
                                match res {
                                    Ok((new, _)) => {
                                        tracing::trace!(%new, "account inserted");
                                        // We only track public accounts as events
                                        if new.has_announced() {
                                            let _ = event_tx.broadcast_direct(ChainEvent::Announcement {
                                                peer: new.public_key.into(),
                                                address: new.chain_addr,
                                                multiaddresses: new.get_multiaddr().into_iter().collect(),
                                            }).await;
                                        }
                                    },
                                    Err(error) => {
                                        tracing::error!(%error, "error processing account from subscription");
                                    }
                                }
                            }
                        })
                        .await;
                    tracing::warn!("account subscription ended");
                }
                Err(error) => {
                    let _ = accounts_connected_tx.send(Err(error));
                },
            }
        });

        accounts_connected_rx
            .map_err(|_| ConnectorError::InvalidState("cannot notify account connection"))
            .and_then(|res| futures::future::ready(res.map_err(ConnectorError::from)))
            .await
    }

    async fn connect_graph(&self) -> Result<AbortHandle, ConnectorError> {
        let (accounts_connected_tx, accounts_connected_rx) = futures::channel::oneshot::channel();
        let client = self.client.clone();
        let graph = self.graph.clone();
        let backend = self.backend.clone();
        let event_tx = self.events.0.clone();
        let me = self.chain_key.public().to_address();
        hopr_async_runtime::prelude::spawn(async move {
            match client.subscribe_graph() {
                Ok(stream) => {
                    let (stream, handle) = futures::stream::abortable(stream);
                    let _ = accounts_connected_tx.send(Ok(handle));
                    stream
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
                                .and_then(|res| futures::future::ready(res.map_err(|e| ConnectorError::BackendError(e.into()))))
                        })
                        .for_each(|res| {
                            let event_tx = event_tx.clone();
                            async move {
                                match res {
                                    Ok((new, Some(changes))) => {
                                        for change in changes {
                                            tracing::trace!(id = %new.get_id(), %change, "channel updated");
                                            match change {
                                                ChannelChange::Status { left: ChannelStatus::Open, right: ChannelStatus::PendingToClose(_) } => {
                                                    let _ = event_tx.broadcast_direct(ChainEvent::ChannelClosureInitiated(new.clone())).await;
                                                }
                                                ChannelChange::Status {left: ChannelStatus::PendingToClose(_), right: ChannelStatus::Closed} => {
                                                    let _ = event_tx.broadcast_direct(ChainEvent::ChannelClosed(new.clone())).await;
                                                }
                                                ChannelChange::Status { left: ChannelStatus::Closed, right: ChannelStatus::Open } => {
                                                    let _ = event_tx.broadcast_direct(ChainEvent::ChannelOpened(new.clone())).await;
                                                }
                                                ChannelChange::Balance { left, right } => {
                                                    if left > right {
                                                        let _ = event_tx.broadcast_direct(ChainEvent::ChannelBalanceDecreased(new.clone(), left - right)).await;
                                                    } else {
                                                        let _ = event_tx.broadcast_direct(ChainEvent::ChannelBalanceIncreased(new.clone(), right - left)).await;
                                                    }
                                                }
                                                // This way we track ticket redemptions on other channels as well.
                                                // Ticket redemption events on own channels must be tracked via redeem transaction tracking, as it
                                                // the event must also include the ticket that has been redeemed.
                                                ChannelChange::TicketIndex { .. }  if new.direction(&me).is_none() => {
                                                    let _ = event_tx.broadcast_direct(ChainEvent::TicketRedeemed(new.clone(), None)).await;
                                                }
                                                _ => {}
                                            }
                                        }
                                    },
                                    Ok((new, None))  => {
                                        tracing::trace!(id = %new.get_id(), "channel inserted");
                                        let _ = event_tx.broadcast_direct(ChainEvent::ChannelOpened(new.clone())).await;
                                    },
                                    Err(error) => {
                                        tracing::error!(%error, "error processing channel from subscription");
                                    }
                                }
                            }
                        })
                        .await;
                    tracing::warn!("channel subscription ended");
                }
                Err(error) => {
                    let _ = accounts_connected_tx.send(Err(error));
                },
            }
        });

        accounts_connected_rx
            .map_err(|_| ConnectorError::InvalidState("cannot notify account connection"))
            .and_then(|res| futures::future::ready(res.map_err(ConnectorError::from)))
            .await
    }

    pub async fn connect(&mut self) -> Result<(), ConnectorError> {
        self.disconnect();

        let (h1, h2) = futures::try_join!(self.connect_accounts(), self.connect_graph())?;
        self.connection_handles.push(h1);
        self.connection_handles.push(h2);

        Ok(())
    }

    pub fn disconnect(&mut self) {
        for handle in self.connection_handles.drain(..) {
            handle.abort();
        }
    }
}

