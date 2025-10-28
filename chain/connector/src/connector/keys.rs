use hopr_api::chain::{HoprKeyIdent, HoprSphinxHeaderSpec, HoprSphinxSuite};
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_primitive_types::prelude::Address;

use crate::connector::{HoprBlockchainConnector};
use crate::connector::backend::Backend;
use crate::errors::ConnectorError;

pub struct HoprKeyMapper<B> {
    id_to_key: moka::sync::Cache<HoprKeyIdent, Option<OffchainPublicKey>>,
    key_to_id: moka::sync::Cache<OffchainPublicKey, Option<HoprKeyIdent>>,
    backend: std::sync::Arc<B>,
}

impl<B> Clone for HoprKeyMapper<B> {
    fn clone(&self) -> Self {
        Self {
            id_to_key: self.id_to_key.clone(),
            key_to_id: self.key_to_id.clone(),
            backend: self.backend.clone(),
        }
    }
}

impl<B> hopr_api::chain::KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec> for HoprKeyMapper<B>
where
    B: Backend + Send + Sync + 'static {
    fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
        self.key_to_id.get_with_by_ref(key, || {
            match self.backend.get_account_by_key(key) {
                Ok(Some(account)) => Some(account.key_id),
                Ok(None) => None,
                Err(error) => {
                    tracing::error!(%error, %key, "failed to get account by key");
                    None
                }
            }
        })
    }

    fn map_id_to_public(&self, id: &HoprKeyIdent) -> Option<OffchainPublicKey> {
        self.id_to_key.get_with_by_ref(id, || {
            match self.backend.get_account_by_id(id) {
                Ok(Some(account)) => Some(account.public_key),
                Ok(None) => None,
                Err(error) => {
                    tracing::error!(%error, %id, "failed to get account by id");
                    None
                }
            }
        })
    }
}


#[async_trait::async_trait]
impl<B> hopr_api::chain::ChainKeyOperations for HoprBlockchainConnector<B>
where
    B: Backend + Send + Sync + 'static {
    type Error = ConnectorError;
    type Mapper = HoprKeyMapper<B>;

    async fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
        let backend = self.backend.clone();
        let chain = *chain;
        Ok(self.chain_to_packet.try_get_with_by_ref(&chain, async move {
            match hopr_async_runtime::prelude::spawn_blocking(move || {
                backend.get_account_by_address(&chain)
            }).await {
                Ok(Ok(value)) => Ok(value.map(|account| account.public_key)),
                Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                Err(e) => Err(ConnectorError::BackendError(e.into())),
            }
        })?)
    }

    async fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
        let backend = self.backend.clone();
        let packet = *packet;
        Ok(self.packet_to_chain.try_get_with_by_ref(&packet, async move {
            match hopr_async_runtime::prelude::spawn_blocking(move || {
                backend.get_account_by_key(&packet)
            }).await {
                Ok(Ok(value)) => Ok(value.map(|account| account.chain_addr)),
                Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                Err(e) => Err(ConnectorError::BackendError(e.into())),
            }
        })?)
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        &self.mapper
    }
}