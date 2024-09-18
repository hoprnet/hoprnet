use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, Related,
};
use sea_query::OnConflict;
use std::str::FromStr;
use tracing::error;

use hopr_db_entity::errors::DbEntityError;
use hopr_db_entity::prelude::{Log, LogStatus};
use hopr_db_entity::{log, log_status};
use hopr_primitive_types::prelude::*;

use crate::db::HoprDb;
use crate::errors::{DbSqlError, Result};
use crate::TargetDb;
use crate::{HoprDbGeneralModelOperations, OptTx};

#[async_trait]
pub trait HoprDbLogOperations {
    /// Retrieve acknowledged winning tickets according to the given `selector`.
    ///
    /// The optional transaction `tx` must be in the database.
    async fn store_logs<'a>(&'a self, tx: OptTx<'a>, logs: Vec<SerializableLog>) -> Result<()>;

    async fn get_log<'a>(
        &'a self,
        tx: OptTx<'a>,
        block_number: u64,
        tx_index: u64,
        log_index: u64,
    ) -> Result<SerializableLog>;

    async fn get_all_logs<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<SerializableLog>>;
}

#[async_trait]
impl HoprDbLogOperations for HoprDb {
    async fn store_logs<'a>(&'a self, tx: OptTx<'a>, logs: Vec<SerializableLog>) -> Result<()> {
        if logs.is_empty() {
            return Err(DbSqlError::EmptyLogsList.into());
        }

        self.nest_transaction_in_db(tx, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let models = logs.clone().into_iter().map(log::ActiveModel::from).collect::<Vec<_>>();
                    let status_models = logs.into_iter().map(log_status::ActiveModel::from).collect::<Vec<_>>();
                    match log_status::Entity::insert_many(status_models)
                        .on_conflict(
                            OnConflict::columns([
                                log_status::Column::LogIndex,
                                log_status::Column::TransactionIndex,
                                log_status::Column::BlockNumber,
                            ])
                            .do_nothing()
                            .to_owned(),
                        )
                        .exec(tx.as_ref())
                        .await
                    {
                        Ok(_) => {
                            match log::Entity::insert_many(models)
                                .on_conflict(
                                    OnConflict::columns([
                                        log::Column::LogIndex,
                                        log::Column::TransactionIndex,
                                        log::Column::BlockNumber,
                                    ])
                                    .do_nothing()
                                    .to_owned(),
                                )
                                .exec(tx.as_ref())
                                .await
                            {
                                Ok(_) => Ok(()),
                                Err(DbErr::RecordNotInserted) => {
                                    error!("Failed to insert log status into db");
                                    Err(DbErr::RecordNotInserted.into())
                                }
                                Err(e) => Err(e.into()),
                            }
                        }
                        Err(DbErr::RecordNotInserted) => {
                            error!("Failed to insert log into db");
                            Err(DbErr::RecordNotInserted.into())
                        }
                        Err(e) => Err(e.into()),
                    }
                })
            })
            .await
    }

    async fn get_log<'a>(
        &'a self,
        tx: OptTx<'a>,
        block_number: u64,
        tx_index: u64,
        log_index: u64,
    ) -> Result<SerializableLog> {
        self.nest_transaction_in_db(tx, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let bn_enc = block_number.to_be_bytes().to_vec();
                    let tx_index_enc = tx_index.to_be_bytes().to_vec();
                    let log_index_enc = log_index.to_be_bytes().to_vec();
                    let maybe_log = Log::find()
                        .filter(log::Column::BlockNumber.eq(bn_enc))
                        .filter(log::Column::TransactionIndex.eq(tx_index_enc))
                        .filter(log::Column::LogIndex.eq(log_index_enc))
                        .find_also_related(LogStatus)
                        .all(tx.as_ref())
                        .await?
                        .pop();
                    if let Some((log, log_status)) = maybe_log {
                        if let Some(status) = log_status {
                            create_log(log, status)
                        } else {
                            Err(DbSqlError::MissingLogStatus)
                        }
                    } else {
                        Err(DbSqlError::MissingLog)
                    }
                })
            })
            .await
    }

    async fn get_all_logs<'a>(&'a self, tx: OptTx<'a>) -> Result<Vec<SerializableLog>> {
        self.nest_transaction_in_db(tx, TargetDb::Logs)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Log::find()
                        .find_also_related(LogStatus)
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(|(log, log_status)| {
                            if let Some(status) = log_status {
                                create_log(log, status)
                            } else {
                                Err(DbSqlError::MissingLog)
                            }
                        })
                        .collect()
                })
            })
            .await
    }
}

