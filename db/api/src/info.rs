use async_trait::async_trait;
use futures::TryFutureExt;
use hopr_crypto_types::prelude::Hash;
use hopr_db_entity::{chain_info, global_settings, node_info};
use hopr_primitive_types::prelude::{Address, Balance, BalanceType, BinarySerializable, IntoEndian, ToHex};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

use crate::db::HoprDb;

use crate::cache::{CachedValue, CachedValueDiscriminants};
use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx, SINGULAR_TABLE_FIXED_ID};

/// Contains various on-chain information collected by Indexer,
/// such as domain separators, ticket price, Network Registry status...etc.
/// All these members change very rarely and therefore can be cached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IndexerData {
    /// Ledger smart contract domain separator
    pub ledger_dst: Option<Hash>,
    /// Node safe registry smart contract domain separator
    pub safe_registry_dst: Option<Hash>,
    /// Channels smart contract domain separator
    pub channels_dst: Option<Hash>,
    /// Current ticket price
    pub ticket_price: Option<Balance>,
    /// Network registry state
    pub nr_enabled: bool,
}

/// Contains information about node's safe.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SafeInfo {
    /// Safe address
    pub safe_address: Address,
    /// Safe module address.
    pub module_address: Address,
}

/// Enumerates different domain separators
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DomainSeparator {
    /// Ledger smart contract domain separator
    Ledger,
    /// Node safe registry smart contract domain separator
    SafeRegistry,
    /// Channels smart contract domain separator
    Channel,
}

/// Defines DB access API for various node information.
#[async_trait]
pub trait HoprDbInfoOperations {
    /// Gets node's Safe balance.
    async fn get_safe_balance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance>;

    /// Sets node's Safe balance.
    async fn set_safe_balance<'a>(&'a self, tx: OptTx<'a>, new_balance: Balance) -> Result<()>;

    /// Gets node's Safe allowance.
    async fn get_safe_allowance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance>;

    /// Sets node's Safe allowance.
    async fn set_safe_allowance<'a>(&'a self, tx: OptTx<'a>, new_allowance: Balance) -> Result<()>;

    /// Gets node's Safe addresses info.
    async fn get_safe_info<'a>(&'a self, tx: OptTx<'a>) -> Result<Option<SafeInfo>>;

    /// Sets node's Safe addresses info.
    async fn set_safe_info<'a>(&'a self, tx: OptTx<'a>, safe_info: SafeInfo) -> Result<()>;

    /// Gets stored Indexer data (either from the cache or from the DB).
    ///
    /// To update information stored in [IndexerData], use the individual setter methods,
    /// such as [`HoprDbInfoOperations::set_domain_separator`]... etc.
    async fn get_indexer_data<'a>(&'a self, tx: OptTx<'a>) -> Result<IndexerData>;

    /// Sets a domain separator.
    ///
    /// To retrieve stored domain separator info, use [`HoprDbInfoOperations::get_indexer_data`],
    /// note that this setter should invalidate the cache.
    async fn set_domain_separator<'a>(&'a self, tx: OptTx<'a>, dst_type: DomainSeparator, value: Hash) -> Result<()>;

    /// Updates the ticket price.
    /// To retrieve stored ticket price, use [`HoprDbInfoOperations::get_indexer_data`],
    /// note that this setter should invalidate the cache.
    async fn update_ticket_price<'a>(&'a self, tx: OptTx<'a>, price: Balance) -> Result<()>;

    /// Retrieves the last indexed block number.
    async fn get_last_indexed_block<'a>(&'a self, tx: OptTx<'a>) -> Result<u32>;

    /// Updates the last indexed block number.
    async fn set_last_indexed_block<'a>(&'a self, tx: OptTx<'a>, block_num: u32) -> Result<()>;

    /// Updates the network registry state.
    /// To retrieve stored network registry state, use [`HoprDbInfoOperations::get_indexer_data`],
    /// note that this setter should invalidate the cache.
    async fn set_network_registry_enabled<'a>(&'a self, tx: OptTx<'a>, enabled: bool) -> Result<()>;

