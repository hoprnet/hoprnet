use crate::{db::HoprdDb, errors::Result};
use async_trait::async_trait;
use hoprd_db_entity::{aliases::Column, types::Alias};
use hoprd_migration::OnConflict;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, TransactionTrait};

pub const ME_AS_ALIAS: &str = "me";

/// Defines DB API for accessing HOPR settings (mainly aliases for now)
#[async_trait]
pub trait HoprdDbAliasesOperations {
    /// Retrieve the peer id for a given alias
    async fn resolve_alias(&self, alias: String) -> Result<Option<String>>;

    /// Retrieve all aliases
    async fn get_aliases(&self) -> Result<Vec<Alias>>;

    /// Create new key pair value in the db. If peer is already aliased, db entry will be updated with the new alias. If `peer` is node's PeerID, throws an error
    async fn set_alias(&self, peer: String, alias: String) -> Result<()>;

    /// Update aliases. If some peers or aliases are already in the db, db entries will be updated with the new aliases. If node's PeerID is among passed aliases, throws an error
    async fn set_aliases(&self, aliases: Vec<Alias>) -> Result<()>;

    /// Delete alias. If not found, throws an error.
    async fn delete_alias(&self, alias: String) -> Result<()>;

    /// Delete all aliases
    async fn clear_aliases(&self) -> Result<()>;
}

#[async_trait]
impl HoprdDbAliasesOperations for HoprdDb {
    async fn resolve_alias(&self, alias: String) -> Result<Option<String>> {
        let row = hoprd_db_entity::aliases::Entity::find()
            .filter(hoprd_db_entity::aliases::Column::Alias.eq(alias))
            .one(&self.metadata)
            .await?;

        Ok(row.map(|model| model.peer_id))
    }

    async fn get_aliases(&self) -> Result<Vec<Alias>> {
        let rows = hoprd_db_entity::aliases::Entity::find().all(&self.metadata).await?;

        let aliases: Vec<Alias> = rows.into_iter().map(Alias::from).collect();

        Ok(aliases)
    }

    async fn set_aliases(&self, aliases: Vec<Alias>) -> Result<()> {
        if let Some(me) = self.resolve_alias(ME_AS_ALIAS.to_string()).await.unwrap() {
            if aliases.iter().any(|entry| entry.peer_id == me) {
                return Err(crate::errors::DbError::LogicalError(
                    "own alias cannot be modified".into(),
                ));
            }
        }

        let new_aliases = aliases
            .into_iter()
            .map(|entry| hoprd_db_entity::aliases::ActiveModel {
                id: Default::default(),
                peer_id: sea_orm::ActiveValue::Set(entry.peer_id.clone()),
                alias: sea_orm::ActiveValue::Set(entry.alias.clone()),
            })
            .collect::<Vec<_>>();

        let _ = hoprd_db_entity::aliases::Entity::insert_many(new_aliases)
            .on_conflict(
                OnConflict::new()
                    .update_columns([Column::PeerId, Column::Alias])
                    .to_owned(),
            )
            .exec(&self.metadata)
            .await;

        Ok(())
    }

    async fn set_alias(&self, peer: String, alias: String) -> Result<()> {
        if let Some(me) = self.resolve_alias(ME_AS_ALIAS.to_string()).await.unwrap() {
            if me == peer {
                return Err(crate::errors::DbError::LogicalError(
                    "own alias cannot be modified".into(),
                ));
            }
        }
        let res = self
            .metadata
            .transaction::<_, _, DbErr>(|tx| {
                Box::pin(async move {
                    let row = hoprd_db_entity::aliases::Entity::find()
                        .filter(hoprd_db_entity::aliases::Column::PeerId.eq(peer.clone()))
                        .one(tx)
                        .await?
                        .map(|model| model.id);

                    let new_alias = hoprd_db_entity::aliases::ActiveModel {
                        peer_id: sea_orm::ActiveValue::Set(peer.clone()),
                        alias: sea_orm::ActiveValue::Set(alias.clone()),
                        id: if let Some(id) = row {
                            sea_orm::ActiveValue::Set(id)
                        } else {
                            sea_orm::ActiveValue::NotSet
                        },
                    };

                    new_alias.save(tx).await?;

                    Ok(())
                })
            })
            .await;

        Ok(res?)
    }

    async fn delete_alias(&self, alias: String) -> Result<()> {
        let res = hoprd_db_entity::aliases::Entity::delete_many()
            .filter(hoprd_db_entity::aliases::Column::Alias.eq(alias))
            .filter(hoprd_db_entity::aliases::Column::Alias.ne(ME_AS_ALIAS.to_string()))
            .exec(&self.metadata)
            .await?;

        if res.rows_affected > 0 {
            Ok(())
        } else {
            Err(crate::errors::DbError::LogicalError(
                "peer cannot be removed because it does not exist or is self".into(),
            ))
        }
    }

