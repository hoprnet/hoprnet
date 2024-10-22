use async_trait::async_trait;
use futures::TryFutureExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use tracing::trace;

use hopr_crypto_types::prelude::Hash;
use hopr_db_api::info::*;
use hopr_db_entity::{chain_info, global_settings, node_info};
use hopr_primitive_types::prelude::*;

use crate::cache::{CachedValue, CachedValueDiscriminants};
use crate::db::HoprDb;
use crate::errors::DbSqlError::MissingFixedTableEntry;
use crate::errors::{DbSqlError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx, SINGULAR_TABLE_FIXED_ID};

/// Enumerates different domain separators
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DescribedBlock {
    pub latest_block_number: u32,
    pub checksum: Hash,
    pub block_prior_to_checksum_update: u32,
}

/// Defines DB access API for various node information.
///
/// # Checksum computation
///
/// $H$ denotes Keccak256 hash function and $||$  byte string concatenation.
///
/// For a block $b_1$ containing logs $L_1, L_2, \ldots L_n$ corresponding to tx hashes $Tx_1, Tx_2, \ldots Tx_n$, a block hash is computed as:
///```math
/// H_{b_1} = H(Tx_1 || Tx_2 || \ldots || Tx_n)
///```
/// Given $C_0 = H(0x00...0)$ , the checksum $C_{k+1}$ after processing block $b_{k+1}$ is given as follows:
///
/// ```math
/// C_{k+1} = H(C_k || H_{b_{k+1}})
/// ```
///
#[async_trait]
pub trait HoprDbInfoOperations {
    /// Gets node's Safe balance of HOPR tokens.
    async fn get_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance>;

    /// Sets node's Safe balance of HOPR tokens.
    async fn set_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>, new_balance: Balance) -> Result<()>;

    /// Gets node's Safe allowance of HOPR tokens.
    async fn get_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance>;

    /// Sets node's Safe allowance of HOPR tokens.
    async fn set_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>, new_allowance: Balance) -> Result<()>;

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

    /// Sets the minimum required winning probability for incoming tickets.
    /// The value must be between zero and 1.
    async fn set_minimum_incoming_ticket_win_prob<'a>(&'a self, tx: OptTx<'a>, win_prob: f64) -> Result<()>;

    /// Updates the ticket price.
    /// To retrieve the stored ticket price, use [`HoprDbInfoOperations::get_indexer_data`],
    /// note that this setter should invalidate the cache.
    async fn update_ticket_price<'a>(&'a self, tx: OptTx<'a>, price: Balance) -> Result<()>;

    /// Retrieves the last indexed block number.
    async fn get_last_indexed_block<'a>(&'a self, tx: OptTx<'a>) -> Result<DescribedBlock>;

    /// Updates the last indexed block number together with the checksum of log TXs processed
    /// in that block (if there were any logs in this block).
    async fn set_last_indexed_block<'a>(
        &'a self,
        tx: OptTx<'a>,
        block_num: u32,
        block_log_tx_hash: Option<Hash>,
    ) -> Result<()>;

    /// Updates the network registry state.
    /// To retrieve the stored network registry state, use [`HoprDbInfoOperations::get_indexer_data`],
    /// note that this setter should invalidate the cache.
    async fn set_network_registry_enabled<'a>(&'a self, tx: OptTx<'a>, enabled: bool) -> Result<()>;

    /// Gets global setting value with the given key.
    async fn get_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str) -> Result<Option<Box<[u8]>>>;

    /// Sets the global setting value with the given key.
    ///
    /// If the setting with the given `key` does not exist, it is created.
    /// If `value` is `None` and a setting with the given `key` exists, it is removed.
    async fn set_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str, value: Option<&[u8]>) -> Result<()>;
}

