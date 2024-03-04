use async_trait::async_trait;
use hopr_crypto_types::prelude::Hash;
use hopr_db_entity::{chain_info, global_settings, node_info};
use hopr_primitive_types::prelude::{Balance, BalanceType, BinarySerializable, IntoEndian};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::db::HoprDb;
use crate::errors::DbError::CorruptedData;

use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx, SINGULAR_TABLE_FIXED_ID};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OnChainData {
    pub ledger_dst: Hash,
    pub safe_registry_dst: Hash,
    pub channels_dst: Hash,
    pub ticket_price: Balance,
    pub nr_enabled: bool,
    pub last_indexed_block: u32,
}

#[async_trait]
pub trait HoprDbInfoOperations {
    async fn get_safe_balance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance>;

    async fn set_safe_balance<'a>(&'a self, tx: OptTx<'a>, new_balance: Balance) -> Result<()>;

    async fn get_safe_allowance<'a>(&'a self, tx: OptTx<'a>) -> Result<Balance>;

    async fn get_chain_data<'a>(&'a self, tx: OptTx<'a>) -> Result<OnChainData>;

    async fn get_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str) -> Result<Option<Box<[u8]>>>;

    async fn set_global_setting<'a>(&'a self, txc: OptTx<'a>, key: &str, value: &[u8]) -> Result<()>;
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
                        .ok_or(CorruptedData)
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
                    node_info::ActiveModel {
                        id: Set(SINGULAR_TABLE_FIXED_ID),
                        safe_balance: Set(new_balance.amount().to_be_bytes().into()),
                        ..Default::default()
                    }
                    .update(tx.as_ref())
                    .await
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
                        .ok_or(CorruptedData)
                        .map(|m| BalanceType::HOPR.balance_bytes(m.safe_allowance))
                })
            })
            .await
    }

    async fn get_chain_data<'a>(&'a self, tx: OptTx<'a>) -> Result<OnChainData> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let model = chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .ok_or(CorruptedData)?;

                    Ok::<OnChainData, DbError>(OnChainData {
                        ledger_dst: Hash::from_bytes(&model.ledger_dst)?,
                        safe_registry_dst: Hash::from_bytes(&model.safe_registry_dst)?,
                        channels_dst: Hash::from_bytes(&model.channels_dst)?,
                        ticket_price: BalanceType::HOPR.balance_bytes(model.ticket_price),
                        nr_enabled: model.network_registry_enabled,
                        last_indexed_block: model.last_indexed_block as u32,
                    })
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

    async fn set_global_setting<'a>(&'a self, tx: OptTx<'a>, key: &str, value: &[u8]) -> Result<()> {
        let k = key.to_owned();
        let v = value.to_vec();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    global_settings::ActiveModel {
                        key: Set(k),
                        value: Set(v),
                        ..Default::default()
                    }
                    .insert(tx.as_ref())
                    .await?;
                    Ok::<(), DbError>(())
                })
            })
            .await
    }
}
