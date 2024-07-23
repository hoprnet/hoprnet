use crate::{db::HoprDb, errors::Result};
use async_trait::async_trait;
use hopr_internal_types::prelude::AliasEntry;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

pub const ALIASES_ENTRY_NAME: &str = "aliases";

/// Defines DB API for accessing HOPR settings (mainly aliases for now)
#[async_trait]
pub trait HoprDbSettingsOperations {
    /// Retrieve the alias for a given PeerID
    async fn get_alias(&self, alias: String) -> Result<Option<String>>;

    /// Retrive all aliases
    async fn get_aliases(&self) -> Result<Vec<AliasEntry>>;

    /// Create new key pair value in the db
    async fn set_alias(&self, peer: String, alias: String) -> Result<bool>;

    /// Update aliases
    async fn set_aliases(&self, aliases: Vec<AliasEntry>) -> Result<bool>;

    /// Delete alias
    async fn delete_alias(&self, alias: String) -> Result<bool>;
}

#[async_trait]
impl HoprDbSettingsOperations for HoprDb {
    async fn get_alias(&self, alias: String) -> Result<Option<String>> {
        let aliases = self.get_aliases().await?;
        let alias_entry = aliases.into_iter().find(|entry| entry.alias.to_string() == alias);

        match alias_entry {
            Some(alias_entry) => return Ok(Some(alias_entry.peer_id)),
            None => return Ok(None),
        }
    }

    async fn get_aliases(&self) -> Result<Vec<AliasEntry>> {
        let model = hopr_db_entity::global_settings::Entity::find()
            .filter(hopr_db_entity::global_settings::Column::Key.eq("aliases"))
            .one(&self.settings_db)
            .await?;

        if model.is_none() {
            let model = hopr_db_entity::global_settings::ActiveModel {
                id: Default::default(),
                key: sea_orm::ActiveValue::Set(ALIASES_ENTRY_NAME.to_string()),
                value: sea_orm::ActiveValue::Set("{}".into()),
            };

            model.insert(&self.settings_db).await?;
            return Ok(vec![]);
        }

        let model = model.unwrap();
        let aliases_row = std::str::from_utf8(&model.value).unwrap();
        let aliases: Vec<AliasEntry> = serde_json::from_str(aliases_row).unwrap_or_default();

        return Ok(aliases);
    }

    async fn set_aliases(&self, aliases: Vec<AliasEntry>) -> Result<bool> {
        let aliases_str = serde_json::to_string(&aliases).unwrap();

        let model = hopr_db_entity::global_settings::Entity::find()
            .filter(hopr_db_entity::global_settings::Column::Key.eq("aliases"))
            .one(&self.settings_db)
            .await?;

        if let Some(model) = model {
            let mut data: hopr_db_entity::global_settings::ActiveModel = model.into();
            data.value = sea_orm::ActiveValue::Set(aliases_str.into());
            data.save(&self.settings_db).await?;
        } else {
            let model = hopr_db_entity::global_settings::ActiveModel {
                key: sea_orm::ActiveValue::Set(ALIASES_ENTRY_NAME.to_string()),
                value: sea_orm::ActiveValue::Set(aliases_str.into()),
                ..Default::default()
            };
            model.insert(&self.settings_db).await?;
        }

        Ok(true)
    }

    async fn set_alias(&self, peer: String, alias: String) -> Result<bool> {
        let mut aliases = self.get_aliases().await?;
        let alias_entry = AliasEntry {
            peer_id: peer.clone(),
            alias,
        };

        if let Some(_) = aliases.iter_mut().find(|alias| alias.peer_id == peer) {
            return Ok(false);
        }

        aliases.push(alias_entry);

        self.set_aliases(aliases).await?;

        Ok(true)
    }

    async fn delete_alias(&self, alias: String) -> Result<bool> {
        let aliases = self.get_aliases().await?;
        let aliases = aliases.into_iter().filter(|a| a.alias != alias).collect::<Vec<_>>();

        println!("aliases: {:?}", aliases);

        self.set_aliases(aliases).await?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hopr_crypto_types::keypairs::{ChainKeypair, Keypair};
    use libp2p_identity::PeerId;

    #[async_std::test]
    async fn test_set_alias() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        db.set_alias(PeerId::random().to_string(), "test_alias".to_string())
            .await
            .expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
    }

    #[async_std::test]
    async fn test_set_aliases() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;
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
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
    }

    #[async_std::test]
    async fn test_get_alias() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        let alias = db.get_alias(alias).await.expect("should get alias");

        assert!(alias.is_some());
    }

    #[async_std::test]
    async fn test_get_not_set_alias() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

        let peer_id = PeerId::random().to_string();
        let alias = "test_alias".to_string();

        db.set_alias(peer_id.clone(), alias.clone())
            .await
            .expect("should add alias");

        let alias = db
            .get_alias(PeerId::random().to_string())
            .await
            .expect("should get alias");

        assert!(alias.is_none());
    }

    #[async_std::test]
    async fn test_delete_alias() {
        let db = HoprDb::new_in_memory(ChainKeypair::random()).await;

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
}
