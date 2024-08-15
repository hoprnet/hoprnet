use async_trait::async_trait;
use hopr_primitive_types::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{db::HoprdDb, errors::Result};

/// Defines DB API for accessing HOPR settings (mainly aliases for now)
#[async_trait]
pub trait HoprdDbMetadataOperations {
    /// Retrieve the alias for a given PeerID
    async fn resolve_alias(&self, alias: String) -> Result<Option<String>>;

    /// Retrive all aliases
    async fn get_aliases(&self) -> Result<Vec<Alias>>;

    /// Create new key pair value in the db
    async fn set_alias(&self, peer: String, alias: String) -> Result<()>;

    /// Update aliases
    async fn set_aliases(&self, aliases: Vec<Alias>) -> Result<()>;

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

    async fn get_aliases(&self) -> Result<Vec<Alias>> {
        let rows = hoprd_db_entity::aliases::Entity::find().all(&self.metadata).await?;

        let aliases = rows
            .iter()
            .map(|row| Alias {
                peer_id: row.peer_id.clone(),
                alias: row.alias.clone(),
            })
            .collect();

        Ok(aliases)
    }

    async fn set_aliases(&self, aliases: Vec<Alias>) -> Result<()> {
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
        // update the entry if peer already in the db
        let row = hoprd_db_entity::aliases::Entity::find()
            .filter(hoprd_db_entity::aliases::Column::PeerId.eq(peer.clone()))
            .one(&self.metadata)
            .await?;

        if let Some(mut row) = row {
            row.alias = alias;
            let _ = hoprd_db_entity::aliases::Entity::update_many()
                .filter(hoprd_db_entity::aliases::Column::PeerId.eq(peer.clone()))
                .exec(&self.metadata)
                .await?;
        } else {
            let new_alias = hoprd_db_entity::aliases::ActiveModel {
                peer_id: sea_orm::ActiveValue::Set(peer.clone()),
                alias: sea_orm::ActiveValue::Set(alias.clone()),
                ..Default::default()
            };
            let _ = hoprd_db_entity::aliases::Entity::insert(new_alias)
                .exec(&self.metadata)
                .await?;
        }

        Ok(())
    }

    async fn delete_alias(&self, alias: String) -> Result<()> {
        let res = hoprd_db_entity::aliases::Entity::delete_many()
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
        let res = hoprd_db_entity::aliases::Entity::delete_many()
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
    async fn set_alias_should_set_multiple_aliases_in_a_transaction() -> Result<()> {
        let db = HoprdDb::new_in_memory().await;
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
    async fn test_set_aliases_twice() {
        let db = HoprdDb::new_in_memory().await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should replace alias");

        let aliases = db.get_aliases().await.unwrap();

        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].alias, alias);
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
