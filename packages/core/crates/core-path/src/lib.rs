pub mod channel_graph;
pub mod errors;
pub mod path;
pub mod selectors;

use async_std::sync::RwLock;
use async_trait::async_trait;
use core_crypto::types::OffchainPublicKey;
use core_ethereum_db::db::CoreEthereumDb;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::protocol::PeerAddressResolver;
use std::sync::Arc;
use utils_db::rusty::RustyLevelDbShim;
use utils_log::error;
use utils_types::primitives::Address;

/// DB backed packet key to chain key resolver
#[derive(Debug, Clone)]
pub struct DbPeerAddressResolver(pub Arc<RwLock<CoreEthereumDb<RustyLevelDbShim>>>);

#[async_trait(? Send)]
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