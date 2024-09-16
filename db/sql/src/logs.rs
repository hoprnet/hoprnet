use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, Related,
};
use sea_query::OnConflict;
use tracing::error;

use hopr_db_entity::{log, log_status};
use hopr_primitive_types::prelude::*;

use crate::db::HoprDb;
use crate::errors::{DbSqlError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx};

#[async_trait]
pub trait HoprDbLogOperations {
    /// Retrieve acknowledged winning tickets according to the given `selector`.
    ///
    /// The optional transaction `tx` must be in the database.
    async fn store_logs<'a>(&'a self, tx: OptTx<'a>, logs: Vec<SerializableLog>) -> Result<()>;
}

#[async_trait]
impl HoprDbLogOperations for HoprDb {
    async fn store_logs<'a>(&'a self, tx: OptTx<'a>, logs: Vec<SerializableLog>) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let models = logs.clone().into_iter().map(log::ActiveModel::from).collect::<Vec<_>>();
                    let status_models = logs.into_iter().map(log_status::ActiveModel::from).collect::<Vec<_>>();
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
                        Ok(_) => {
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
}
