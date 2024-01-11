pub mod channel_graph;
pub mod errors;
pub mod path;
pub mod selectors;

use async_lock::RwLock;
use async_trait::async_trait;
use hopr_crypto::types::OffchainPublicKey;
use chain_db::db::CoreEthereumDb;
use chain_db::traits::HoprCoreEthereumDbActions;
use core_types::protocol::PeerAddressResolver;
use log::error;
use std::sync::Arc;
use utils_db::CurrentDbShim;
use utils_types::primitives::Address;

/// DB backed packet key to chain key resolver
#[derive(Debug, Clone)]
pub struct DbPeerAddressResolver(pub Arc<RwLock<CoreEthereumDb<CurrentDbShim>>>);

#[async_trait]
impl PeerAddressResolver for DbPeerAddressResolver {
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
        match self.0.read().await.get_packet_key(onchain_key).await {
            Ok(k) => k,
            Err(e) => {
                error!("failed to resolve packet key for {onchain_key}: {e}");
                None
            }
        }
    }

    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
        match self.0.read().await.get_chain_key(offchain_key).await {
            Ok(k) => k,
            Err(e) => {
                error!("failed to resolve chain key for {offchain_key}: {e}");
                None
            }
        }
    }
}
