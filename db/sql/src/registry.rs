use async_trait::async_trait;
use hopr_db_entity::{chain_info, network_eligibility, network_registry};
use hopr_primitive_types::prelude::{Address, ToHex};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter, Set};
use sea_query::OnConflict;

use crate::{
    HoprDbGeneralModelOperations, OptTx, SINGULAR_TABLE_FIXED_ID,
    db::HoprDb,
    errors::{DbSqlError, Result},
};

/// Defines DB access API for network registry operations.
#[async_trait]
pub trait HoprDbRegistryOperations {
    /// Sets the given node as allowed or denied in network registry.
    async fn set_access_in_network_registry<'a>(&'a self, tx: OptTx<'a>, address: Address, allowed: bool)
    -> Result<()>;

    /// Returns `true` if the given node is allowed in network registry.
    async fn is_allowed_in_network_registry<'a, T>(&'a self, tx: OptTx<'a>, address_like: &T) -> Result<bool>
    where
        Address: TryFrom<T>,
        T: Clone + Sync;

    /// Sets or unsets Safe NR eligibility.
    async fn set_safe_eligibility<'a>(&'a self, tx: OptTx<'a>, address: Address, eligible: bool) -> Result<()>;

    /// Returns `true` if the given Safe is NR eligible.
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
                            Ok(_) | Err(DbErr::RecordNotInserted) => Ok::<_, DbSqlError>(()),
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        network_registry::Entity::delete_many()
                            .filter(network_registry::Column::ChainAddress.eq(address.to_hex()))
                            .exec(tx.as_ref())
                            .await?;
                        Ok::<_, DbSqlError>(())
                    }
                })
            })
            .await
    }

    async fn is_allowed_in_network_registry<'a, T>(&'a self, tx: OptTx<'a>, address_like: &T) -> Result<bool>
    where
        Address: TryFrom<T>,
        T: Clone + Sync,
    {
        let address = Address::try_from((*address_like).clone()).map_err(|_| DbSqlError::DecodingError)?;

        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let is_registry_enabled = chain_info::Entity::find_by_id(SINGULAR_TABLE_FIXED_ID)
                        .one(tx.as_ref())
                        .await?
                        .map(|v| v.network_registry_enabled)
                        .unwrap_or(false);

                    if is_registry_enabled {
                        Ok::<_, DbSqlError>(
                            network_registry::Entity::find()
                                .filter(network_registry::Column::ChainAddress.eq(address.to_hex()))
                                .one(tx.as_ref())
                                .await?
                                .is_some(),
                        )
                    } else {
                        Ok(true)
                    }
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
                            Ok(_) | Err(DbErr::RecordNotInserted) => Ok::<_, DbSqlError>(()),
                            Err(e) => Err(e.into()),
                        }
                    } else {
                        network_eligibility::Entity::delete_many()
                            .filter(network_eligibility::Column::SafeAddress.eq(address.to_hex()))
                            .exec(tx.as_ref())
                            .await?;
                        Ok::<_, DbSqlError>(())
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
                    Ok::<_, DbSqlError>(
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
    use hopr_crypto_types::{keypairs::ChainKeypair, prelude::Keypair};
    use hopr_primitive_types::prelude::Address;
    use lazy_static::lazy_static;

    use crate::{db::HoprDb, registry::HoprDbRegistryOperations};

    lazy_static! {
        static ref ADDR_1: Address = "4331eaa9542b6b034c43090d9ec1c2198758dbc3"
            .parse()
            .expect("lazy static address should be valid");
        static ref ADDR_2: Address = "47d1677e018e79dcdd8a9c554466cb1556fa5007"
            .parse()
            .expect("lazy static address should be valid");
    }

    #[tokio::test]
    async fn test_network_registry_db() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert!(!db.is_allowed_in_network_registry(None, &ADDR_1.as_ref()).await?);
        assert!(!db.is_allowed_in_network_registry(None, &ADDR_2.as_ref()).await?);

        db.set_access_in_network_registry(None, *ADDR_1, true).await?;

        assert!(db.is_allowed_in_network_registry(None, &ADDR_1.as_ref()).await?);
        assert!(!db.is_allowed_in_network_registry(None, &ADDR_2.as_ref()).await?);

        db.set_access_in_network_registry(None, *ADDR_1, true).await?;

        assert!(db.is_allowed_in_network_registry(None, &ADDR_1.as_ref()).await?);
        assert!(!db.is_allowed_in_network_registry(None, &ADDR_2.as_ref()).await?);

        db.set_access_in_network_registry(None, *ADDR_1, false).await?;

        assert!(!db.is_allowed_in_network_registry(None, &ADDR_1.as_ref()).await?);
        assert!(!db.is_allowed_in_network_registry(None, &ADDR_2.as_ref()).await?);

        db.set_access_in_network_registry(None, *ADDR_1, false).await?;

        assert!(!db.is_allowed_in_network_registry(None, &ADDR_1.as_ref()).await?);
        assert!(!db.is_allowed_in_network_registry(None, &ADDR_2.as_ref()).await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_network_eligiblity_db() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        assert!(!db.is_safe_eligible(None, *ADDR_1).await?);
        assert!(!db.is_safe_eligible(None, *ADDR_2).await?);

        db.set_safe_eligibility(None, *ADDR_1, true).await?;

        assert!(db.is_safe_eligible(None, *ADDR_1).await?);
        assert!(!db.is_safe_eligible(None, *ADDR_2).await?);

        db.set_safe_eligibility(None, *ADDR_1, true).await?;

        assert!(db.is_safe_eligible(None, *ADDR_1).await?);
        assert!(!db.is_safe_eligible(None, *ADDR_2).await?);

        db.set_safe_eligibility(None, *ADDR_1, false).await?;

        assert!(!db.is_safe_eligible(None, *ADDR_1).await?);
        assert!(!db.is_safe_eligible(None, *ADDR_2).await?);

        db.set_safe_eligibility(None, *ADDR_1, false).await?;

        assert!(!db.is_safe_eligible(None, *ADDR_1).await?);
        assert!(!db.is_safe_eligible(None, *ADDR_2).await?);
        Ok(())
    }
}
