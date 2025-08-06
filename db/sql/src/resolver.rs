use async_trait::async_trait;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_api::{
    errors::{DbError, Result},
    resolver::HoprDbResolverOperations,
};
use hopr_primitive_types::primitives::Address;

use crate::{accounts::HoprDbAccountOperations, db::HoprDb};

#[async_trait]
impl HoprDbResolverOperations for HoprDb {
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Result<Option<OffchainPublicKey>> {
        Ok(self
            .translate_key(None, *onchain_key)
            .await?
            .map(|k| k.try_into())
            .transpose()
            .map_err(|_e| DbError::LogicalError("failed to transpose the translated key".into()))?)
    }

    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Result<Option<Address>> {
        Ok(self
            .translate_key(None, *offchain_key)
            .await?
            .map(|k| k.try_into())
            .transpose()
            .map_err(|_e| DbError::LogicalError("failed to transpose the translated key".into()))?)
    }
}

#[cfg(test)]
mod tests {
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_internal_types::account::{AccountEntry, AccountType};
    use hopr_primitive_types::prelude::ToHex;
    use sea_orm::{EntityTrait, Set};

    use super::*;

    #[tokio::test]
    async fn test_get_offchain_key_should_return_nothing_if_a_mapping_to_chain_key_does_not_exist() -> anyhow::Result<()>
    {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain = ChainKeypair::random().public().to_address();

        let actual_pk = db.resolve_packet_key(&chain).await?;
        assert_eq!(actual_pk, None, "offchain key should not be present");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_chain_key_should_return_nothing_if_a_mapping_to_offchain_key_does_not_exist() -> anyhow::Result<()>
    {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let packet = *OffchainKeypair::random().public();

        let actual_ck = db.resolve_chain_key(&packet).await?;
        assert_eq!(actual_ck, None, "chain key should not be present");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_chain_key_should_succeed_if_a_mapping_to_offchain_key_exists() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        // Inserting to the table directly to avoid cache

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();
        let account_1 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_1.to_hex()),
            packet_key: Set(packet_1.to_hex()),
            ..Default::default()
        };

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = *OffchainKeypair::random().public();
        let account_2 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_2.to_hex()),
            packet_key: Set(packet_2.to_hex()),
            ..Default::default()
        };

        hopr_db_entity::account::Entity::insert_many([account_1, account_2])
            .exec(db.index_db.read_write())
            .await?;

        let actual_ck = db.resolve_chain_key(&packet_1).await?;
        assert_eq!(actual_ck, Some(chain_1), "chain keys must match");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_chain_key_should_succeed_if_a_mapping_to_offchain_key_exists_with_cache() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        // Inserting to the table via API to insert into cache as well

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();
        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                entry_type: AccountType::NotAnnounced,
                published_at: 1,
            },
        )
        .await?;

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = *OffchainKeypair::random().public();
        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_2,
                chain_addr: chain_2,
                entry_type: AccountType::NotAnnounced,
                published_at: 1,
            },
        )
        .await?;

        let actual_ck = db.resolve_chain_key(&packet_1).await?;
        assert_eq!(actual_ck, Some(chain_1), "chain keys must match");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_offchain_key_should_succeed_if_a_mapping_to_chain_key_exists() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        // Inserting to the table directly to avoid cache

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();
        let account_1 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_1.to_hex()),
            packet_key: Set(packet_1.to_hex()),
            ..Default::default()
        };

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = *OffchainKeypair::random().public();
        let account_2 = hopr_db_entity::account::ActiveModel {
            chain_key: Set(chain_2.to_hex()),
            packet_key: Set(packet_2.to_hex()),
            ..Default::default()
        };

        hopr_db_entity::account::Entity::insert_many([account_1, account_2])
            .exec(db.index_db.read_write())
            .await?;

        let actual_pk = db.resolve_packet_key(&chain_2).await?;

        assert_eq!(actual_pk, Some(packet_2), "packet keys must match");
        Ok(())
    }

    #[tokio::test]
    async fn test_get_offchain_key_should_succeed_if_a_mapping_to_chain_key_exists_with_cache() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        // Inserting to the table via API to insert into cache as well

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();
        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                entry_type: AccountType::NotAnnounced,
                published_at: 1,
            },
        )
        .await?;

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = *OffchainKeypair::random().public();
        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_2,
                chain_addr: chain_2,
                entry_type: AccountType::NotAnnounced,
                published_at: 1,
            },
        )
        .await?;

        let actual_pk = db.resolve_packet_key(&chain_2).await?;

        assert_eq!(actual_pk, Some(packet_2), "packet keys must match");
        Ok(())
    }
}
