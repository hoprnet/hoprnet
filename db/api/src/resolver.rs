use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_primitive_types::{primitives::Address, traits::ToHex};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{db::HoprDb, errors::Result};

/// Trait for linking and resolving the corresponding `OffchainPublicKey` and on-chain `Address`.
#[async_trait]
pub trait HoprDbResolverOperations {
    /// Tries to resolve off-chain public key given the on-chain address
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Result<Option<OffchainPublicKey>>;

    /// Tries to resolve on-chain public key given the off-chain public key
    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Result<Option<Address>>;
}

#[async_trait]
impl HoprDbResolverOperations for HoprDb {
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Result<Option<OffchainPublicKey>> {
        let packet_key = hopr_db_entity::account::Entity::find()
            .filter(hopr_db_entity::account::Column::ChainKey.eq(onchain_key.to_hex()))
            .one(&self.db)
            .await?
            .map(|model| {
                OffchainPublicKey::from_hex(&model.packet_key).map_err(|_| crate::errors::DbError::DecodingError)
            });

        if let Some(packet_key) = packet_key {
            Ok(Some(packet_key?))
        } else {
            Ok(None)
        }
    }

    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Result<Option<Address>> {
        let chain_key = hopr_db_entity::account::Entity::find()
            .filter(hopr_db_entity::account::Column::PacketKey.eq(offchain_key.to_string()))
            .one(&self.db)
            .await?
            .map(|model| Address::from_hex(&model.chain_key).map_err(|_| crate::errors::DbError::DecodingError));

        if let Some(chain_key) = chain_key {
            Ok(Some(chain_key?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
    use sea_orm::Set;

    #[async_std::test]
    async fn test_get_offchain_key_should_return_nothing_if_a_mapping_to_chain_key_does_not_exist() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain = ChainKeypair::random().public().to_address();

        let actual_pk = db.resolve_packet_key(&chain).await.expect("must succeed");
        assert_eq!(actual_pk, None, "offchain key should not be present");
    }

    #[async_std::test]
    async fn test_get_chain_key_should_return_nothing_if_a_mapping_to_offchain_key_does_not_exist() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let packet = OffchainKeypair::random().public().clone();

        let actual_ck = db.resolve_chain_key(&packet).await.expect("must succeed");
        assert_eq!(actual_ck, None, "chain key should not be present");
    }

    #[async_std::test]
    async fn test_get_chain_key_should_succeed_if_a_mapping_to_offchain_key_exists() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = OffchainKeypair::random().public().clone();
        let account_1 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_1.to_hex()),
            packet_key: Set(packet_1.to_hex()),
            ..Default::default()
        };

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = OffchainKeypair::random().public().clone();
        let account_2 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_2.to_hex()),
            packet_key: Set(packet_2.to_hex()),
            ..Default::default()
        };

        hopr_db_entity::account::Entity::insert_many([account_1, account_2])
            .exec(&db.db)
            .await
            .expect("insert must succeed");

        let actual_ck = db.resolve_chain_key(&packet_1).await.expect("must succeed");
        assert_eq!(actual_ck, Some(chain_1), "chain keys must match");
    }

    #[async_std::test]
    async fn test_get_offchain_key_should_succeed_if_a_mapping_to_chain_key_exists() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = OffchainKeypair::random().public().clone();
        let account_1 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_1.to_hex()),
            packet_key: Set(packet_1.to_hex()),
            ..Default::default()
        };

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = OffchainKeypair::random().public().clone();
        let account_2 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_2.to_hex()),
            packet_key: Set(packet_2.to_hex()),
            ..Default::default()
        };

        hopr_db_entity::account::Entity::insert_many([account_1, account_2])
            .exec(&db.db)
            .await
            .expect("insert must succeed");

        let actual_pk = db.resolve_packet_key(&chain_2).await.expect("must succeed");

        assert_eq!(actual_pk, Some(packet_2), "packet keys must match");
    }
}
