use async_trait::async_trait;
use futures::TryFutureExt;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_db_entity::{
    account, announcement,
    prelude::{Account, Announcement},
};
use hopr_internal_types::{account::AccountType, prelude::AccountEntry};
use hopr_primitive_types::{
    errors::GeneralError,
    prelude::{Address, ToHex},
};
use multiaddr::Multiaddr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, Related,
    Set, sea_query::Expr,
};
use sea_query::{Condition, IntoCondition, OnConflict};
use tracing::instrument;

use crate::{
    HoprDbGeneralModelOperations, OptTx,
    db::HoprDb,
    errors::{DbSqlError, DbSqlError::MissingAccount, Result},
};

/// A type that can represent both [chain public key](Address) and [packet public key](OffchainPublicKey).
#[allow(clippy::large_enum_variant)] // TODO: use CompactOffchainPublicKey
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
/// routable network information if the account has been announced as well.
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

    /// Inserts a new account entry to the database.
    /// Fails if such an entry already exists.
    async fn insert_account<'a>(&'a self, tx: OptTx<'a>, account: AccountEntry) -> Result<()>;

    /// Inserts a routable address announcement linked to a specific entry.
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

// NOTE: this function currently assumes `announcements` are sorted from latest to earliest
pub(crate) fn model_to_account_entry(
    account: account::Model,
    announcements: Vec<announcement::Model>,
) -> Result<AccountEntry> {
    // Currently, we always take only the most recent announcement
    let announcement = announcements.first();

    Ok(AccountEntry {
        public_key: OffchainPublicKey::from_hex(&account.packet_key)?,
        chain_addr: account.chain_key.parse()?,
        published_at: account.published_at as u32,
        entry_type: match announcement {
            None => AccountType::NotAnnounced,
            Some(a) => AccountType::Announced {
                multiaddr: a.multiaddress.parse().map_err(|_| DbSqlError::DecodingError)?,
                updated_block: a.at_block as u32,
            },
        },
    })
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

                    Ok::<_, DbSqlError>(if let Some((account, announcements)) = maybe_model {
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
            .ok_or(DbSqlError::MissingAccount)
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
                        published_at: Set(account.published_at as i32),
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
                        res @ Ok(_) | res @ Err(DbErr::RecordNotInserted) => {
                            myself
                                .caches
                                .chain_to_offchain
                                .insert(account.chain_addr, Some(account.public_key))
                                .await;
                            myself
                                .caches
                                .offchain_to_chain
                                .insert(account.public_key, Some(account.chain_addr))
                                .await;

                            // Update key-id binding only if the account was inserted successfully
                            // (= not re-announced)
                            if res.is_ok() {
                                if let Err(error) = myself.caches.key_id_mapper.update_key_id_binding(&account) {
                                    tracing::warn!(?account, %error, "keybinding not updated")
                                }
                            }

                            if let AccountType::Announced {
                                multiaddr,
                                updated_block,
                            } = account.entry_type
                            {
                                myself
                                    .insert_announcement(Some(tx), account.chain_addr, multiaddr, updated_block)
                                    .await?;
                            }

                            Ok::<(), DbSqlError>(())
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

                        Ok::<_, DbSqlError>(())
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
                    if let Some(entry) = account::Entity::find().filter(cpk).one(tx.as_ref()).await? {
                        let account_entry = model_to_account_entry(entry.clone(), vec![])?;
                        entry.delete(tx.as_ref()).await?;

                        myself
                            .caches
                            .chain_to_offchain
                            .invalidate(&account_entry.chain_addr)
                            .await;
                        myself
                            .caches
                            .offchain_to_chain
                            .invalidate(&account_entry.public_key)
                            .await;
                        Ok::<_, DbSqlError>(())
                    } else {
                        Err(MissingAccount)
                    }
                })
            })
            .await
    }

    #[instrument(level = "trace", skip_all, err)]
    async fn translate_key<'a, T: Into<ChainOrPacketKey> + Send + Sync>(
        &'a self,
        tx: OptTx<'a>,
        key: T,
    ) -> Result<Option<ChainOrPacketKey>> {
        Ok(match key.into() {
            ChainOrPacketKey::ChainKey(chain_key) => self
                .caches
                .chain_to_offchain
                .try_get_with_by_ref(
                    &chain_key,
                    self.nest_transaction(tx).and_then(|op| {
                        tracing::warn!(?chain_key, "cache miss on chain key lookup");
                        op.perform(|tx| {
                            Box::pin(async move {
                                let maybe_model = Account::find()
                                    .filter(account::Column::ChainKey.eq(chain_key.to_string()))
                                    .one(tx.as_ref())
                                    .await?;
                                if let Some(m) = maybe_model {
                                    Ok(Some(OffchainPublicKey::from_hex(&m.packet_key)?))
                                } else {
                                    Ok(None)
                                }
                            })
                        })
                    }),
                )
                .await?
                .map(ChainOrPacketKey::PacketKey),
            ChainOrPacketKey::PacketKey(packet_key) => self
                .caches
                .offchain_to_chain
                .try_get_with_by_ref(
                    &packet_key,
                    self.nest_transaction(tx).and_then(|op| {
                        tracing::warn!(?packet_key, "cache miss on packet key lookup");
                        op.perform(|tx| {
                            Box::pin(async move {
                                let maybe_model = Account::find()
                                    .filter(account::Column::PacketKey.eq(packet_key.to_string()))
                                    .one(tx.as_ref())
                                    .await?;
                                if let Some(m) = maybe_model {
                                    Ok(Some(Address::from_hex(&m.chain_key)?))
                                } else {
                                    Ok(None)
                                }
                            })
                        })
                    }),
                )
                .await?
                .map(ChainOrPacketKey::ChainKey),
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
    use hopr_internal_types::prelude::AccountType::NotAnnounced;

    use super::*;
    use crate::{
        HoprDbGeneralModelOperations,
        errors::{DbSqlError, DbSqlError::DecodingError},
    };

    #[tokio::test]
    async fn test_insert_account_announcement() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();

        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                published_at: 1,
                entry_type: AccountType::NotAnnounced,
            },
        )
        .await?;

        let acc = db.get_account(None, chain_1).await?.expect("should contain account");
        assert_eq!(packet_1, acc.public_key, "pub keys must match");
        assert_eq!(AccountType::NotAnnounced, acc.entry_type.clone());
        assert_eq!(1, acc.published_at);

        let maddr: Multiaddr = "/ip4/1.2.3.4/tcp/8000".parse()?;
        let block = 100;

        let db_acc = db.insert_announcement(None, chain_1, maddr.clone(), block).await?;

        let acc = db.get_account(None, chain_1).await?.context("should contain account")?;
        assert_eq!(Some(maddr.clone()), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc);

        let block = 200;
        let db_acc = db.insert_announcement(None, chain_1, maddr.clone(), block).await?;

        let acc = db.get_account(None, chain_1).await?.expect("should contain account");
        assert_eq!(Some(maddr), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc);

        let maddr: Multiaddr = "/dns4/useful.domain/tcp/56".parse()?;
        let block = 300;
        let db_acc = db.insert_announcement(None, chain_1, maddr.clone(), block).await?;

        let acc = db.get_account(None, chain_1).await?.expect("should contain account");
        assert_eq!(Some(maddr), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc);

        Ok(())
    }

    #[tokio::test]
    async fn test_should_allow_reannouncement() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();

        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                published_at: 1,
                entry_type: AccountType::NotAnnounced,
            },
        )
        .await?;

        db.insert_announcement(None, chain_1, "/ip4/1.2.3.4/tcp/8000".parse()?, 100)
            .await?;

        let ae = db.get_account(None, chain_1).await?.ok_or(MissingAccount)?;

        assert_eq!(
            "/ip4/1.2.3.4/tcp/8000",
            ae.get_multiaddr().expect("has multiaddress").to_string()
        );

        db.insert_announcement(None, chain_1, "/ip4/1.2.3.4/tcp/8001".parse()?, 110)
            .await?;

        let ae = db.get_account(None, chain_1).await?.ok_or(MissingAccount)?;

        assert_eq!(
            "/ip4/1.2.3.4/tcp/8001",
            ae.get_multiaddr().expect("has multiaddress").to_string()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_should_not_insert_account_announcement_to_nonexisting_account() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();

        let maddr: Multiaddr = "/ip4/1.2.3.4/tcp/8000".parse()?;
        let block = 100;

        let r = db.insert_announcement(None, chain_1, maddr.clone(), block).await;
        assert!(
            matches!(r, Err(MissingAccount)),
            "should not insert announcement to non-existing account"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_should_allow_duplicate_announcement_per_different_accounts() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();

        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                published_at: 1,
                entry_type: AccountType::NotAnnounced,
            },
        )
        .await?;

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = *OffchainKeypair::random().public();

        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_2,
                chain_addr: chain_2,
                published_at: 2,
                entry_type: AccountType::NotAnnounced,
            },
        )
        .await?;

        let maddr: Multiaddr = "/ip4/1.2.3.4/tcp/8000".parse()?;
        let block = 100;

        let db_acc_1 = db.insert_announcement(None, chain_1, maddr.clone(), block).await?;
        let db_acc_2 = db.insert_announcement(None, chain_2, maddr.clone(), block).await?;

        let acc = db.get_account(None, chain_1).await?.expect("should contain account");
        assert_eq!(Some(maddr.clone()), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc_1);

        let acc = db.get_account(None, chain_2).await?.expect("should contain account");
        assert_eq!(Some(maddr.clone()), acc.get_multiaddr(), "multiaddress must match");
        assert_eq!(Some(block), acc.updated_at());
        assert_eq!(acc, db_acc_2);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_account() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let packet_1 = *OffchainKeypair::random().public();
        let chain_1 = ChainKeypair::random().public().to_address();
        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                published_at: 1,
                entry_type: AccountType::Announced {
                    multiaddr: "/ip4/1.2.3.4/tcp/1234".parse()?,
                    updated_block: 10,
                },
            },
        )
        .await?;

        assert!(db.get_account(None, chain_1).await?.is_some());

        db.delete_account(None, chain_1).await?;

        assert!(db.get_account(None, chain_1).await?.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_should_fail_to_delete_nonexistent_account() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();

        let r = db.delete_account(None, chain_1).await;
        assert!(
            matches!(r, Err(MissingAccount)),
            "should not delete non-existing account"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_should_not_fail_on_duplicate_account_insert() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();

        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                published_at: 1,
                entry_type: AccountType::NotAnnounced,
            },
        )
        .await?;

        db.insert_account(
            None,
            AccountEntry {
                public_key: packet_1,
                chain_addr: chain_1,
                published_at: 1,
                entry_type: AccountType::NotAnnounced,
            },
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_announcements() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let packet_1 = *OffchainKeypair::random().public();
        let chain_1 = ChainKeypair::random().public().to_address();
        let mut entry = AccountEntry {
            public_key: packet_1,
            chain_addr: chain_1,
            published_at: 1,
            entry_type: AccountType::Announced {
                multiaddr: "/ip4/1.2.3.4/tcp/1234".parse()?,
                updated_block: 10,
            },
        };

        db.insert_account(None, entry.clone()).await?;

        assert_eq!(Some(entry.clone()), db.get_account(None, chain_1).await?);

        db.delete_all_announcements(None, chain_1).await?;

        entry.entry_type = NotAnnounced;

        assert_eq!(Some(entry), db.get_account(None, chain_1).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_should_fail_to_delete_nonexistent_account_announcements() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();

        let r = db.delete_all_announcements(None, chain_1).await;
        assert!(
            matches!(r, Err(MissingAccount)),
            "should not delete non-existing account"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_translate_key() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = *OffchainKeypair::random().public();

        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .insert_account(
                            tx.into(),
                            AccountEntry {
                                public_key: packet_1,
                                chain_addr: chain_1,
                                published_at: 1,
                                entry_type: AccountType::NotAnnounced,
                            },
                        )
                        .await?;
                    db_clone
                        .insert_account(
                            tx.into(),
                            AccountEntry {
                                public_key: packet_2,
                                chain_addr: chain_2,
                                published_at: 2,
                                entry_type: AccountType::NotAnnounced,
                            },
                        )
                        .await?;
                    Ok::<(), DbSqlError>(())
                })
            })
            .await?;

        let a: Address = db
            .translate_key(None, packet_1)
            .await?
            .context("must contain key")?
            .try_into()?;

        let b: OffchainPublicKey = db
            .translate_key(None, chain_2)
            .await?
            .context("must contain key")?
            .try_into()?;

        assert_eq!(chain_1, a, "chain keys must match");
        assert_eq!(packet_2, b, "chain keys must match");

        Ok(())
    }

    #[tokio::test]
    async fn test_translate_key_no_cache() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();
        let packet_1 = *OffchainKeypair::random().public();

        let chain_2 = ChainKeypair::random().public().to_address();
        let packet_2 = *OffchainKeypair::random().public();

        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .insert_account(
                            tx.into(),
                            AccountEntry {
                                public_key: packet_1,
                                chain_addr: chain_1,
                                published_at: 1,
                                entry_type: AccountType::NotAnnounced,
                            },
                        )
                        .await?;
                    db_clone
                        .insert_account(
                            tx.into(),
                            AccountEntry {
                                public_key: packet_2,
                                chain_addr: chain_2,
                                published_at: 2,
                                entry_type: AccountType::NotAnnounced,
                            },
                        )
                        .await?;
                    Ok::<(), DbSqlError>(())
                })
            })
            .await?;

        db.caches.invalidate_all();

        let a: Address = db
            .translate_key(None, packet_1)
            .await?
            .context("must contain key")?
            .try_into()?;

        let b: OffchainPublicKey = db
            .translate_key(None, chain_2)
            .await?
            .context("must contain key")?
            .try_into()?;

        assert_eq!(chain_1, a, "chain keys must match");
        assert_eq!(packet_2, b, "chain keys must match");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_accounts() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await?;

        let chain_1 = ChainKeypair::random().public().to_address();
        let chain_2 = ChainKeypair::random().public().to_address();
        let chain_3 = ChainKeypair::random().public().to_address();

        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry {
                                public_key: *OffchainKeypair::random().public(),
                                chain_addr: chain_1,
                                entry_type: AccountType::NotAnnounced,
                                published_at: 1,
                            },
                        )
                        .await?;
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry {
                                public_key: *OffchainKeypair::random().public(),
                                chain_addr: chain_2,
                                entry_type: AccountType::Announced {
                                    multiaddr: "/ip4/10.10.10.10/tcp/1234".parse().map_err(|_| DecodingError)?,
                                    updated_block: 10,
                                },
                                published_at: 2,
                            },
                        )
                        .await?;
                    db_clone
                        .insert_account(
                            Some(tx),
                            AccountEntry {
                                public_key: *OffchainKeypair::random().public(),
                                chain_addr: chain_3,
                                entry_type: AccountType::NotAnnounced,
                                published_at: 3,
                            },
                        )
                        .await?;

                    db_clone
                        .insert_announcement(
                            Some(tx),
                            chain_3,
                            "/ip4/1.2.3.4/tcp/1234".parse().map_err(|_| DecodingError)?,
                            12,
                        )
                        .await?;
                    db_clone
                        .insert_announcement(
                            Some(tx),
                            chain_3,
                            "/ip4/8.8.1.1/tcp/1234".parse().map_err(|_| DecodingError)?,
                            15,
                        )
                        .await?;
                    db_clone
                        .insert_announcement(
                            Some(tx),
                            chain_3,
                            "/ip4/1.2.3.0/tcp/234".parse().map_err(|_| DecodingError)?,
                            14,
                        )
                        .await
                })
            })
            .await?;

        let all_accounts = db.get_accounts(None, false).await?;
        let public_only = db.get_accounts(None, true).await?;

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

        assert_eq!(
            "/ip4/10.10.10.10/tcp/1234",
            acc_1.get_multiaddr().expect("should have a multiaddress").to_string()
        );
        assert_eq!(
            "/ip4/8.8.1.1/tcp/1234",
            acc_2.get_multiaddr().expect("should have a multiaddress").to_string()
        );

        Ok(())
    }
}
