use crate::{db::HoprdDb, errors::Result};
use async_trait::async_trait;
use hopr_internal_types::prelude::AliasEntry;

pub const ALIASES_ENTRY_NAME: &str = "aliases";

/// Defines DB API for accessing HOPR settings (mainly aliases for now)
#[async_trait]
pub trait HoprdDbMetadataOperations {
    /// Retrieve the alias for a given PeerID
    async fn resolve_alias(&self, alias: String) -> Result<Option<String>>;

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
impl HoprdDbMetadataOperations for HoprdDb {
    async fn resolve_alias(&self, alias: String) -> Result<Option<String>> {
        let aliases = self.get_aliases().await?;
        let alias_entry = aliases.into_iter().find(|entry| entry.alias == alias);

        match alias_entry {
            Some(alias_entry) => return Ok(Some(alias_entry.peer_id)),
            None => return Ok(None),
        }
    }

    async fn get_aliases(&self) -> Result<Vec<AliasEntry>> {
        return Ok(vec![]);
    }

    async fn set_aliases(&self, aliases: Vec<AliasEntry>) -> Result<bool> {
        Ok(true)
    }

    async fn set_alias(&self, peer: String, alias: String) -> Result<bool> {
        Ok(true)
    }

    async fn delete_alias(&self, alias: String) -> Result<bool> {
        Ok(true)
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
            .expect("should add alias");

        let aliases = db.get_aliases().await.expect("should get aliases");

        assert_eq!(aliases.len(), 1);
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
}
