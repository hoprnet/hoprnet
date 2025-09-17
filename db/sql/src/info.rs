use async_trait::async_trait;
use futures::TryFutureExt;
use hopr_crypto_types::prelude::Hash;
use hopr_db_entity::{
    chain_info, global_settings, node_info,
    prelude::{Account, Announcement, ChainInfo, Channel, NetworkEligibility, NetworkRegistry, NodeInfo},
};
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::prelude::*;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, EntityOrSelect, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, Set,
};
use tracing::trace;

use crate::{
    HoprDbGeneralModelOperations, OptTx, SINGULAR_TABLE_FIXED_ID, TargetDb,
    cache::{CachedValue, CachedValueDiscriminants},
    db::HoprDb,
    errors::{DbSqlError, DbSqlError::MissingFixedTableEntry, Result},
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexerStateInfo {
    // the latest block number that has been indexed and persisted to the database
    pub latest_block_number: u32,
    pub latest_log_block_number: u32,
    pub latest_log_checksum: Hash,
}

/// Contains various on-chain information collected by Indexer,
/// such as domain separators, ticket price, Network Registry status...etc.
/// All these members change very rarely and therefore can be cached.
#[derive(Clone, Copy, Debug)]
pub struct IndexerData {
    /// Ledger smart contract domain separator
    pub ledger_dst: Option<Hash>,
    /// Node safe registry smart contract domain separator
    pub safe_registry_dst: Option<Hash>,
    /// Channels smart contract domain separator
    pub channels_dst: Option<Hash>,
    /// Current ticket price
    pub ticket_price: Option<HoprBalance>,
    /// Minimum winning probability
    pub minimum_incoming_ticket_winning_prob: WinningProbability,
    /// Network registry state
    pub nr_enabled: bool,
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

impl IndexerData {
    /// Convenience method to retrieve domain separator according to the [DomainSeparator] enum.
    pub fn domain_separator(&self, dst_type: DomainSeparator) -> Option<Hash> {
        match dst_type {
            DomainSeparator::Ledger => self.ledger_dst,
            DomainSeparator::SafeRegistry => self.safe_registry_dst,
            DomainSeparator::Channel => self.channels_dst,
        }
    }
}

/// Defines DB access API for various node information.
///
/// # Checksum computation
///
/// $H$ denotes Keccak256 hash function and $||$  byte string concatenation.
///
/// For a block $b_1$ containing logs $L_1, L_2, \ldots L_n$ corresponding to tx hashes $Tx_1, Tx_2, \ldots Tx_n$, a
/// block hash is computed as:
///
/// ```math
/// H_{b_1} = H(Tx_1 || Tx_2 || \ldots || Tx_n)
/// ```
/// Given $C_0 = H(0x00...0)$ , the checksum $C_{k+1}$ after processing block $b_{k+1}$ is given as follows:
///
/// ```math
/// C_{k+1} = H(C_k || H_{b_{k+1}})
/// ```
#[async_trait]
pub trait HoprDbInfoOperations {
    /// Checks if the index is empty.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating whether the index is empty.
    async fn index_is_empty(&self) -> Result<bool>;

    /// Removes all data from all tables in the index database.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the operation.
    async fn clear_index_db<'a>(&'a self, tx: OptTx<'a>) -> Result<()>;

    /// Gets node's Safe balance of HOPR tokens.
    async fn get_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>) -> Result<HoprBalance>;

    /// Sets node's Safe balance of HOPR tokens.
    async fn set_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>, new_balance: HoprBalance) -> Result<()>;

    /// Gets node's Safe allowance of HOPR tokens.
    async fn get_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>) -> Result<HoprBalance>;

    /// Sets node's Safe allowance of HOPR tokens.
    async fn set_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>, new_allowance: HoprBalance) -> Result<()>;

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
    async fn set_minimum_incoming_ticket_win_prob<'a>(
        &'a self,
        tx: OptTx<'a>,
        win_prob: WinningProbability,
    ) -> Result<()>;

    /// Updates the ticket price.
    /// To retrieve the stored ticket price, use [`HoprDbInfoOperations::get_indexer_data`],
    /// note that this setter should invalidate the cache.
    async fn update_ticket_price<'a>(&'a self, tx: OptTx<'a>, price: HoprBalance) -> Result<()>;

    /// Gets the indexer state info.
    async fn get_indexer_state_info<'a>(&'a self, tx: OptTx<'a>) -> Result<IndexerStateInfo>;

    /// Updates the indexer state info.
    async fn set_indexer_state_info<'a>(&'a self, tx: OptTx<'a>, block_num: u32) -> Result<()>;

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
    async fn index_is_empty(&self) -> Result<bool> {
        let c = self.conn(TargetDb::Index);

        // There is always at least the node's own AccountEntry
        if Account::find().select().count(c).await? > 1 {
            return Ok(false);
        }

        if Announcement::find().one(c).await?.is_some() {
            return Ok(false);
        }

        if Channel::find().one(c).await?.is_some() {
            return Ok(false);
        }

        if NetworkEligibility::find().one(c).await?.is_some() {
            return Ok(false);
        }

        if NetworkRegistry::find().one(c).await?.is_some() {
            return Ok(false);
        }

        Ok(true)
    }

    async fn clear_index_db<'a>(&'a self, tx: OptTx<'a>) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Account::delete_many().exec(tx.as_ref()).await?;
                    Announcement::delete_many().exec(tx.as_ref()).await?;
                    Channel::delete_many().exec(tx.as_ref()).await?;
                    NetworkEligibility::delete_many().exec(tx.as_ref()).await?;
                    NetworkRegistry::delete_many().exec(tx.as_ref()).await?;
                    ChainInfo::delete_many().exec(tx.as_ref()).await?;
                    NodeInfo::delete_many().exec(tx.as_ref()).await?;

                    // Initial rows are needed in the ChainInfo and NodeInfo tables
                    // See the m20240226_000007_index_initial_seed.rs migration

                    let mut initial_row = chain_info::ActiveModel::new();
                    initial_row.id = Set(1);
                    ChainInfo::insert(initial_row).exec(tx.as_ref()).await?;

                    let mut initial_row = node_info::ActiveModel::new();
                    initial_row.id = Set(1);
                    NodeInfo::insert(initial_row).exec(tx.as_ref()).await?;

                    Ok::<(), DbSqlError>(())
                })
            })
            .await?;

        Ok(())
    }

    async fn get_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>) -> Result<HoprBalance> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(MissingFixedTableEntry("node_info".into()))
                        .map(|m| HoprBalance::from_be_bytes(m.safe_balance))
                })
            })
            .await
    }

    async fn set_safe_hopr_balance<'a>(&'a self, tx: OptTx<'a>, new_balance: HoprBalance) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbSqlError>(
                        node_info::ActiveModel {
                            id: Set(SINGULAR_TABLE_FIXED_ID),
                            safe_balance: Set(new_balance.to_be_bytes().into()),
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

    async fn get_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>) -> Result<HoprBalance> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    node_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(MissingFixedTableEntry("node_info".into()))
                        .map(|m| HoprBalance::from_be_bytes(m.safe_allowance))
                })
            })
            .await
    }

    async fn set_safe_hopr_allowance<'a>(&'a self, tx: OptTx<'a>, new_allowance: HoprBalance) -> Result<()> {
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

    async fn get_indexer_data<'a>(&'a self, tx: OptTx<'a>) -> Result<IndexerData> {
        let myself = self.clone();
        Ok(self
            .caches
            .single_values
            .try_get_with_by_ref(&CachedValueDiscriminants::IndexerDataCache, async move {
                tracing::warn!("cache miss on get_indexer_data");
                myself
                    .nest_transaction(tx)
                    .and_then(|op| {
                        op.perform(|tx| {
                            Box::pin(async move {
                                let model = chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                                    .one(tx.as_ref())
                                    .await?
                                    .ok_or(MissingFixedTableEntry("chain_info".into()))?;

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
                                    ticket_price: model.ticket_price.map(HoprBalance::from_be_bytes),
                                    minimum_incoming_ticket_winning_prob: (model.min_incoming_ticket_win_prob as f64)
                                        .try_into()?,
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

    async fn set_minimum_incoming_ticket_win_prob<'a>(
        &'a self,
        tx: OptTx<'a>,
        win_prob: WinningProbability,
    ) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        min_incoming_ticket_win_prob: Set(win_prob.as_f64() as f32),
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

    async fn update_ticket_price<'a>(&'a self, tx: OptTx<'a>, price: HoprBalance) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        ticket_price: Set(Some(price.to_be_bytes().into())),
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

    async fn get_indexer_state_info<'a>(&'a self, tx: OptTx<'a>) -> Result<IndexerStateInfo> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(DbSqlError::MissingFixedTableEntry("chain_info".into()))
                        .map(|m| IndexerStateInfo {
                            latest_block_number: m.last_indexed_block as u32,
                            ..Default::default()
                        })
                })
            })
            .await
    }

    async fn set_indexer_state_info<'a>(&'a self, tx: OptTx<'a>, block_num: u32) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let model = chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(MissingFixedTableEntry("chain_info".into()))?;

                    let current_last_indexed_block = model.last_indexed_block;

                    let mut active_model = model.into_active_model();

                    trace!(
                        old_block = current_last_indexed_block,
                        new_block = block_num,
                        "update block"
                    );

                    active_model.last_indexed_block = Set(block_num as i32);
                    active_model.update(tx.as_ref()).await?;

                    Ok::<_, DbSqlError>(())
                })
            })
            .await
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
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use hopr_primitive_types::{balance::HoprBalance, prelude::Address};

    use crate::{db::HoprDb, info::HoprDbInfoOperations};

    lazy_static::lazy_static! {
        static ref ADDR_1: Address = Address::from(hex!("86fa27add61fafc955e2da17329bba9f31692fe7"));
        static ref ADDR_2: Address = Address::from(hex!("4c8bbd047c2130e702badb23b6b97a88b6562324"));
    }

    #[tokio::test]
    async fn test_set_get_balance() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert_eq!(
            HoprBalance::zero(),
            db.get_safe_hopr_balance(None).await?,
            "balance must be 0"
        );

        let balance = HoprBalance::from(10_000);
        db.set_safe_hopr_balance(None, balance).await?;

        assert_eq!(
            balance,
            db.get_safe_hopr_balance(None).await?,
            "balance must be {balance}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_set_get_allowance() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert_eq!(
            HoprBalance::zero(),
            db.get_safe_hopr_allowance(None).await?,
            "balance must be 0"
        );

        let balance = HoprBalance::from(10_000);
        db.set_safe_hopr_allowance(None, balance).await?;

        assert_eq!(
            balance,
            db.get_safe_hopr_allowance(None).await?,
            "balance must be {balance}"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_set_get_indexer_data() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let data = db.get_indexer_data(None).await?;
        assert_eq!(data.ticket_price, None);

        let price = HoprBalance::from(10);
        db.update_ticket_price(None, price).await?;

        db.set_minimum_incoming_ticket_win_prob(None, 0.5.try_into()?).await?;

        let data = db.get_indexer_data(None).await?;

        assert_eq!(data.ticket_price, Some(price));
        assert_eq!(data.minimum_incoming_ticket_winning_prob, 0.5);
        Ok(())
    }

    #[tokio::test]
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