fn create_log(raw_log: log::Model, status: log_status::Model) -> Result<SerializableLog> {
    let log = SerializableLog::from(raw_log);
    if let Some(raw_ts) = status.processed_at {
        let ts = DateTime::<Utc>::from_str(raw_ts.as_str())
            .map_err(|_| DbEntityError::ConversionError("failed to decode log status processed_at".into()))?;
        Ok(SerializableLog {
            processed: Some(status.processed),
            processed_at: Some(ts),
            ..log
        })
    } else {
        Ok(SerializableLog {
            processed: Some(status.processed),
            processed_at: None,
            ..log
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::DbSqlError;
    use crate::errors::DbSqlError::DecodingError;
    use crate::HoprDbGeneralModelOperations;
    use hopr_crypto_types::prelude::{ChainKeypair, Hash, Keypair, OffchainKeypair};
    use hopr_internal_types::prelude::AccountType::NotAnnounced;

    #[async_std::test]
    async fn test_store_single_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let log = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        db.store_logs(None, [log.clone()].into()).await.unwrap();

        let logs = db.get_all_logs(None).await.unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], log);
    }

    #[async_std::test]
    async fn test_store_multiple_logs() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let log_1 = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            processed: Some(false),
            ..Default::default()
        };

        let log_2 = SerializableLog {
            address: Hash::create(&[b"my address 2"]).to_hex(),
            topics: [Hash::create(&[b"my topic 2"]).to_hex()].into(),
            data: [1, 2, 3, 4, 5].into(),
            tx_index: 2u64,
            block_number: 2u64,
            block_hash: Hash::create(&[b"my block hash 2"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash 2"]).to_hex(),
            log_index: 2u64,
            removed: false,
            processed: Some(true),
            ..Default::default()
        };

        db.store_logs(None, [log_1.clone(), log_2.clone()].into())
            .await
            .unwrap();

        let logs = db.get_all_logs(None).await.unwrap();

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], log_1);
        assert_eq!(logs[1], log_2);

        let log_2_retrieved = db
            .get_log(None, log_2.block_number, log_2.tx_index, log_2.log_index)
            .await
            .unwrap();

        assert_eq!(log_2, log_2_retrieved);
    }

    #[async_std::test]
    async fn test_store_duplicate_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let log = SerializableLog {
            address: Hash::create(&[b"my address"]).to_hex(),
            topics: [Hash::create(&[b"my topic"]).to_hex()].into(),
            data: [1, 2, 3, 4].into(),
            tx_index: 1u64,
            block_number: 1u64,
            block_hash: Hash::create(&[b"my block hash"]).to_hex(),
            tx_hash: Hash::create(&[b"my tx hash"]).to_hex(),
            log_index: 1u64,
            removed: false,
            ..Default::default()
        };

        db.store_logs(None, [log.clone()].into()).await.unwrap();

        db.store_logs(None, [log.clone()].into())
            .await
            .expect_err("should not store duplicate log");

        let logs = db.get_all_logs(None).await.unwrap();

        assert_eq!(logs.len(), 1);
    }

    #[async_std::test]
    async fn test_store_no_log() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        db.store_logs(None, [].into())
            .await
            .expect_err("should fail due to empty list");

        let logs = db.get_all_logs(None).await.unwrap();

        assert_eq!(logs.len(), 0);
    }
}
