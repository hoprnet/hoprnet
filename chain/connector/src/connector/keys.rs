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

// These lookups run synchronously on Rayon threads (called from `HoprPacket::from_incoming`
// inside `spawn_fifo_blocking`). The moka::sync::Cache uses per-key locks, so different keys
// don't block each other, but the init closure performs synchronous redb I/O on cache miss.
// If redb is slow (disk contention, large file, swap pressure), a cache miss on a Rayon thread
// can block for tens of milliseconds, contributing to PACKET_DECODING_TIMEOUT breaches.
// The elapsed_ms timing in each init closure makes this visible in structured logs.
impl<B> hopr_api::chain::KeyIdMapper<HoprSphinxSuite, HoprSphinxHeaderSpec> for HoprKeyMapper<B>
where
    B: Backend + Send + Sync + 'static,
{
    fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
        self.key_to_id.get_with_by_ref(key, || {
            let start = std::time::Instant::now();
            tracing::warn!(%key, "cache miss on map_key_to_id");
            let result = match self.backend.get_account_by_key(key) {
                Ok(Some(account)) => Some(account.key_id),
                Ok(None) => None,
                Err(error) => {
                    tracing::error!(%error, %key, "failed to get account by key");
                    None
                }
            };
            tracing::trace!(
                %key,
                elapsed_ms = start.elapsed().as_millis() as u64,
                found = result.is_some(),
                "map_key_to_id backend lookup"
            );
            result
        })
    }

    fn map_id_to_public(&self, id: &HoprKeyIdent) -> Option<OffchainPublicKey> {
        self.id_to_key.get_with_by_ref(id, || {
            let start = std::time::Instant::now();
            tracing::warn!(%id, "cache miss on map_id_to_public");
            let result = match self.backend.get_account_by_id(id) {
                Ok(Some(account)) => Some(account.public_key),
                Ok(None) => None,
                Err(error) => {
                    tracing::error!(%error, %id, "failed to get account by id");
                    None
                }
            };
            tracing::trace!(
                %id,
                elapsed_ms = start.elapsed().as_millis() as u64,
                found = result.is_some(),
                "map_id_to_public backend lookup"
            );
            result
        })
    }
}

#[async_trait::async_trait]
impl<B, C, P, R> hopr_api::chain::ChainKeyOperations for HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
    C: Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
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
                    Ok(Err(e)) => Err(ConnectorError::backend(e)),
                    Err(e) => Err(ConnectorError::backend(e)),
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
                tracing::warn!(
                    peer_id = packet.to_peerid_str(),
                    "cache miss on packet_key_to_chain_key"
                );
                match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_account_by_key(&packet_key)).await
                {
                    Ok(Ok(value)) => Ok(value.map(|account| account.chain_addr)),
                    Ok(Err(e)) => Err(ConnectorError::backend(e)),
                    Err(e) => Err(ConnectorError::backend(e)),
                }
            })
            .await?)
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        &self.mapper
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_api::chain::{ChainKeyOperations, KeyIdMapper};
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use crate::{connector::tests::create_connector, testing::BlokliTestStateBuilder};

    #[tokio::test]
    async fn connector_should_map_keys_to_ids_and_back() -> anyhow::Result<()> {
        let offchain_key = OffchainKeypair::from_secret(&hex!(
            "60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"
        ))?;
        let chain_addr: Address = [1u8; Address::SIZE].into();
        let account = AccountEntry {
            public_key: *offchain_key.public(),
            chain_addr,
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(account.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1))])
            .build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        assert_eq!(
            Some(chain_addr),
            connector.packet_key_to_chain_key(&offchain_key.public()).await?
        );
        assert_eq!(
            Some(*offchain_key.public()),
            connector.chain_key_to_packet_key(&chain_addr).await?
        );

        let mapper = connector.key_id_mapper_ref();

        assert_eq!(Some(account.key_id), mapper.map_key_to_id(&offchain_key.public()));
        assert_eq!(Some(*offchain_key.public()), mapper.map_id_to_public(&account.key_id));

        Ok(())
    }
}
