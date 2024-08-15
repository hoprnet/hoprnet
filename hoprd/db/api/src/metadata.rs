use async_trait::async_trait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{db::HoprdDb, errors::Result};

/// Represents an alias
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AliasEntry {
    pub peer_id: String,
    pub alias: String,
}

impl AliasEntry {
    pub fn new(peer_id: String, alias: String) -> Self {
        Self { peer_id, alias }
    }
}

/// Defines DB API for accessing HOPR settings (mainly aliases for now)
#[async_trait]
pub trait HoprdDbMetadataOperations {
    /// Retrieve the alias for a given PeerID
    async fn resolve_alias(&self, alias: String) -> Result<Option<String>>;

    /// Retrive all aliases
    async fn get_aliases(&self) -> Result<Vec<AliasEntry>>;

    /// Create new key pair value in the db
    async fn set_alias(&self, peer: String, alias: String) -> Result<()>;

    /// Update aliases
    async fn set_aliases(&self, aliases: Vec<AliasEntry>) -> Result<()>;

    /// Delete alias
    async fn delete_alias(&self, alias: String) -> Result<()>;

    /// Delete all aliases
    async fn clear_aliases(&self) -> Result<()>;
}

#[async_trait]
impl HoprdDbMetadataOperations for HoprdDb {
    async fn resolve_alias(&self, alias: String) -> Result<Option<String>> {
        let row = hoprd_db_entity::aliases::Entity::find()
            .filter(hoprd_db_entity::aliases::Column::Alias.eq(alias))
            .one(&self.metadata)
            .await?;

        Ok(row.map(|model| model.peer_id))
    }

    async fn get_aliases(&self) -> Result<Vec<AliasEntry>> {
        let rows = hoprd_db_entity::aliases::Entity::find().all(&self.metadata).await?;

        let aliases = rows
            .iter()
            .map(|row| AliasEntry {
                peer_id: row.peer_id.clone(),
                alias: row.alias.clone(),
            })
            .collect();

        Ok(aliases)
    }

    async fn set_aliases(&self, aliases: Vec<AliasEntry>) -> Result<()> {
        let new_aliases = aliases
            .iter()
            .map(|entry| hoprd_db_entity::aliases::ActiveModel {
                peer_id: sea_orm::ActiveValue::Set(entry.peer_id.clone()),
                alias: sea_orm::ActiveValue::Set(entry.alias.clone()),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let _ = hoprd_db_entity::aliases::Entity::insert_many(new_aliases)
            .exec(&self.metadata)
            .await?;

        Ok(())
    }

    async fn set_alias(&self, peer: String, alias: String) -> Result<()> {
        let new_alias = hoprd_db_entity::aliases::ActiveModel {
            peer_id: sea_orm::ActiveValue::Set(peer),
            alias: sea_orm::ActiveValue::Set(alias),
            ..Default::default()
        };

        // insert should fail if the new alias, or the peer is already in the db
        let res = hoprd_db_entity::aliases::Entity::insert(new_alias)
            .exec(&self.metadata)
            .await;

        if let Err(_e) = res {
            Err(crate::errors::DbError::LogicalError("alias can't be added".into()))
        } else {
            Ok(())
        }
    }

    async fn delete_alias(&self, alias: String) -> Result<()> {
        let res: sea_orm::DeleteResult = hoprd_db_entity::aliases::Entity::delete_many()
            .filter(hoprd_db_entity::aliases::Column::Alias.eq(alias))
            .exec(&self.metadata)
            .await?;

        if res.rows_affected > 0 {
            Ok(())
        } else {
            Err(crate::errors::DbError::LogicalError(
                "peer cannot be removed because it does not exist".into(),
            ))
        }
    }

    async fn clear_aliases(&self) -> Result<()> {
        let res: sea_orm::DeleteResult = hoprd_db_entity::aliases::Entity::delete_many()
            .filter(hoprd_db_entity::aliases::Column::Alias.ne("me".to_string()))
            .exec(&self.metadata)
            .await?;

        if res.rows_affected > 0 {
            Ok(())
        } else {
            Err(crate::errors::DbError::LogicalError("No aliases removed".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p_identity::PeerId;

    #[async_std::test]
    async fn test_set_alias() {
        let db = HoprdDb::new_in_memory().await;

        db.set_alias(PeerId::random().to_string(), "test_alias".to_string())
            .await
            .expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
    }

    #[async_std::test]
    async fn test_set_aliases() {
        let db = HoprdDb::new_in_memory().await;
        let entries = vec![
            AliasEntry {
                peer_id: PeerId::random().to_string(),
                alias: "test_alias".to_string(),
            },
            AliasEntry {
                peer_id: PeerId::random().to_string(),
                alias: "test_alias_2".to_string(),
            },
        ];

        db.set_aliases(entries.clone()).await.expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 2);
    }

    #[async_std::test]
    async fn test_set_aliases_twice() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect_err("should fail adding existing alias");
    }

    #[async_std::test]
    async fn test_get_alias() {
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
    async fn test_get_not_set_alias() {
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
    async fn test_delete_alias() {
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
    async fn test_clear() {
        let db = HoprdDb::new_in_memory().await;

        let me_peer_id = PeerId::random().to_string();
        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");
        db.set_alias(me_peer_id.clone(), "me".to_string())
            .await
            .expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");
        assert_eq!(aliases.len(), 2);

        db.clear_aliases().await.expect("should clear aliases except 'me'");
        let aliases = db.get_aliases().await.expect("should get aliases");
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].peer_id, me_peer_id);
    }
}
