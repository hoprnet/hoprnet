//! This Rust crate contains all the path construction and path selection algorithms in the HOPR mixnet.

/// Defines the graph of HOPR payment channels.
pub mod channel_graph;
pub mod errors;
/// Defines the two most important types: [TransportPath](crate::path::TransportPath) and [ChannelPath](crate::path::ChannelPath).
pub mod path;
/// Implements different path selectors in the [ChannelGraph](crate::channel_graph::ChannelGraph).
pub mod selectors;

use async_lock::RwLock;
use async_trait::async_trait;
use chain_db::db::CoreEthereumDb;
use chain_db::traits::HoprCoreEthereumDbActions;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::protocol::PeerAddressResolver;
use hopr_primitive_types::primitives::Address;
use tracing::error;
use std::sync::Arc;
use utils_db::CurrentDbShim;

// TODO: use internally a LRU cache
/// DB backed packet key to chain key resolver
#[derive(Debug, Clone)]
pub struct DbPeerAddressResolver(pub Arc<RwLock<CoreEthereumDb<CurrentDbShim>>>);

#[async_trait]
impl PeerAddressResolver for DbPeerAddressResolver {
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
        self.0
            .read()
            .await
            .get_packet_key(onchain_key)
            .await
            .unwrap_or_else(|e| {
                error!("failed to resolve packet key for {onchain_key}: {e}");
                None
            })
    }

    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
        self.0
            .read()
            .await
            .get_chain_key(offchain_key)
            .await
            .unwrap_or_else(|e| {
                error!("failed to resolve chain key for {offchain_key}: {e}");
                None
            })
    }
}