    async fn clear_aliases(&self) -> Result<()> {
        let _ = hoprd_db_entity::aliases::Entity::delete_many()
            .filter(hoprd_db_entity::aliases::Column::Alias.ne(ME_AS_ALIAS.to_string()))
            .exec(&self.metadata)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p_identity::PeerId;

    #[async_std::test]
    async fn set_alias_should_succeed() {
        let db = HoprdDb::new_in_memory().await;

        db.set_alias(PeerId::random().to_string(), "test_alias".to_string())
            .await
            .expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
    }

    #[async_std::test]
    async fn set_alias_should_set_multiple_aliases_in_a_transaction() -> Result<()> {
        let db: HoprdDb = HoprdDb::new_in_memory().await;
        let entries = vec![
            Alias {
                peer_id: PeerId::random().to_string(),
                alias: "test_alias".to_string(),
            },
            Alias {
                peer_id: PeerId::random().to_string(),
                alias: "test_alias_2".to_string(),
            },
        ];

        db.set_aliases(entries.clone()).await?;

        let aliases = db.get_aliases().await?;

        assert_eq!(aliases.len(), 2);

        Ok(())
    }

    #[async_std::test]
    async fn set_me_among_other_alias_should_fail() {
        let db = HoprdDb::new_in_memory().await;

        let me_peer_id = PeerId::random().to_string();
        let alias = ME_AS_ALIAS.to_string();

        let res = db.set_alias(me_peer_id.clone(), alias.clone()).await;

        assert!(res.is_ok());

        let entries = vec![
            Alias {
                peer_id: me_peer_id.to_string(),
                alias: "test_alias".to_string(),
            },
            Alias {
                peer_id: PeerId::random().to_string(),
                alias: "test_alias_2".to_string(),
            },
        ];

        let res = db.set_aliases(entries.clone()).await;

        assert!(res.is_err());
    }

    #[async_std::test]
    async fn set_me_alias_should_fail_if_set() {
        let db = HoprdDb::new_in_memory().await;

        let me_peer_id = PeerId::random().to_string();
        let alias = ME_AS_ALIAS.to_string();

        let res = db.set_alias(me_peer_id.clone(), alias.clone()).await;

        assert!(res.is_ok());

        let res = db.set_alias(me_peer_id.clone(), alias.clone()).await;

        assert!(res.is_err());
    }

    #[async_std::test]
    async fn set_aliases_twice() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        db.set_alias(peer_id.clone(), alias.clone().to_uppercase())
            .await
            .expect("should replace alias");

        let aliases = db.get_aliases().await.unwrap();

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].alias, alias.to_uppercase());
    }

    #[async_std::test]
    async fn resolve_alias_should_return_alias() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        let alias = db.resolve_alias(alias).await.expect("should get alias");

        assert!(alias.is_some());
    }

    #[async_std::test]
    async fn resolve_not_stored_alias_should_return_none() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        let alias = db
            .resolve_alias(PeerId::random().to_string())
            .await
            .expect("should get alias");

        assert!(alias.is_none());
    }

    #[async_std::test]
    async fn delete_stored_alias() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");
        let aliases = db.get_aliases().await.expect("should get aliases");
        assert_eq!(aliases.len(), 1);

        db.delete_alias(alias).await.expect("should delete alias");
        let aliases = db.get_aliases().await.expect("should get aliases");
        assert_eq!(aliases.len(), 0);
    }

    #[async_std::test]
    async fn delete_all_aliases() {
        let db = HoprdDb::new_in_memory().await;

        let me_peer_id = PeerId::random().to_string();
        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");
        db.set_alias(me_peer_id.clone(), ME_AS_ALIAS.to_string())
            .await
            .expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");
        assert_eq!(aliases.len(), 2);

        db.clear_aliases()
            .await
            .expect(format!("should clear aliases except '{}'", ME_AS_ALIAS).as_str());
        let aliases = db.get_aliases().await.expect("should get aliases");
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].peer_id, me_peer_id);
    }

    #[async_std::test]
    async fn set_aliases_with_existing_alias_should_replace_peer_id() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        let new_peer_id = PeerId::random().to_string();
        db.set_aliases(vec![Alias {
            peer_id: new_peer_id.clone(),
            alias: alias.clone(),
        }])
        .await
        .expect("should replace peer_id");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].peer_id, new_peer_id);
    }

    #[async_std::test]
    async fn set_aliases_with_existing_peer_id_should_replace_alias() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        db.set_aliases(vec![Alias {
            peer_id: peer_id.clone(),
            alias: alias.clone().to_uppercase(),
        }])
        .await
        .expect("should replace alias");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].alias, alias.to_uppercase());
    }

    #[async_std::test]
    async fn set_aliases_with_existing_entry_shoud_do_nothing() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        db.set_aliases(vec![Alias {
            peer_id: peer_id.clone(),
            alias: alias.clone(),
        }])
        .await
        .expect("should do nothing");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
    }
}