    /// Gets global setting value with the given key.
    async fn get_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str) -> Result<Option<Box<[u8]>>>;

    /// Sets the global setting value with the given key.
    ///
    /// If setting with the given `key` does not exist, it is created.
    /// /// If setting with the given `key` exists, it is created.
    /// If `value` is `None` and setting with the given `key` exists it is removed.
    async fn set_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str, value: Option<&[u8]>) -> Result<()>;
}

#[async_trait]
impl HoprDbInfoOperations for HoprDb {
    async fn get_safe_balance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbError::MissingFixedTableEntry("node_info".into()))
                        .map(|m| BalanceType::HOPR.balance_bytes(m.safe_balance))
                })
            })
            .await
    }

    async fn set_safe_balance<'a>(&'a self, tx: OptTx<'a>, new_balance: Balance) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        node_info::ActiveModel {
                            id: Set(SINGULAR_TABLE_FIXED_ID),
                            safe_balance: Set(new_balance.amount().to_be_bytes().into()),
                            ..Default::default()
                        }
                        .update(tx.as_ref())
                        .await?,
                    )
                })
            })
            .await?;

        Ok(())
    }

    async fn get_safe_allowance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbError::MissingFixedTableEntry("node_info".into()))
                        .map(|m| BalanceType::HOPR.balance_bytes(m.safe_allowance))
                })
            })
            .await
    }

    async fn set_safe_allowance<'a>(&'a self, tx: OptTx<'a>, new_allowance: Balance) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        safe_allowance: Set(new_allowance.amount().to_be_bytes().to_vec()),
                        ..Default::default()
                    }
                    .save(tx.as_ref())
                    .await?;

                    Ok::<_, DbError>(())
                })
            })
            .await
    }

    async fn get_safe_info<'a>(&'a self, tx: OptTx<'a>) -> Result<Option<SafeInfo>> {
        let myself = self.clone();
        Ok(self
            .caches
            .single_values
            .try_get_with_by_ref(&CachedValueDiscriminants::SafeInfoCache, async move {
                myself
                    .nest_transaction(tx)
                    .and_then(|op| {
                        op.perform(|tx| {
                            Box::pin(async move {
                                let info = node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                                    .one(tx.as_ref())
                                    .await?
                                    .ok_or(DbError::MissingFixedTableEntry("node_info".into()))?;
                                Ok::<_, DbError>(info.safe_address.zip(info.module_address))
                            })
                        })
                    })
                    .await
                    .and_then(|addrs| {
                        if let Some((safe_address, module_address)) = addrs {
                            Ok(Some(SafeInfo {
                                safe_address: safe_address.parse()?,
                                module_address: module_address.parse()?,
                            }))
                        } else {
                            Ok(None)
                        }
                    })
                    .map(CachedValue::SafeInfoCache)
            })
            .await?
            .try_into()?)
    }

    async fn set_safe_info<'a>(&'a self, tx: OptTx<'a>, safe_info: SafeInfo) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        safe_address: Set(Some(safe_info.safe_address.to_hex())),
                        module_address: Set(Some(safe_info.module_address.to_hex())),
                        ..Default::default()
                    }
                    .save(tx.as_ref())
                    .await?;
                    Ok::<_, DbError>(())
                })
            })
            .await?;
        self.caches
            .single_values
            .insert(
                CachedValueDiscriminants::SafeInfoCache,
                CachedValue::SafeInfoCache(Some(safe_info)),
            )
            .await;
        Ok(())
    }

    async fn get_indexer_data<'a>(&'a self, tx: OptTx<'a>) -> Result<IndexerData> {
        let myself = self.clone();
        Ok(self
            .caches
            .single_values
            .try_get_with_by_ref(&CachedValueDiscriminants::IndexerDataCache, async move {
                myself
                    .nest_transaction(tx)
                    .and_then(|op| {
                        op.perform(|tx| {
                            Box::pin(async move {
                                let model = chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                                    .one(tx.as_ref())
                                    .await?
                                    .ok_or(DbError::MissingFixedTableEntry("chain_info".into()))?;

                                let ledger_dst = if let Some(b) = model.ledger_dst {
                                    Some(Hash::from_bytes(&b)?)
                                } else {
                                    None
                                };

                                let safe_registry_dst = if let Some(b) = model.safe_registry_dst {
                                    Some(Hash::from_bytes(&b)?)
                                } else {
                                    None
                                };

                                let channels_dst = if let Some(b) = model.channels_dst {
                                    Some(Hash::from_bytes(&b)?)
                                } else {
                                    None
                                };

                                Ok::<_, DbError>(CachedValue::IndexerDataCache(IndexerData {
                                    ledger_dst,
                                    safe_registry_dst,
                                    channels_dst,
                                    ticket_price: model.ticket_price.map(|p| BalanceType::HOPR.balance_bytes(p)),
                                    nr_enabled: model.network_registry_enabled,
                                }))
                            })
                        })
                    })
                    .await
            })
            .await?
            .try_into()?)
    }

    async fn set_domain_separator<'a>(&'a self, tx: OptTx<'a>, dst_type: DomainSeparator, value: Hash) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let mut active_model = chain_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        ..Default::default()
                    };

                    match dst_type {
                        DomainSeparator::Ledger => {
                            active_model.ledger_dst = Set(Some(value.to_bytes().into()));
                        }
                        DomainSeparator::SafeRegistry => {
                            active_model.safe_registry_dst = Set(Some(value.to_bytes().into()));
                        }
                        DomainSeparator::Channel => {
                            active_model.channels_dst = Set(Some(value.to_bytes().into()));
                        }
                    }

                    active_model.update(tx.as_ref()).await?;

                    Ok::<(), DbError>(())
                })
            })
            .await?;

        self.caches
            .single_values
            .invalidate(&CachedValueDiscriminants::IndexerDataCache)
            .await;
        Ok(())
    }

    async fn update_ticket_price<'a>(&'a self, tx: OptTx<'a>, price: Balance) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        ticket_price: Set(Some(price.amount().to_be_bytes().into())),
                        ..Default::default()
                    }
                    .update(tx.as_ref())
                    .await?;

                    Ok::<(), DbError>(())
                })
            })
            .await?;

        self.caches
            .single_values
            .invalidate(&CachedValueDiscriminants::IndexerDataCache)
            .await;
        Ok(())
    }

    async fn get_last_indexed_block<'a>(&'a self, tx: OptTx<'a>) -> Result<u32> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbError::MissingFixedTableEntry("node_info".into()))
                        .map(|m| m.last_indexed_block as u32)
                })
            })
            .await
    }

    async fn set_last_indexed_block<'a>(&'a self, tx: OptTx<'a>, block_num: u32) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        last_indexed_block: Set(block_num as i32),
                        ..Default::default()
                    }
                    .save(tx.as_ref())
                    .await?;
                    Ok::<_, DbError>(())
                })
            })
            .await
    }

    async fn set_network_registry_enabled<'a>(&'a self, tx: OptTx<'a>, enabled: bool) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        network_registry_enabled: Set(enabled),
                        ..Default::default()
                    }
                    .save(tx.as_ref())
                    .await?;
                    Ok::<_, DbError>(())
                })
            })
            .await?;

        self.caches
            .single_values
            .invalidate(&CachedValueDiscriminants::IndexerDataCache)
            .await;
        Ok(())
    }

    async fn get_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str) -> Result<Option<Box<[u8]>>> {
        let k = key.to_owned();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<Option<Box<[u8]>>, DbError>(
                        global_settings::Entity::find()
                            .filter(global_settings::Column::Key.eq(k))
                            .one(tx.as_ref())
                            .await?
                            .map(|m| m.value.into_boxed_slice()),
                    )
                })
            })
            .await
    }

    async fn set_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str, value: Option<&[u8]>) -> Result<()> {
        let k = key.to_owned();
        let value = value.map(Vec::from);
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    if let Some(v) = value {
                        let mut am = global_settings::Entity::find()
                            .filter(global_settings::Column::Key.eq(k.clone()))
                            .one(tx.as_ref())
                            .await?
                            .map(|m| m.into_active_model())
                            .unwrap_or(global_settings::ActiveModel {
                                key: Set(k),
                                ..Default::default()
                            });
                        am.value = Set(v);
                        am.save(tx.as_ref()).await?;
                    } else {
                        global_settings::Entity::delete_many()
                            .filter(global_settings::Column::Key.eq(k))
                            .exec(tx.as_ref())
                            .await?;
                    }
                    Ok::<(), DbError>(())
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_crypto_types::keypairs::ChainKeypair;
    use hopr_crypto_types::prelude::Keypair;
    use hopr_primitive_types::prelude::{Address, BalanceType};

    use crate::db::HoprDb;
    use crate::info::{HoprDbInfoOperations, SafeInfo};

    lazy_static::lazy_static! {
        static ref ADDR_1: Address = Address::from(hex!("86fa27add61fafc955e2da17329bba9f31692fe7"));
        static ref ADDR_2: Address = Address::from(hex!("4c8bbd047c2130e702badb23b6b97a88b6562324"));
    }

    #[async_std::test]
    async fn test_set_get_balance() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        assert_eq!(
            BalanceType::HOPR.zero(),
            db.get_safe_balance(None).await.unwrap(),
            "balance must be 0"
        );

        let balance = BalanceType::HOPR.balance(10_000);
        db.set_safe_balance(None, balance).await.unwrap();

        assert_eq!(
            balance,
            db.get_safe_balance(None).await.unwrap(),
            "balance must be {balance}"
        );
    }

    #[async_std::test]
    async fn test_set_get_allowance() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        assert_eq!(
            BalanceType::HOPR.zero(),
            db.get_safe_allowance(None).await.unwrap(),
            "balance must be 0"
        );

        let balance = BalanceType::HOPR.balance(10_000);
        db.set_safe_allowance(None, balance).await.unwrap();

        assert_eq!(
            balance,
            db.get_safe_allowance(None).await.unwrap(),
            "balance must be {balance}"
        );
    }

    #[async_std::test]
    async fn test_set_get_indexer_data() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let data = db.get_indexer_data(None).await.unwrap();
        assert_eq!(data.ticket_price, None);

        let price = BalanceType::HOPR.balance(10);
        db.update_ticket_price(None, price).await.unwrap();

        let data = db.get_indexer_data(None).await.unwrap();

        assert_eq!(data.ticket_price, Some(price));
    }

    #[async_std::test]
    async fn test_set_get_safe_info_with_cache() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        assert_eq!(None, db.get_safe_info(None).await.unwrap());

        let safe_info = SafeInfo {
            safe_address: *ADDR_1,
            module_address: *ADDR_2,
        };

        db.set_safe_info(None, safe_info).await.unwrap();

        assert_eq!(Some(safe_info), db.get_safe_info(None).await.unwrap());
    }

    #[async_std::test]
    async fn test_set_get_safe_info() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        assert_eq!(None, db.get_safe_info(None).await.unwrap());

        let safe_info = SafeInfo {
            safe_address: *ADDR_1,
            module_address: *ADDR_2,
        };

        db.set_safe_info(None, safe_info).await.unwrap();
        db.caches.single_values.invalidate_all();

        assert_eq!(Some(safe_info), db.get_safe_info(None).await.unwrap());
    }

    #[async_std::test]
    async fn test_set_last_indexed_block() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        assert_eq!(0, db.get_last_indexed_block(None).await.unwrap());

        let block_num = 100000;
        db.set_last_indexed_block(None, block_num).await.unwrap();

        assert_eq!(block_num, db.get_last_indexed_block(None).await.unwrap());
    }

    #[async_std::test]
    async fn test_set_get_global_setting() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let key = "test";
        let value = hex!("deadbeef");

        assert_eq!(None, db.get_global_setting(None, key).await.unwrap());

        db.set_global_setting(None, key, Some(&value)).await.unwrap();

        assert_eq!(Some(value.into()), db.get_global_setting(None, key).await.unwrap());

        db.set_global_setting(None, key, None).await.unwrap();

        assert_eq!(None, db.get_global_setting(None, key).await.unwrap());
    }
}