#[async_trait]
impl HoprDbInfoOperations for HoprDb {
    async fn get_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbSqlError::MissingFixedTableEntry("node_info".into()))
                        .map(|m| BalanceType::HOPR.balance_bytes(m.safe_balance))
                })
            })
            .await
    }

    async fn set_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>, new_balance: Balance) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbSqlError>(
                        node_info::ActiveModel {
                            id: Set(SINGULAR_TABLE_FIXED_ID),
                            safe_balance: Set(new_balance.amount().to_be_bytes().into()),
                            ..Default::default()
                        }
                        .update(tx.as_ref()) // DB is primed in the migration, so only update is needed
                        .await?,
                    )
                })
            })
            .await?;

        Ok(())
    }

    async fn get_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbSqlError::MissingFixedTableEntry("node_info".into()))
                        .map(|m| BalanceType::HOPR.balance_bytes(m.safe_allowance))
                })
            })
            .await
    }

    async fn set_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>, new_allowance: Balance) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        safe_allowance: Set(new_allowance.amount().to_be_bytes().to_vec()),
                        ..Default::default()
                    }
                    .update(tx.as_ref()) // DB is primed in the migration, so only update is needed
                    .await?;

                    Ok::<_, DbSqlError>(())
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
                                    .ok_or(DbSqlError::MissingFixedTableEntry("node_info".into()))?;
                                Ok::<_, DbSqlError>(info.safe_address.zip(info.module_address))
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
                    .update(tx.as_ref()) // DB is primed in the migration, so only update is needed
                    .await?;
                    Ok::<_, DbSqlError>(())
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
                                    .ok_or(DbSqlError::MissingFixedTableEntry("chain_info".into()))?;

                                let ledger_dst = if let Some(b) = model.ledger_dst {
                                    Some(Hash::try_from(b.as_ref())?)
                                } else {
                                    None
                                };

                                let safe_registry_dst = if let Some(b) = model.safe_registry_dst {
                                    Some(Hash::try_from(b.as_ref())?)
                                } else {
                                    None
                                };

                                let channels_dst = if let Some(b) = model.channels_dst {
                                    Some(Hash::try_from(b.as_ref())?)
                                } else {
                                    None
                                };

                                Ok::<_, DbSqlError>(CachedValue::IndexerDataCache(IndexerData {
                                    ledger_dst,
                                    safe_registry_dst,
                                    channels_dst,
                                    ticket_price: model.ticket_price.map(|p| BalanceType::HOPR.balance_bytes(p)),
                                    minimum_incoming_ticket_winning_prob: model.min_incoming_ticket_win_prob as f64,
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
                            active_model.ledger_dst = Set(Some(value.as_ref().into()));
                        }
                        DomainSeparator::SafeRegistry => {
                            active_model.safe_registry_dst = Set(Some(value.as_ref().into()));
                        }
                        DomainSeparator::Channel => {
                            active_model.channels_dst = Set(Some(value.as_ref().into()));
                        }
                    }

                    // DB is primed in the migration, so only update is needed
                    active_model.update(tx.as_ref()).await?;

                    Ok::<(), DbSqlError>(())
                })
            })
            .await?;

        self.caches
            .single_values
            .invalidate(&CachedValueDiscriminants::IndexerDataCache)
            .await;
        Ok(())
    }

    async fn set_minimum_incoming_ticket_win_prob<'a>(&'a self, tx: OptTx<'a>, win_prob: f64) -> Result<()> {
        if !(0.0..=1.0).contains(&win_prob) {
            return Err(DbSqlError::LogicalError(
                "winning probability must be between 0 and 1".into(),
            ));
        }

        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        min_incoming_ticket_win_prob: Set(win_prob as f32),
                        ..Default::default()
                    }
                    .update(tx.as_ref())
                    .await?;

                    Ok::<(), DbSqlError>(())
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

                    Ok::<(), DbSqlError>(())
                })
            })
            .await?;

        self.caches
            .single_values
            .invalidate(&CachedValueDiscriminants::IndexerDataCache)
            .await;
        Ok(())
    }

    async fn get_last_indexed_block<'a>(&'a self, tx: OptTx<'a>) -> Result<DescribedBlock> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbSqlError::MissingFixedTableEntry("chain_info".into()))
                        .and_then(|m| {
                            let chain_checksum = if let Some(b) = m.chain_checksum {
                                Hash::try_from(b.as_slice()).map_err(|_| DbSqlError::DecodingError)?
                            } else {
                                Hash::default()
                            };
                            Ok(DescribedBlock {
                                latest_block_number: m.last_indexed_block as u32,
                                checksum: chain_checksum,
                                block_prior_to_checksum_update: m.previous_indexed_block_prio_to_checksum_update as u32,
                            })
                        })
                })
            })
            .await
    }

    async fn set_last_indexed_block<'a>(
        &'a self,
        tx: OptTx<'a>,
        block_num: u32,
        block_log_tx_hash: Option<Hash>,
    ) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let model = chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(MissingFixedTableEntry("chain_info".into()))?;

                    let current_last_indexed_block = model.last_indexed_block;
                    let current_checksum = model
                        .chain_checksum
                        .clone()
                        .map(|v| Hash::try_from(v.as_ref()))
                        .unwrap_or(Ok(Hash::default()))?;

                    let mut active_model = model.into_active_model();

                    if let Some(block_log_hash) = block_log_tx_hash {
                        let new_hash = Hash::create(&[current_checksum.as_ref(), block_log_hash.as_ref()]);
                        active_model.chain_checksum = Set(Some(new_hash.as_ref().to_vec()));
                        // when a new checksum is computed, we need to update previous_indexed_block_prio_to_checksum_update
                        active_model.previous_indexed_block_prio_to_checksum_update = Set(current_last_indexed_block);
                        trace!(
                            old_checksum = ?current_checksum,
                            old_block = current_last_indexed_block,
                            new_checksum = ?new_hash,
                            new_block = block_num,
                            "update block checksum"
                        );
                    }

                    active_model.last_indexed_block = Set(block_num as i32);
                    active_model.update(tx.as_ref()).await?;

                    Ok::<_, DbSqlError>(())
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
                    .update(tx.as_ref())
                    .await?;
                    Ok::<_, DbSqlError>(())
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
                    Ok::<Option<Box<[u8]>>, DbSqlError>(
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
                    Ok::<(), DbSqlError>(())
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
    use hopr_crypto_types::types::Hash;
    use hopr_primitive_types::prelude::{Address, BalanceType};

    use crate::db::HoprDb;
    use crate::info::{HoprDbInfoOperations, SafeInfo};

    lazy_static::lazy_static! {
        static ref ADDR_1: Address = Address::from(hex!("86fa27add61fafc955e2da17329bba9f31692fe7"));
        static ref ADDR_2: Address = Address::from(hex!("4c8bbd047c2130e702badb23b6b97a88b6562324"));
    }

    #[async_std::test]
    async fn test_set_get_balance() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert_eq!(
            BalanceType::HOPR.zero(),
            db.get_safe_hopr_balance(None).await?,
            "balance must be 0"
        );

        let balance = BalanceType::HOPR.balance(10_000);
        db.set_safe_hopr_balance(None, balance).await?;

        assert_eq!(
            balance,
            db.get_safe_hopr_balance(None).await?,
            "balance must be {balance}"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_set_get_allowance() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert_eq!(
            BalanceType::HOPR.zero(),
            db.get_safe_hopr_allowance(None).await?,
            "balance must be 0"
        );

        let balance = BalanceType::HOPR.balance(10_000);
        db.set_safe_hopr_allowance(None, balance).await?;

        assert_eq!(
            balance,
            db.get_safe_hopr_allowance(None).await?,
            "balance must be {balance}"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_set_get_indexer_data() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let data = db.get_indexer_data(None).await?;
        assert_eq!(data.ticket_price, None);

        let price = BalanceType::HOPR.balance(10);
        db.update_ticket_price(None, price).await?;

        db.set_minimum_incoming_ticket_win_prob(None, 0.5).await?;

        let data = db.get_indexer_data(None).await?;

        assert_eq!(data.ticket_price, Some(price));
        assert_eq!(data.minimum_incoming_ticket_winning_prob, 0.5);
        Ok(())
    }

    #[async_std::test]
    async fn test_set_get_safe_info_with_cache() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert_eq!(None, db.get_safe_info(None).await?);

        let safe_info = SafeInfo {
            safe_address: *ADDR_1,
            module_address: *ADDR_2,
        };

        db.set_safe_info(None, safe_info).await?;

        assert_eq!(Some(safe_info), db.get_safe_info(None).await?);
        Ok(())
    }

    #[async_std::test]
    async fn test_set_get_safe_info() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert_eq!(None, db.get_safe_info(None).await?);

        let safe_info = SafeInfo {
            safe_address: *ADDR_1,
            module_address: *ADDR_2,
        };

        db.set_safe_info(None, safe_info).await?;
        db.caches.single_values.invalidate_all();

        assert_eq!(Some(safe_info), db.get_safe_info(None).await?);
        Ok(())
    }

    #[async_std::test]
    async fn test_set_last_indexed_block() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let described_block = db.get_last_indexed_block(None).await?;
        assert_eq!(0, described_block.latest_block_number);
        assert_eq!(0, described_block.block_prior_to_checksum_update);

        let checksum = Hash::default().hash();
        let expexted_block_num = 100000;

        db.set_last_indexed_block(None, expexted_block_num, Some(checksum))
            .await?;

        let next_described_block = db.get_last_indexed_block(None).await?;
        assert_eq!(expexted_block_num, next_described_block.latest_block_number);
        assert_eq!(0, next_described_block.block_prior_to_checksum_update);

        let expected_next_checksum = Hash::create(&[described_block.checksum.as_ref(), checksum.as_ref()]);
        assert_eq!(expected_next_checksum, next_described_block.checksum);
        Ok(())
    }

    #[async_std::test]
    async fn test_set_get_global_setting() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let key = "test";
        let value = hex!("deadbeef");

        assert_eq!(None, db.get_global_setting(None, key).await?);

        db.set_global_setting(None, key, Some(&value)).await?;

        assert_eq!(Some(value.into()), db.get_global_setting(None, key).await?);

        db.set_global_setting(None, key, None).await?;

        assert_eq!(None, db.get_global_setting(None, key).await?);
        Ok(())
    }
}
