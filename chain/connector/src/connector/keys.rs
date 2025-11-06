use hopr_api::chain::{HoprKeyIdent, HoprSphinxHeaderSpec, HoprSphinxSuite};
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_primitive_types::prelude::Address;

use crate::{backend::Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

pub struct HoprKeyMapper<B> {
    pub(crate) id_to_key: moka::sync::Cache<HoprKeyIdent, Option<OffchainPublicKey>, ahash::RandomState>,
    pub(crate) key_to_id: moka::sync::Cache<OffchainPublicKey, Option<HoprKeyIdent>, ahash::RandomState>,
    pub(crate) backend: std::sync::Arc<B>,
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
    B: Backend + Send + Sync + 'static,
{
    fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
        self.key_to_id.get_with_by_ref(key, || {
            tracing::warn!(%key, "cache miss on map_key_to_id");
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
            tracing::warn!(%id, "cache miss on map_id_to_public");
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
impl<B, C, P> hopr_api::chain::ChainKeyOperations for HoprBlockchainConnector<C, B, P>
where
    B: Backend + Send + Sync + 'static,
    C: Send + Sync,
    P: Send + Sync,
{
    type Error = ConnectorError;
    type Mapper = HoprKeyMapper<B>;

    async fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
        self.check_connection_state()?;

        let backend = self.backend.clone();
        let chain_key = *chain;
        Ok(self
            .chain_to_packet
            .try_get_with_by_ref(&chain_key, async move {
                tracing::warn!(%chain_key, "cache miss on chain_key_to_packet_key");
                match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_account_by_address(&chain_key))
                    .await
                {
                    Ok(Ok(value)) => Ok(value.map(|account| account.public_key)),
                    Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                    Err(e) => Err(ConnectorError::BackendError(e.into())),
                }
            })
            .await?)
    }

    async fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
        self.check_connection_state()?;

        let backend = self.backend.clone();
        let packet_key = *packet;
        Ok(self
            .packet_to_chain
            .try_get_with_by_ref(&packet_key, async move {
                tracing::warn!(%packet_key, "cache miss on packet_key_to_chain_key");
                match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_account_by_key(&packet_key)).await
                {
                    Ok(Ok(value)) => Ok(value.map(|account| account.chain_addr)),
                    Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                    Err(e) => Err(ConnectorError::BackendError(e.into())),
                }
            })
            .await?)
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        &self.mapper
    }
}
