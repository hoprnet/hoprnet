use async_trait::async_trait;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use sea_query::OnConflict;
use hopr_db_entity::network_registry;
use hopr_primitive_types::prelude::{Address, ToHex};

use crate::db::HoprDb;
use crate::{HoprDbGeneralModelOperations, OptTx};
use crate::errors::{DbError, Result};

#[async_trait]
pub trait HoprDbRegistryOperations {
    async fn allow_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<()>;

    async fn is_allowed_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<bool>;

    async fn deny_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<()>;
}

#[async_trait]
impl HoprDbRegistryOperations for HoprDb {
    async fn allow_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| Box::pin(async move {
                let entry = network_registry::ActiveModel {
                    chain_address: Set(address.to_hex()),
                    ..Default::default()
                };

                match network_registry::Entity::insert(entry)
                    .on_conflict(OnConflict::column(network_registry::Column::ChainAddress).do_nothing().to_owned())
                    .exec(tx.as_ref())
                    .await {
                    Ok(_) | Err(DbErr::RecordNotInserted) => Ok::<_, DbError>(()),
                    Err(e) => Err(e.into())
                }
            }))
            .await
    }

    async fn is_allowed_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<bool> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| Box::pin(async move {
                Ok::<_, DbError>(network_registry::Entity::find()
                    .filter(network_registry::Column::ChainAddress.eq(address.to_hex()))
                    .one(tx.as_ref())
                    .await?
                    .is_some())
            }))
            .await
    }

    async fn deny_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| Box::pin(async move {
                network_registry::Entity::delete_many()
                    .filter(network_registry::Column::ChainAddress.eq(address.to_hex()))
                    .exec(tx.as_ref())
                    .await?;
                Ok::<_, DbError>(())
            }))
            .await
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use hopr_primitive_types::prelude::Address;
    use crate::db::HoprDb;
    use crate::registry::HoprDbRegistryOperations;

    lazy_static! {
        static ref ADDR_1: Address = "4331eaa9542b6b034c43090d9ec1c2198758dbc3".parse().unwrap();
        static ref ADDR_2: Address = "47d1677e018e79dcdd8a9c554466cb1556fa5007".parse().unwrap();
    }

    #[async_std::test]
    async fn test_network_registry_db() {
        let db = HoprDb::new_in_memory().await;

        assert!(!db.is_allowed_in_network_registry(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_allowed_in_network_registry(None, *ADDR_2).await.expect("should not fail"));

        db.allow_in_network_registry(None, *ADDR_1).await.expect("should not fail to allow in nr");

        assert!(db.is_allowed_in_network_registry(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_allowed_in_network_registry(None, *ADDR_2).await.expect("should not fail"));

        db.allow_in_network_registry(None, *ADDR_1).await.expect("should not fail to allow in nr when allowed");

        assert!(db.is_allowed_in_network_registry(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_allowed_in_network_registry(None, *ADDR_2).await.expect("should not fail"));

        db.deny_in_network_registry(None, *ADDR_1).await.expect("should fail to deny in nr");

        assert!(!db.is_allowed_in_network_registry(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_allowed_in_network_registry(None, *ADDR_2).await.expect("should not fail"));

        db.deny_in_network_registry(None, *ADDR_1).await.expect("should fail to deny in nr when denied");

        assert!(!db.is_allowed_in_network_registry(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_allowed_in_network_registry(None, *ADDR_2).await.expect("should not fail"));
    }

}
