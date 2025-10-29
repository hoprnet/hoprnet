use std::time::Duration;
use blokli_client::{BlokliClient, BlokliClientConfig};
use blokli_client::api::BlokliSubscriptionClient;
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use petgraph::prelude::{DiGraphMap, StableDiGraph};
use hopr_api::chain::HoprKeyIdent;
use hopr_async_runtime::AbortHandle;
use hopr_chain_types::NetworkRegistryProxy::Safe;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::Address;
use crate::connector::keys::HoprKeyMapper;
use crate::connector::utils::{model_to_account_entry, model_to_graph_entry};
use crate::errors::ConnectorError;
use crate::payload::SafePayloadGenerator;

mod channels;
mod accounts;
mod keys;
mod utils;
mod values;

pub trait Backend {
    type Error: std::error::Error + Send + Sync + 'static;
    fn insert_account(&self, entry: AccountEntry) -> Result<(), Self::Error>;
    fn count_accounts(&self) -> Result<usize, Self::Error>;
    fn insert_channel(&self, channel: ChannelEntry) -> Result<(), Self::Error>;
    fn get_account_by_id(&self, id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error>;
    fn get_account_by_key(&self, key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error>;
    fn get_account_by_address(&self, chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error>;
    fn get_channel_by_id(&self, id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error>;
}

pub struct HoprBlockchainConnector<B, C> {
    payload_generator: SafePayloadGenerator,
    chain_key: ChainKeypair,
    safe_address: Address,
    client: std::sync::Arc<C>,
    graph: std::sync::Arc<parking_lot::RwLock<DiGraphMap<HoprKeyIdent, ChannelId, ahash::RandomState>>>,
    backend: std::sync::Arc<B>,
    connection_handles: Vec<AbortHandle>,

    // Caches
    mapper: keys::HoprKeyMapper<B>,
    chain_to_packet: moka::future::Cache<Address, Option<OffchainPublicKey>, ahash::RandomState>,
    packet_to_chain: moka::future::Cache<OffchainPublicKey, Option<Address>, ahash::RandomState>,
    channel_by_id: moka::future::Cache<ChannelId, Option<ChannelEntry>, ahash::RandomState>,
    channel_by_parties: moka::future::Cache<ChannelParties, Option<ChannelEntry>, ahash::RandomState>,
}

impl<B, C> HoprBlockchainConnector<B, C>
    where
        B: Backend + Send + Sync + 'static,
        C: BlokliSubscriptionClient + Send + Sync + 'static,
{
    pub fn new(chain_key: ChainKeypair, client: C, backend: B) -> Self {
        let backend = std::sync::Arc::new(backend);
        Self {
            payload_generator: SafePayloadGenerator::new(&chain_key, Default::default(), Default::default()),
            chain_key,
            safe_address: Default::default(),
            client: std::sync::Arc::new(client),
            graph: std::sync::Arc::new(parking_lot::RwLock::new(
                DiGraphMap::with_capacity_and_hasher(
                    10_000,
                    100_000,
                    ahash::RandomState::default()))
            ),
            backend: backend.clone(),
            connection_handles: Vec::with_capacity(2),
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
        }
    }

    async fn connect_accounts(&self) -> Result<AbortHandle, ConnectorError> {
        let (accounts_connected_tx, accounts_connected_rx) = futures::channel::oneshot::channel();
        let client = self.client.clone();
        let mapper = self.mapper.clone();
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
                                mapper.backend.insert_account(account)
                            })
                                .map_err(|e| ConnectorError::BackendError(e.into()))
                                .and_then(|res| futures::future::ready(res.map_err(|e| ConnectorError::BackendError(e.into()))))
                        })
                        .for_each(|res| {
                            match res {
                                Ok(_) => tracing::trace!("account inserted"),
                                Err(error) => {
                                    tracing::error!(%error, "error processing account from subscription");
                                }
                            }
                            futures::future::ready(())
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
                                backend.insert_channel(channel)
                            })
                                .map_err(|e| ConnectorError::BackendError(e.into()))
                                .and_then(|res| futures::future::ready(res.map_err(|e| ConnectorError::BackendError(e.into()))))
                        })
                        .for_each(|res| {
                            match res {
                                Ok(_) => tracing::trace!("channel inserted"),
                                Err(error) => {
                                    tracing::error!(%error, "error processing channel from subscription");
                                }
                            }
                            futures::future::ready(())
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

