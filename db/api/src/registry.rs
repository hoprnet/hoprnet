use async_trait::async_trait;
use hopr_db_entity::{network_eligibility, network_registry};
use hopr_primitive_types::prelude::{Address, ToHex};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use sea_query::OnConflict;

use crate::db::HoprDb;
use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx};

#[async_trait]
pub trait HoprDbRegistryOperations {
    async fn set_access_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address, allowed: bool)
        -> Result<()>;

    async fn is_allowed_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<bool>;

    async fn set_safe_eligibility<'a>(&'a self, tx: OptTx<'a>, address: Address, eligible: bool) -> Result<()>;

    async fn is_safe_eligible<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<bool>;
}

#[async_trait]
impl HoprDbRegistryOperations for HoprDb {
    async fn set_access_in_network_registry<'a>(
        &'a self,
        tx: OptTx<'a>,
        address: Address,
        allowed: bool,
    ) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    if allowed {
                        let entry = network_registry::ActiveModel {
                            chain_address: Set(address.to_hex()),
                            ..Default::default()
                        };

                        match network_registry::Entity::insert(entry)
                            .on_conflict(
                                OnConflict::column(network_registry::Column::ChainAddress)
                                    .do_nothing()
                                    .to_owned(),
                            )
                            .exec(tx.as_ref())
                            .await
                        {
                            Ok(_) | Err(DbErr::RecordNotInserted) => Ok::<_, DbError>(()),
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        network_registry::Entity::delete_many()
                            .filter(network_registry::Column::ChainAddress.eq(address.to_hex()))
                            .exec(tx.as_ref())
                            .await?;
                        Ok::<_, DbError>(())
                    }
                })
            })
            .await
    }

    async fn is_allowed_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<bool> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        network_registry::Entity::find()
                            .filter(network_registry::Column::ChainAddress.eq(address.to_hex()))
                            .one(tx.as_ref())
                            .await?
                            .is_some(),
                    )
                })
            })
            .await
    }

    async fn set_safe_eligibility<'a>(&'a self, tx: OptTx<'a>, address: Address, eligible: bool) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    if eligible {
                        let new_entry = network_eligibility::ActiveModel {
                            safe_address: Set(address.to_hex()),
                            ..Default::default()
                        };

                        match network_eligibility::Entity::insert(new_entry)
                            .on_conflict(
                                OnConflict::column(network_eligibility::Column::SafeAddress)
                                    .do_nothing()
                                    .to_owned(),
                            )
                            .exec(tx.as_ref())
                            .await
                        {
                            Ok(_) | Err(DbErr::RecordNotInserted) => Ok::<_, DbError>(()),
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        network_eligibility::Entity::delete_many()
                            .filter(network_eligibility::Column::SafeAddress.eq(address.to_hex()))
                            .exec(tx.as_ref())
                            .await?;
                        Ok::<_, DbError>(())
                    }
                })
            })
            .await
    }

    async fn is_safe_eligible<'a>(&'a self, tx: OptTx<'a>, address: Address) -> Result<bool> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(
                        network_eligibility::Entity::find()
                            .filter(network_eligibility::Column::SafeAddress.eq(address.to_hex()))
                            .one(tx.as_ref())
                            .await?
                            .is_some(),
                    )
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::db::HoprDb;
    use crate::registry::HoprDbRegistryOperations;
    use hopr_primitive_types::prelude::Address;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref ADDR_1: Address = "4331eaa9542b6b034c43090d9ec1c2198758dbc3".parse().unwrap();
        static ref ADDR_2: Address = "47d1677e018e79dcdd8a9c554466cb1556fa5007".parse().unwrap();
    }

    #[async_std::test]
    async fn test_network_registry_db() {
        let db = HoprDb::new_in_memory().await;

        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_1)
            .await
            .expect("should not fail"));
        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_2)
            .await
            .expect("should not fail"));

        db.set_access_in_network_registry(None, *ADDR_1, true)
            .await
            .expect("should not fail to allow in nr");

        assert!(db
            .is_allowed_in_network_registry(None, *ADDR_1)
            .await
            .expect("should not fail"));
        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_2)
            .await
            .expect("should not fail"));

        db.set_access_in_network_registry(None, *ADDR_1, true)
            .await
            .expect("should not fail to allow in nr when allowed");

        assert!(db
            .is_allowed_in_network_registry(None, *ADDR_1)
            .await
            .expect("should not fail"));
        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_2)
            .await
            .expect("should not fail"));

        db.set_access_in_network_registry(None, *ADDR_1, false)
            .await
            .expect("should fail to deny in nr");

        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_1)
            .await
            .expect("should not fail"));
        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_2)
            .await
            .expect("should not fail"));

        db.set_access_in_network_registry(None, *ADDR_1, false)
            .await
            .expect("should fail to deny in nr when denied");

        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_1)
            .await
            .expect("should not fail"));
        assert!(!db
            .is_allowed_in_network_registry(None, *ADDR_2)
            .await
            .expect("should not fail"));
    }

    #[async_std::test]
    async fn test_network_eligiblity_db() {
        let db = HoprDb::new_in_memory().await;

        assert!(!db.is_safe_eligible(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_safe_eligible(None, *ADDR_2).await.expect("should not fail"));

        db.set_safe_eligibility(None, *ADDR_1, true)
            .await
            .expect("should not fail to allow in nr");

        assert!(db.is_safe_eligible(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_safe_eligible(None, *ADDR_2).await.expect("should not fail"));

        db.set_safe_eligibility(None, *ADDR_1, true)
            .await
            .expect("should not fail to allow in nr when allowed");

        assert!(db.is_safe_eligible(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_safe_eligible(None, *ADDR_2).await.expect("should not fail"));

        db.set_safe_eligibility(None, *ADDR_1, false)
            .await
            .expect("should fail to deny in nr");

        assert!(!db.is_safe_eligible(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_safe_eligible(None, *ADDR_2).await.expect("should not fail"));

        db.set_safe_eligibility(None, *ADDR_1, false)
            .await
            .expect("should fail to deny in nr when denied");

        assert!(!db.is_safe_eligible(None, *ADDR_1).await.expect("should not fail"));
        assert!(!db.is_safe_eligible(None, *ADDR_2).await.expect("should not fail"));
    }
}
