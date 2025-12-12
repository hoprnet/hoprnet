use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // remove the db and all indices
        super::m20251124_00004_peers_create_peers::Migration.down(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // add back the removed db and indices
        super::m20251124_00004_peers_create_peers::Migration.up(manager).await
    }
}
