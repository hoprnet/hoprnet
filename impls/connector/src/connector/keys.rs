use hopr_api::{
    chain::{HoprKeyIdent, KeyIdMapping},
    types::{crypto::prelude::OffchainPublicKey, primitive::prelude::Address},
};

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
// inside `spawn_fifo_blocking`). The elapsed_ms timing in each init closure makes the rayon
// execution time visible in structured logs.
impl<B> KeyIdMapping<HoprKeyIdent, OffchainPublicKey> for HoprKeyMapper<B>
where
    B: Backend + Send + Sync + 'static,
{
    fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
        self.key_to_id.get_with_by_ref(key, || {
            let start = std::time::Instant::now();
            tracing::debug!(%key, "cache miss on map_key_to_id");
            let result = match self.backend.get_account_by_key(key) {
                Ok(Some(account)) => Some(account.key_id),
                Ok(None) => None,
                Err(error) => {
                    tracing::warn!(%error, %key, "failed to get account by key");
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
            tracing::debug!(%id, "cache miss on map_id_to_public");
            let result = match self.backend.get_account_by_id(id) {
                Ok(Some(account)) => Some(account.public_key),
                Ok(None) => None,
                Err(error) => {
                    tracing::warn!(%error, %id, "failed to get account by id");
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

    fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
        self.check_connection_state()?;

        Ok(self.chain_to_packet.try_get_with_by_ref(chain, || {
            tracing::debug!(%chain, "cache miss on chain_key_to_packet_key");
            self.backend
                .get_account_by_address(chain)
                .map(|a| a.map(|ac| ac.public_key))
                .map_err(ConnectorError::backend)
        })?)
    }

    fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
        self.check_connection_state()?;

        Ok(self.packet_to_chain.try_get_with_by_ref(packet, || {
            tracing::debug!(
                peer_id = packet.to_peerid_str(),
                "cache miss on packet_key_to_chain_key"
            );
            self.backend
                .get_account_by_key(packet)
                .map(|a| a.map(|ac| ac.chain_addr))
                .map_err(ConnectorError::backend)
        })?)
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        &self.mapper
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_api::{
        chain::{ChainKeyOperations, HoprKeyIdent, KeyIdMapping},
        types::{crypto::prelude::*, internal::prelude::*, primitive::prelude::*},
    };

    use crate::{
        backend::Backend, connector::tests::create_connector, errors::ConnectorError, testing::BlokliTestStateBuilder,
    };

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
            connector.packet_key_to_chain_key(offchain_key.public())?
        );
        assert_eq!(
            Some(*offchain_key.public()),
            connector.chain_key_to_packet_key(&chain_addr)?
        );

        let mapper = connector.key_id_mapper_ref();

        assert_eq!(Some(account.key_id), mapper.map_key_to_id(offchain_key.public()));
        assert_eq!(Some(*offchain_key.public()), mapper.map_id_to_public(&account.key_id));

        Ok(())
    }

    struct MockErrorBackend;

    impl Backend for MockErrorBackend {
        type Error = ConnectorError;

        fn insert_account(&self, _entry: AccountEntry) -> Result<Option<AccountEntry>, Self::Error> {
            Err(ConnectorError::InvalidState("mock error"))
        }

        fn insert_channel(&self, _channel: ChannelEntry) -> Result<Option<ChannelEntry>, Self::Error> {
            Err(ConnectorError::InvalidState("mock error"))
        }

        fn get_account_by_id(&self, _id: &HoprKeyIdent) -> Result<Option<AccountEntry>, Self::Error> {
            Err(ConnectorError::InvalidState("mock error"))
        }

        fn get_account_by_key(&self, _key: &OffchainPublicKey) -> Result<Option<AccountEntry>, Self::Error> {
            Err(ConnectorError::InvalidState("mock error"))
        }

        fn get_account_by_address(&self, _chain_key: &Address) -> Result<Option<AccountEntry>, Self::Error> {
            Err(ConnectorError::InvalidState("mock error"))
        }

        fn get_channel_by_id(&self, _id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
            Err(ConnectorError::InvalidState("mock error"))
        }
    }

    #[test]
    fn mapper_should_handle_backend_errors() {
        let mapper = super::HoprKeyMapper {
            id_to_key: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            key_to_id: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            backend: std::sync::Arc::new(MockErrorBackend),
        };

        let key = *OffchainKeypair::random().public();
        let id = HoprKeyIdent::from(1);

        assert_eq!(None, mapper.map_key_to_id(&key));
        assert_eq!(None, mapper.map_id_to_public(&id));
    }

    #[test]
    fn mapper_should_handle_missing_accounts() {
        use crate::InMemoryBackend;
        let mapper = super::HoprKeyMapper {
            id_to_key: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            key_to_id: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            backend: std::sync::Arc::new(InMemoryBackend::default()),
        };

        let key = *OffchainKeypair::random().public();
        let id = HoprKeyIdent::from(1);

        assert_eq!(None, mapper.map_key_to_id(&key));
        assert_eq!(None, mapper.map_id_to_public(&id));
    }

    #[tokio::test]
    async fn connector_should_check_connection_state() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default().build_static_client();
        let connector = create_connector(blokli_client)?;

        let key = *OffchainKeypair::random().public();
        let addr = Address::from([0u8; 20]);

        assert!(connector.packet_key_to_chain_key(&key).is_err());
        assert!(connector.chain_key_to_packet_key(&addr).is_err());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_handle_backend_errors() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default().build_static_client();
        let connector = create_connector(blokli_client)?;

        // Instead of manual struct initialization, use what we have and inject the error backend
        let mapper = super::HoprKeyMapper {
            id_to_key: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            key_to_id: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            backend: std::sync::Arc::new(MockErrorBackend),
        };

        let mut connector = crate::connector::HoprBlockchainConnector::new(
            connector.chain_key.clone(),
            connector.cfg,
            (*connector.client).clone(),
            MockErrorBackend,
            connector.payload_generator,
        );
        connector.mapper = mapper;
        connector.connect().await?;

        let key = *OffchainKeypair::random().public();
        let addr = Address::from([0u8; 20]);

        assert!(connector.packet_key_to_chain_key(&key).is_err());
        assert!(connector.chain_key_to_packet_key(&addr).is_err());

        Ok(())
    }

    // Regression test: address caches (chain_to_packet, packet_to_chain) must be populated
    // from the subscription stream regardless of whether backend.insert_account succeeds.
    // Previously these were only updated inside `if let Ok(...) = &res { ... }`, so a transient
    // backend failure left the caches empty.  packet_key_to_chain_key would then immediately
    // cache None from the (also-failing) backend fallback and the mapping was permanently lost.
    #[tokio::test]
    async fn connector_address_caches_populated_from_subscription_despite_backend_insert_failure() -> anyhow::Result<()>
    {
        use crate::connector::BlockchainConnectorConfig;

        let client_keypair = OffchainKeypair::random();
        let client_chain_addr = Address::from([42u8; Address::SIZE]);
        let client_account = AccountEntry {
            public_key: *client_keypair.public(),
            chain_addr: client_chain_addr,
            entry_type: AccountType::NotAnnounced,
            safe_address: None,
            key_id: 1.into(),
        };

        // Pre-load the client account so the subscription delivers it on connect.
        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(client_account, HoprBalance::new_base(0), XDaiBalance::new_base(0))])
            .build_static_client();

        let connector = create_connector(blokli_client)?;

        // sync_tolerance = 0: connect() returns immediately (min_accounts = 0) so the
        // backend-failing account event is processed asynchronously after connect() returns.
        let cfg = BlockchainConnectorConfig {
            sync_tolerance: 0,
            ..connector.cfg
        };
        let mapper = super::HoprKeyMapper {
            id_to_key: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            key_to_id: moka::sync::Cache::builder()
                .max_capacity(10)
                .build_with_hasher(ahash::RandomState::default()),
            backend: std::sync::Arc::new(MockErrorBackend),
        };
        let mut connector = crate::connector::HoprBlockchainConnector::new(
            connector.chain_key.clone(),
            cfg,
            (*connector.client).clone(),
            MockErrorBackend,
            connector.payload_generator,
        );
        connector.mapper = mapper;
        connector.connect().await?;

        // Wait until the subscription loop has populated both address caches.
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let packet_to_chain_ready =
                    connector.packet_key_to_chain_key(client_keypair.public()).ok() == Some(Some(client_chain_addr));
                let chain_to_packet_ready =
                    connector.chain_key_to_packet_key(&client_chain_addr).ok() == Some(Some(*client_keypair.public()));
                if packet_to_chain_ready && chain_to_packet_ready {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await?;

        // The address caches must be populated even though insert_account failed.
        assert_eq!(
            Some(client_chain_addr),
            connector.packet_key_to_chain_key(client_keypair.public())?
        );
        assert_eq!(
            Some(*client_keypair.public()),
            connector.chain_key_to_packet_key(&client_chain_addr)?
        );

        Ok(())
    }
}
