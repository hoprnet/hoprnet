use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove peers table entries to reset the state of the network peers handling and force it
        // to start fresh.
        let db = manager.get_connection();
        db.execute_unprepared("DELETE FROM `network_peer`").await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // no-op
        Ok(())
    }
}
