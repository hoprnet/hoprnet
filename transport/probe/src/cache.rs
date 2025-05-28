use async_trait::async_trait;
use hopr_db_api::{protocol::HoprDbProtocolOperations, resolver::HoprDbResolverOperations};

use crate::traits::CacheOperations;

#[derive(Debug, Clone)]
pub struct CacheProxy<T>
where
    T: HoprDbResolverOperations + HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    db: T,
}

impl<T> CacheProxy<T>
where
    T: HoprDbResolverOperations + HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    pub fn new(db: T) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<T> CacheOperations for CacheProxy<T>
where
    T: HoprDbResolverOperations + HoprDbProtocolOperations + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    async fn find_surb(
        &self,
        matcher: hopr_network_types::types::SurbMatcher,
    ) -> hopr_db_api::errors::Result<(hopr_db_api::protocol::HoprSenderId, hopr_db_api::protocol::HoprSurb)> {
        self.db.find_surb(matcher).await
    }

    async fn resolve_chain_key(
        &self,
        offchain_key: &hopr_crypto_types::types::OffchainPublicKey,
    ) -> hopr_db_api::errors::Result<Option<hopr_primitive_types::prelude::Address>> {
        self.db.resolve_chain_key(offchain_key).await
    }
}
