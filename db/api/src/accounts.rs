use async_trait::async_trait;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::AccountEntry;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use hopr_db_entity::prelude::{Account, Announcement};
use hopr_db_entity::{account, announcement};
use hopr_internal_types::account::AccountType;
use hopr_internal_types::ChainOrPacketKey;
use hopr_primitive_types::prelude::{Address, ToHex};

use crate::db::HoprDb;
use crate::errors::DbError::CorruptedData;
use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx};

#[async_trait]
pub trait HoprDbAccountOperations {
    async fn get_account<'a, T: Into<ChainOrPacketKey> + Send + Sync>(
        &'a self,
        tx: OptTx<'a>,
        key: T,
    ) -> Result<Option<AccountEntry>>;

    async fn get_accounts<'a>(&'a self, tx: OptTx<'a>, public_only: bool) -> Result<Vec<AccountEntry>>;

    async fn insert_account<'a>(&'a self, tx: OptTx<'a>, account: AccountEntry) -> Result<()>;

    async fn translate_key<'a, T: Into<ChainOrPacketKey> + Send + Sync>(
        &'a self,
        tx: OptTx<'a>,
        key: T,
    ) -> Result<Option<ChainOrPacketKey>>;
}

// NOTE: this currently function assumes `announcements` are sorted from latest to earliest
fn model_to_account_entry(account: account::Model, announcements: Vec<announcement::Model>) -> Result<AccountEntry> {
    // Currently we always take only the most recent announcement
    let announcement = announcements.first();

    Ok(AccountEntry::new(
        OffchainPublicKey::from_hex(&account.packet_key)?,
        account.chain_key.parse()?,
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
    async fn get_account<'a, T: Into<ChainOrPacketKey> + Send + Sync>(
        &'a self,
        tx: OptTx<'a>,
        key: T,
    ) -> Result<Option<AccountEntry>> {
        let cpk = key.into();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let maybe_model = Account::find()
                        .find_with_related(Announcement)
                        .filter(match cpk {
                            ChainOrPacketKey::ChainKey(chain_key) => {
                                account::Column::ChainKey.eq(chain_key.to_string())
                            }
                            ChainOrPacketKey::PacketKey(packet_key) => {
                                account::Column::PacketKey.eq(packet_key.to_string())
                            }
                        })
                        .all(tx.as_ref())
                        .await?
                        .pop();

                    Ok::<Option<AccountEntry>, DbError>(if let Some((account, announcements)) = maybe_model {
                        Some(model_to_account_entry(account, announcements)?)
                    } else {
                        None
                    })
                })
            })
            .await
    }

    async fn insert_account<'a>(&'a self, tx: OptTx<'a>, account: AccountEntry) -> Result<()> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let new_account = account::ActiveModel {
                        chain_key: Set(account.chain_addr.to_hex()),
                        packet_key: Set(account.public_key.to_hex()),
                        ..Default::default()
                    }
                    .insert(tx.as_ref())
                    .await?;

                    if let AccountType::Announced {
                        multiaddr,
                        updated_block,
                    } = account.entry_type
                    {
                        announcement::ActiveModel {
                            account_id: Set(new_account.id),
                            multiaddress: Set(multiaddr.to_string()),
                            at_block: Set(updated_block as i32),
                            ..Default::default()
                        }
                        .insert(tx.as_ref())
                        .await?;
                    }

                    Ok::<(), DbError>(())
                })
            })
            .await
    }

    async fn get_accounts<'a>(&'a self, tx: OptTx<'a>, public_only: bool) -> Result<Vec<AccountEntry>> {
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Account::find()
                        .find_with_related(Announcement)
                        .filter(if public_only {
                            announcement::Column::Multiaddress.ne("")
                        } else {
                            Expr::value(1)
                        })
                        .order_by_desc(announcement::Column::AtBlock)
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(|(a, b)| model_to_account_entry(a, b))
                        .collect()
                })
            })
            .await
    }

    async fn translate_key<'a, T: Into<ChainOrPacketKey> + Send + Sync>(
        &'a self,
        tx: OptTx<'a>,
        key: T,
    ) -> Result<Option<ChainOrPacketKey>> {
        let cpk = key.into();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    Ok::<_, DbError>(match cpk {
                        ChainOrPacketKey::ChainKey(chain_key) => {
                            let maybe_model = Account::find()
                                .filter(account::Column::ChainKey.eq(chain_key.to_string()))
                                .one(tx.as_ref())
                                .await?;
                            if let Some(m) = maybe_model {
                                Some(OffchainPublicKey::from_hex(&m.packet_key)?.into())
                            } else {
                                None
                            }
                        }
                        ChainOrPacketKey::PacketKey(packet_key) => {
                            let maybe_model = Account::find()
                                .filter(account::Column::PacketKey.eq(packet_key.to_string()))
                                .one(tx.as_ref())
                                .await?;
                            if let Some(m) = maybe_model {
                                Some(Address::from_hex(&m.chain_key)?.into())
                            } else {
                                None
                            }
                        }
                    })
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::DbError;
    use crate::HoprDbGeneralModelOperations;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_db_entity::prelude::Announcement;
    use sea_orm::Set;

    #[async_std::test]
    async fn test_translate_key() {
        let db = HoprDb::new_in_memory().await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = OffchainKeypair::random().public().clone();

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = OffchainKeypair::random().public().clone();

        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    let entry = AccountEntry::new(packet_1, chain_1, AccountType::NotAnnounced);
                    db_clone.insert_account(Some(tx), entry).await?;

                    let entry = AccountEntry::new(packet_2, chain_2, AccountType::NotAnnounced);
                    db_clone.insert_account(Some(tx), entry).await?;
                    Ok::<(), DbError>(())
                })
            })
            .await
            .expect("tx should not fail");

        let (a, b) = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    let a: Address = db
                        .translate_key(Some(tx), packet_1)
                        .await
                        .expect("must translate")
                        .expect("must contain key")
                        .try_into()
                        .expect("must be chain key");

                    let b: OffchainPublicKey = db
                        .translate_key(Some(tx), chain_2)
                        .await
                        .expect("must translate")
                        .expect("must contain key")
                        .try_into()
                        .expect("must be chain key");
                    Ok::<_, DbError>((a, b))
                })
            })
            .await
            .unwrap();

        assert_eq!(chain_1, a, "chain keys must match");
        assert_eq!(packet_2, b, "chain keys must match");
    }

    #[async_std::test]
    async fn test_get_accounts() {
        let db = HoprDb::new_in_memory().await;

        let tx = db.begin_transaction().await.unwrap();

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
            .exec(tx.as_ref())
            .await
            .expect("insert must succeed");

        let ma_1 = "/ip4/10.10.10.10/tcp/1234";
        let announcement = announcement::ActiveModel {
            account_id: Set(res.last_insert_id.into()),
            multiaddress: Set(ma_1.into()),
            at_block: Set(10),
            ..Default::default()
        };
        Announcement::insert(announcement)
            .exec(tx.as_ref())
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
            .exec(tx.as_ref())
            .await
            .expect("insert must succeed");

        let announcement_1 = announcement::ActiveModel {
            account_id: Set(res.last_insert_id.into()),
            multiaddress: Set("/ip4/1.2.3.4/tcp/1234".into()),
            at_block: Set(12),
            ..Default::default()
        };

        let ma_2 = "/ip4/8.8.1.1/tcp/1234";
        let announcement_2 = announcement::ActiveModel {
            account_id: Set(res.last_insert_id.into()),
            multiaddress: Set(ma_2.into()),
            at_block: Set(15),
            ..Default::default()
        };

        let announcement_3 = announcement::ActiveModel {
            account_id: Set(res.last_insert_id.into()),
            multiaddress: Set("/ip4/1.2.3.0/tcp/234".into()),
            at_block: Set(14),
            ..Default::default()
        };

        Announcement::insert_many([announcement_1, announcement_2, announcement_3])
            .exec(tx.as_ref())
            .await
            .expect("insert must succeed");

        tx.commit().await.unwrap();

        let (all_accounts, public_only) = db
            .begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    let all_accounts = db
                        .get_accounts(Some(tx.into()), false)
                        .await
                        .expect("should get accounts");
                    let public_only = db
                        .get_accounts(Some(tx.into()), true)
                        .await
                        .expect("should get accounts");
                    Ok::<_, DbError>((all_accounts, public_only))
                })
            })
            .await
            .unwrap();

        assert_eq!(3, all_accounts.len());

        assert_eq!(2, public_only.len());
        let acc_1 = public_only
            .iter()
            .find(|a| a.chain_addr.eq(&chain_2))
            .expect("should contain 1");
        let acc_2 = public_only
            .iter()
            .find(|a| a.chain_addr.eq(&chain_3))
            .expect("should contain 2");

        assert_eq!(ma_1, acc_1.get_multiaddr().unwrap().to_string());
        assert_eq!(ma_2, acc_2.get_multiaddr().unwrap().to_string());
    }
}
