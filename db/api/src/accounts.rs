use async_trait::async_trait;
use futures::TryStreamExt;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::AccountEntry;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Related};

use hopr_db_entity::prelude::Account;
use hopr_db_entity::{account, announcement};
use hopr_internal_types::account::AccountType;
use hopr_primitive_types::prelude::{Address, GeneralError, ToHex};

use crate::db::HoprDb;
use crate::errors::DbError::{CorruptedData, NotFound};
use crate::errors::Result;

pub enum ChainOrPacketKey {
    ChainKey(Address),
    PacketKey(OffchainPublicKey),
}

impl From<Address> for ChainOrPacketKey {
    fn from(value: Address) -> Self {
        Self::ChainKey(value)
    }
}

impl From<OffchainPublicKey> for ChainOrPacketKey {
    fn from(value: OffchainPublicKey) -> Self {
        Self::PacketKey(value)
    }
}

impl TryFrom<ChainOrPacketKey> for OffchainPublicKey {
    type Error = GeneralError;

    fn try_from(value: ChainOrPacketKey) -> std::result::Result<Self, Self::Error> {
        match value {
            ChainOrPacketKey::ChainKey(_) => Err(GeneralError::InvalidInput),
            ChainOrPacketKey::PacketKey(k) => Ok(k),
        }
    }
}

impl TryFrom<ChainOrPacketKey> for Address {
    type Error = GeneralError;

    fn try_from(value: ChainOrPacketKey) -> std::result::Result<Self, Self::Error> {
        match value {
            ChainOrPacketKey::ChainKey(k) => Ok(k),
            ChainOrPacketKey::PacketKey(_) => Err(GeneralError::InvalidInput),
        }
    }
}

#[async_trait]
pub trait HoprDbAccountOperations {
    async fn get_accounts(&self, public_only: bool) -> Result<Vec<AccountEntry>>;

    async fn translate_key<T: Into<ChainOrPacketKey> + Send + Sync>(&self, key: T) -> Result<ChainOrPacketKey>;
}

fn model_to_account_entry(account: account::Model, announcement: Option<announcement::Model>) -> Result<AccountEntry> {
    Ok(AccountEntry::new(
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
            ret.push(model_to_account_entry(account, announcement)?);
        }

        Ok(ret)
    }

    async fn translate_key<T: Into<ChainOrPacketKey> + Send + Sync>(&self, key: T) -> Result<ChainOrPacketKey> {
        match key.into() {
            ChainOrPacketKey::ChainKey(chain_key) => {
                let r = Account::find()
                    .filter(account::Column::ChainKey.eq(chain_key.to_string()))
                    .one(&self.db)
                    .await?
                    .ok_or(NotFound)?;
                Ok(OffchainPublicKey::from_hex(&r.packet_key)?.into())
            }
            ChainOrPacketKey::PacketKey(packet_key) => {
                let r = Account::find()
                    .filter(account::Column::PacketKey.eq(packet_key.to_string()))
                    .one(&self.db)
                    .await?
                    .ok_or(NotFound)?;
                Ok(Address::from_hex(&r.chain_key)?.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_db_entity::prelude::Announcement;
    use sea_orm::Set;

    #[async_std::test]
    async fn test_get_account_by_key() {
        let db = HoprDb::new_in_memory().await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = OffchainKeypair::random().public().clone();
        let account_1 = account::ActiveModel {
            chain_key: Set(chain_1.to_hex()),
            packet_key: Set(packet_1.to_hex()),
            ..Default::default()
        };

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = OffchainKeypair::random().public().clone();
        let account_2 = account::ActiveModel {
            chain_key: Set(chain_2.to_hex()),
            packet_key: Set(packet_2.to_hex()),
            ..Default::default()
        };

        let res = Account::insert_many([account_1, account_2])
            .exec(&db.db)
            .await
            .expect("insert must succeed");

        let announcement = announcement::ActiveModel {
            account_id: Set(res.last_insert_id.into()),
            multiaddress: Set("/ip4/10.10.10.10/tcp/1234".into()),
            at_block: Set(10),
            ..Default::default()
        };
        Announcement::insert(announcement)
            .exec(&db.db)
            .await
            .expect("insert must succeed");

        let chain_3 = ChainKeypair::random().public().to_address();
        let packet_3 = OffchainKeypair::random().public().clone();
        let account_3 = account::ActiveModel {
            chain_key: Set(chain_3.to_hex()),
            packet_key: Set(packet_3.to_hex()),
            ..Default::default()
        };

        let res = Account::insert(account_3)
            .exec(&db.db)
            .await
            .expect("insert must succeed");
        let announcement_1 = announcement::ActiveModel {
            account_id: Set(res.last_insert_id.into()),
            multiaddress: Set("/ip4/8.8.1.1/tcp/1234".into()),
            at_block: Set(11),
            ..Default::default()
        };

        let announcement_2 = announcement::ActiveModel {
            account_id: Set(res.last_insert_id.into()),
            multiaddress: Set("/ip4/1.2.3.4/tcp/1234".into()),
            at_block: Set(12),
            ..Default::default()
        };
        Announcement::insert_many([announcement_1, announcement_2])
            .exec(&db.db)
            .await
            .expect("insert must succeed");

        let a: Address = db
            .translate_key(packet_1)
            .await
            .expect("must translate")
            .try_into()
            .expect("must be chain key");
        assert_eq!(chain_1, a, "chain keys must match");
    }
}
