use async_trait::async_trait;
use hopr_api::chain::ChainKeyOperations;
use hopr_transport_protocol::{FoundSurb, SurbStore};

use crate::traits::DbOperations;

#[derive(Debug, Clone)]
pub struct DbProxy<T, R> {
    db: T,
    resolver: R,
}

impl<T, R> DbProxy<T, R> {
    pub fn new(db: T, resolver: R) -> Self {
        Self { db, resolver }
    }
}

#[async_trait]
impl<T, R> DbOperations for DbProxy<T, R>
where
    T: SurbStore + Clone + Send + Sync + 'static,
    R: ChainKeyOperations + Clone + Send + Sync + 'static,
{
    type ChainError = R::Error;

    async fn find_surb(&self, matcher: hopr_network_types::types::SurbMatcher) -> Option<FoundSurb> {
        tracing::trace!(target: "db_proxy", ?matcher, "finding SURB with matcher");
        self.db.find_surb(matcher).await
    }

    async fn resolve_chain_key(
        &self,
        offchain_key: &hopr_crypto_types::types::OffchainPublicKey,
    ) -> Result<Option<hopr_primitive_types::prelude::Address>, R::Error> {
        self.resolver.packet_key_to_chain_key(offchain_key).await
    }
}
