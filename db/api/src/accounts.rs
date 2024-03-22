use async_trait::async_trait;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::AccountEntry;
use multiaddr::Multiaddr;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, Related, Set};
use sea_query::{Condition, IntoCondition, OnConflict};

use hopr_db_entity::prelude::{Account, Announcement};
use hopr_db_entity::{account, announcement};
use hopr_internal_types::account::AccountType;
use hopr_primitive_types::errors::GeneralError;
use hopr_primitive_types::prelude::{Address, ToHex};

use crate::db::HoprDb;
use crate::errors::DbError::MissingAccount;
use crate::errors::{DbError, Result};
use crate::{HoprDbGeneralModelOperations, OptTx};

/// A type that can represent both [chain public key](Address) and [packet public key](OffchainPublicKey).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChainOrPacketKey {
    /// Represents [chain public key](Address).
    ChainKey(Address),
    /// Represents [packet public key](OffchainPublicKey).
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

impl IntoCondition for ChainOrPacketKey {
    fn into_condition(self) -> Condition {
        match self {
            ChainOrPacketKey::ChainKey(chain_key) => {
                account::Column::ChainKey.eq(chain_key.to_string()).into_condition()
            }
            ChainOrPacketKey::PacketKey(packet_key) => {
                account::Column::PacketKey.eq(packet_key.to_string()).into_condition()
            }
        }
    }
}

/// Defines DB API for accessing HOPR accounts and corresponding on-chain announcements.
///
/// Accounts store the Chain and Packet key information, so as the
/// routable network information, if the account has been announced as well.
#[async_trait]
pub trait HoprDbAccountOperations {
    /// Retrieves the account entry using a Packet key or Chain key.
    async fn get_account<'a, T>(&'a self, tx: OptTx<'a>, key: T) -> Result<Option<AccountEntry>>
    where
        T: Into<ChainOrPacketKey> + Send + Sync;

    /// Retrieves account entry about this node's account.
    /// This a unique account in the database that must always be present.
    async fn get_self_account<'a>(&'a self, tx: OptTx<'a>) -> Result<AccountEntry>;

    /// Retrieves entries of accounts with routable address announcements (if `public_only` is `true`)
    /// or about all accounts without routeable address announcements (if `public_only` is `false`).
    async fn get_accounts<'a>(&'a self, tx: OptTx<'a>, public_only: bool) -> Result<Vec<AccountEntry>>;

    /// Inserts new account entry to the database.
    /// Fails if such entry already exists.
    async fn insert_account<'a>(&'a self, tx: OptTx<'a>, account: AccountEntry) -> Result<()>;

