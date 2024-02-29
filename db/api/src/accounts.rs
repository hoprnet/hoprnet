use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::AccountEntry;
use sea_orm::EntityTrait;

use hopr_db_entity::announcement;
use hopr_db_entity::prelude::Account;
use hopr_internal_types::account::AccountType;
use hopr_primitive_types::prelude::{Address, ToHex};

use crate::db::HoprDb;
use crate::errors::DbError::CorruptedData;
use crate::errors::Result;

#[async_trait]
pub trait HoprDbAccountOperations {
    async fn get_accounts(&self, public_only: bool) -> Result<Vec<AccountEntry>>;
}

#[async_trait]
impl HoprDbAccountOperations for HoprDb {
    async fn get_accounts(&self, public_only: bool) -> Result<Vec<AccountEntry>> {
        let mut s = if public_only {
            Account::find()
                .inner_join(announcement::Entity)
                .select_also(announcement::Entity)
                .stream(&self.db)
                .await?
        } else {
            Account::find()
                .left_join(announcement::Entity)
                .select_also(announcement::Entity)
                .stream(&self.db)
                .await?
        };

        let mut ret = Vec::new();
        while let Some((account, announcement)) = s.try_next().await? {
            ret.push(AccountEntry::new(
                OffchainPublicKey::from_hex(&account.packet_key)?,
                Address::from_hex(&account.chain_key)?,
                match announcement {
                    None => AccountType::NotAnnounced,
                    Some(a) => AccountType::Announced {
                        multiaddr: a.multiaddress.parse().map_err(|_| CorruptedData)?,
                        updated_block: a.at_block as u32,
                    },
                },
            ))
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {}
