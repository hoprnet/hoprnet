use blokli_client::{BlokliClient, BlokliClientConfig};
use petgraph::prelude::StableDiGraph;
use hopr_api::chain::{HoprKeyIdent, HoprSphinxHeaderSpec, HoprSphinxSuite};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{Address, KeyIdent};
use crate::connector::backend::Backend;
use crate::payload::SafePayloadGenerator;

mod channels;
mod accounts;
mod keys;
mod utils;
mod backend;

pub struct HoprBlockchainConnectorConfig {
    blokli_config: BlokliClientConfig,
}

pub struct HoprBlockchainConnector<B> {
    payload_generator: SafePayloadGenerator,
    chain_key: ChainKeypair,
    safe_address: Address,
    client: BlokliClient,
    graph: std::sync::Arc<parking_lot::Mutex<StableDiGraph<HoprKeyIdent, ChannelId>>>,
    backend: std::sync::Arc<B>,

    // Caches
    mapper: keys::HoprKeyMapper<B>,
    chain_to_packet: moka::future::Cache<Address, Option<OffchainPublicKey>>,
    packet_to_chain: moka::future::Cache<OffchainPublicKey, Option<Address>>,
    channel_by_id: moka::future::Cache<ChannelId, Option<ChannelEntry>>,
    channel_by_parties: moka::future::Cache<ChannelParties, Option<ChannelEntry>>,
}

impl<B: Backend> HoprBlockchainConnector<B> {
    pub fn new(blokli_url: url::Url, backend: B, cfg: HoprBlockchainConnectorConfig) -> Self {
        todo!()
    }
}