    /// Inserts routable address announcement linked to a specific entry.
    ///
    /// If an account matching the given `key` (chain or off-chain key) does not exist, an
    /// error is returned.
    /// If such `multiaddr` has been already announced for the given account `key`, only
    /// the `at_block` will be updated on that announcement.
    async fn insert_announcement<'a, T>(
        &'a self,
        tx: OptTx<'a>,
        key: T,
        multiaddr: Multiaddr,
        at_block: u32,
    ) -> Result<AccountEntry>
    where
        T: Into<ChainOrPacketKey> + Send + Sync;

    /// Deletes all address announcements for the given account.
    async fn delete_all_announcements<'a, T>(&'a self, tx: OptTx<'a>, key: T) -> Result<()>
    where
        T: Into<ChainOrPacketKey> + Send + Sync;

    /// Deletes account with the given `key` (chain or off-chain).
    async fn delete_account<'a, T>(&'a self, tx: OptTx<'a>, key: T) -> Result<()>
    where
        T: Into<ChainOrPacketKey> + Send + Sync;

    /// Translates the given Chain or Packet key to its counterpart.
    ///
    /// If `Address` is given as `key`, the result will contain `OffchainPublicKey` if present.
    /// If `OffchainPublicKey` is given as `key`, the result will contain `Address` if present.
    async fn translate_key<'a, T>(&'a self, tx: OptTx<'a>, key: T) -> Result<Option<ChainOrPacketKey>>
    where
        T: Into<ChainOrPacketKey> + Send + Sync;
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
                multiaddr: a.multiaddress.parse().map_err(|_| DbError::DecodingError)?,
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
                        .filter(cpk)
                        .order_by_desc(announcement::Column::AtBlock)
                        .all(tx.as_ref())
                        .await?
                        .pop();

                    Ok::<_, DbError>(if let Some((account, announcements)) = maybe_model {
                        Some(model_to_account_entry(account, announcements)?)
                    } else {
                        None
                    })
                })
            })
            .await
    }

    async fn get_self_account<'a>(&'a self, tx: OptTx<'a>) -> Result<AccountEntry> {
        self.get_account(tx, self.me_onchain)
            .await?
            .ok_or(DbError::MissingAccount)
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

    async fn insert_account<'a>(&'a self, tx: OptTx<'a>, account: AccountEntry) -> Result<()> {
        let myself = self.clone();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    match account::Entity::insert(account::ActiveModel {
                        chain_key: Set(account.chain_addr.to_hex()),
                        packet_key: Set(account.public_key.to_hex()),
                        ..Default::default()
                    })
                    .on_conflict(
                        OnConflict::columns([account::Column::ChainKey, account::Column::PacketKey])
                            .do_nothing()
                            .to_owned(),
                    )
                    .exec(tx.as_ref())
                    .await
                    {
                        // Proceed if succeeded or already exists
                        Ok(_) | Err(DbErr::RecordNotInserted) => {
                            myself.caches.chain_to_offchain.insert(account.chain_addr, Some(account.public_key)).await;
                            myself.caches.offchain_to_chain.insert(account.public_key, Some(account.chain_addr)).await;

                            if let AccountType::Announced {
                                multiaddr,
                                updated_block,
                            } = account.entry_type
                            {
                                myself
                                    .insert_announcement(Some(tx), account.chain_addr, multiaddr, updated_block)
                                    .await?;
                            }
                            Ok::<(), DbError>(())
                        }
                        Err(e) => Err(e.into()),
                    }
                })
            })
            .await
    }

    async fn insert_announcement<'a, T>(
        &'a self,
        tx: OptTx<'a>,
        key: T,
        multiaddr: Multiaddr,
        at_block: u32,
    ) -> Result<AccountEntry>
    where
        T: Into<ChainOrPacketKey> + Send + Sync,
    {
        let cpk = key.into();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let (existing_account, mut existing_announcements) = account::Entity::find()
                        .find_with_related(announcement::Entity)
                        .filter(cpk)
                        .order_by_desc(announcement::Column::AtBlock)
                        .all(tx.as_ref())
                        .await?
                        .pop()
                        .ok_or(MissingAccount)?;

                    if let Some((index, _)) = existing_announcements
                        .iter()
                        .enumerate()
                        .find(|(_, announcement)| announcement.multiaddress == multiaddr.to_string())
                    {
                        let mut existing_announcement = existing_announcements.remove(index).into_active_model();
                        existing_announcement.at_block = Set(at_block as i32);
                        let updated_announcement = existing_announcement.update(tx.as_ref()).await?;

                        // To maintain the sort order, insert at the original location
                        existing_announcements.insert(index, updated_announcement);
                    } else {
                        let new_announcement = announcement::ActiveModel {
                            account_id: Set(existing_account.id),
                            multiaddress: Set(multiaddr.to_string()),
                            at_block: Set(at_block as i32),
                            ..Default::default()
                        }
                        .insert(tx.as_ref())
                        .await?;

                        // Assuming this is the latest announcement, so prepend it
                        existing_announcements.insert(0, new_announcement);
                    }

                    model_to_account_entry(existing_account, existing_announcements)
                })
            })
            .await
    }

    async fn delete_all_announcements<'a, T>(&'a self, tx: OptTx<'a>, key: T) -> Result<()>
    where
        T: Into<ChainOrPacketKey> + Send + Sync,
    {
        let cpk = key.into();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let to_delete = account::Entity::find_related()
                        .filter(cpk)
                        .all(tx.as_ref())
                        .await?
                        .into_iter()
                        .map(|x| x.id)
                        .collect::<Vec<_>>();

                    if !to_delete.is_empty() {
                        announcement::Entity::delete_many()
                            .filter(announcement::Column::Id.is_in(to_delete))
                            .exec(tx.as_ref())
                            .await?;

                        Ok::<_, DbError>(())
                    } else {
                        Err(MissingAccount)
                    }
                })
            })
            .await
    }

    async fn delete_account<'a, T>(&'a self, tx: OptTx<'a>, key: T) -> Result<()>
    where
        T: Into<ChainOrPacketKey> + Send + Sync,
    {
        let myself = self.clone();
        let cpk = key.into();
        self.nest_transaction(tx)
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    if let Some(entry) = account::Entity::find()
                        .filter(cpk)
                        .one(tx.as_ref())
                        .await? {
                        let account_entry = model_to_account_entry(entry.clone(), vec![])?;
                        entry.delete(tx.as_ref()).await?;

                        myself.caches.chain_to_offchain.invalidate(&account_entry.chain_addr).await;
                        myself.caches.offchain_to_chain.invalidate(&account_entry.public_key).await;
                        Ok::<_, DbError>(())
                    } else {
                        Err(MissingAccount)
                    }
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
    use crate::errors::DbError::DecodingError;
    use crate::HoprDbGeneralModelOperations;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_internal_types::prelude::AccountType::NotAnnounced;

    #[async_std::test]
    async fn test_insert_account_announcement() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = OffchainKeypair::random().public().clone();

        db.insert_account(None, AccountEntry::new(packet_1, chain_1, AccountType::NotAnnounced))
            .await
            .unwrap();

        let acc = db
            .get_account(None, chain_1)
            .await
            .unwrap()
            .expect("should contain account");
        assert_eq!(packet_1, acc.public_key, "pub keys must match");
        assert_eq!(AccountType::NotAnnounced, acc.entry_type.clone());

        let maddr: Multiaddr = "/ip4/1.2.3.4/tcp/8000".parse().unwrap();
        let block = 100;

        let db_acc = db
            .insert_announcement(None, chain_1, maddr.clone(), block)
            .await
            .expect("should insert announcement");

        let acc = db
            .get_account(None, chain_1)
            .await
            .unwrap()
            .expect("should contain account");
        assert_eq!(Some(maddr.clone()), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc);

        let block = 200;
        let db_acc = db
            .insert_announcement(None, chain_1, maddr.clone(), block)
            .await
            .expect("should insert duplicate announcement");

        let acc = db
            .get_account(None, chain_1)
            .await
            .unwrap()
            .expect("should contain account");
        assert_eq!(Some(maddr), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc);

        let maddr: Multiaddr = "/dns4/useful.domain/tcp/56".parse().unwrap();
        let block = 300;
        let db_acc = db
            .insert_announcement(None, chain_1, maddr.clone(), block)
            .await
            .expect("should insert updated announcement");

        let acc = db
            .get_account(None, chain_1)
            .await
            .unwrap()
            .expect("should contain account");
        assert_eq!(Some(maddr), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc);
    }

    #[async_std::test]
    async fn test_should_not_insert_account_announcement_to_nonexisting_account() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();

        let maddr: Multiaddr = "/ip4/1.2.3.4/tcp/8000".parse().unwrap();
        let block = 100;

        let r = db.insert_announcement(None, chain_1, maddr.clone(), block).await;
        assert!(
            matches!(r, Err(MissingAccount)),
            "should not insert announcement to non-existing account"
        )
    }

    #[async_std::test]
    async fn test_should_allow_duplicate_announcement_per_different_accounts() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = OffchainKeypair::random().public().clone();

        db.insert_account(None, AccountEntry::new(packet_1, chain_1, AccountType::NotAnnounced))
            .await
            .unwrap();

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = OffchainKeypair::random().public().clone();

        db.insert_account(None, AccountEntry::new(packet_2, chain_2, AccountType::NotAnnounced))
            .await
            .unwrap();

        let maddr: Multiaddr = "/ip4/1.2.3.4/tcp/8000".parse().unwrap();
        let block = 100;

        let db_acc_1 = db
            .insert_announcement(None, chain_1, maddr.clone(), block)
            .await
            .expect("should insert announcement");
        let db_acc_2 = db
            .insert_announcement(None, chain_2, maddr.clone(), block)
            .await
            .expect("should insert announcement");

        let acc = db
            .get_account(None, chain_1)
            .await
            .unwrap()
            .expect("should contain account");
        assert_eq!(Some(maddr.clone()), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc_1);

        let acc = db
            .get_account(None, chain_2)
            .await
            .unwrap()
            .expect("should contain account");
        assert_eq!(Some(maddr.clone()), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc_2);
    }

    #[async_std::test]
    async fn test_delete_account() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        db.insert_account(
            None,
            AccountEntry::new(
                OffchainKeypair::random().public().clone(),
                chain_1,
                AccountType::Announced {
                    multiaddr: "/ip4/1.2.3.4/tcp/1234".parse().unwrap(),
                    updated_block: 10,
                },
            ),
        )
        .await
        .unwrap();

        assert!(db.get_account(None, chain_1).await.unwrap().is_some());

        db.delete_account(None, chain_1)
            .await
            .expect("should not fail to delete");

        assert!(db.get_account(None, chain_1).await.unwrap().is_none());
    }

    #[async_std::test]
    async fn test_should_fail_to_delete_nonexistent_account() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();

        let r = db.delete_account(None, chain_1).await;
        assert!(
            matches!(r, Err(MissingAccount)),
            "should not delete non-existing account"
        )
    }

    #[async_std::test]
    async fn test_should_not_fail_on_duplicate_account_insert() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = OffchainKeypair::random().public().clone();

        db.insert_account(None, AccountEntry::new(packet_1, chain_1, AccountType::NotAnnounced))
            .await
            .unwrap();

        db.insert_account(None, AccountEntry::new(packet_1, chain_1, AccountType::NotAnnounced))
            .await
            .expect("should not fail the second time");
    }

    #[async_std::test]
    async fn test_delete_announcements() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let mut entry = AccountEntry::new(
            OffchainKeypair::random().public().clone(),
            chain_1,
            AccountType::Announced {
                multiaddr: "/ip4/1.2.3.4/tcp/1234".parse().unwrap(),
                updated_block: 10,
            },
        );

        db.insert_account(None, entry.clone()).await.unwrap();

        assert_eq!(Some(entry.clone()), db.get_account(None, chain_1).await.unwrap());

        db.delete_all_announcements(None, chain_1).await.unwrap();

        entry.entry_type = NotAnnounced;

        assert_eq!(Some(entry), db.get_account(None, chain_1).await.unwrap());
    }

    #[async_std::test]
    async fn test_should_fail_to_delete_nonexistent_account_announcements() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();

        let r = db.delete_all_announcements(None, chain_1).await;
        assert!(
            matches!(r, Err(MissingAccount)),
            "should not delete non-existing account"
        )
    }

    #[async_std::test]
    async fn test_translate_key() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

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
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry::new(packet_1, chain_1, AccountType::NotAnnounced),
                        )
                        .await?;
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry::new(packet_2, chain_2, AccountType::NotAnnounced),
                        )
                        .await?;
                    Ok::<(), DbError>(())
                })
            })
            .await
            .expect("tx should not fail");

        let a: Address = db
            .translate_key(None, packet_1)
            .await
            .expect("must translate")
            .expect("must contain key")
            .try_into()
            .expect("must be chain key");

        let b: OffchainPublicKey = db
            .translate_key(None, chain_2)
            .await
            .expect("must translate")
            .expect("must contain key")
            .try_into()
            .expect("must be chain key");

        assert_eq!(chain_1, a, "chain keys must match");
        assert_eq!(packet_2, b, "chain keys must match");
    }

    #[async_std::test]
    async fn test_get_accounts() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let chain_1 = ChainKeypair::random().public().to_address();
        let chain_2 = ChainKeypair::random().public().to_address();
        let chain_3 = ChainKeypair::random().public().to_address();

        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry::new(*OffchainKeypair::random().public(), chain_1, AccountType::NotAnnounced),
                        )
                        .await?;
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry::new(
                                *OffchainKeypair::random().public(),
                                chain_2,
                                AccountType::Announced {
                                    multiaddr: "/ip4/10.10.10.10/tcp/1234".parse().map_err(|_| DecodingError)?,
                                    updated_block: 10,
                                },
                            ),
                        )
                        .await?;
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry::new(*OffchainKeypair::random().public(), chain_3, AccountType::NotAnnounced),
                        )
                        .await?;

                    db_clone
                        .insert_announcement(Some(tx), chain_3, "/ip4/1.2.3.4/tcp/1234".parse().unwrap(), 12)
                        .await?;
                    db_clone
                        .insert_announcement(Some(tx), chain_3, "/ip4/8.8.1.1/tcp/1234".parse().unwrap(), 15)
                        .await?;
                    db_clone
                        .insert_announcement(Some(tx), chain_3, "/ip4/1.2.3.0/tcp/234".parse().unwrap(), 14)
                        .await
                })
            })
            .await
            .expect("must insert announcements");

        let all_accounts = db.get_accounts(None, false).await.expect("must get all");
        let public_only = db.get_accounts(None, true).await.expect("must get public");

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

        assert_eq!("/ip4/10.10.10.10/tcp/1234", acc_1.get_multiaddr().unwrap().to_string());
        assert_eq!("/ip4/8.8.1.1/tcp/1234", acc_2.get_multiaddr().unwrap().to_string());
    }
}
