use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove index table entries to reset the state of the indexer.
        let db = manager.get_connection();
        db.execute_unprepared("DELETE FROM `account`").await?;
        db.execute_unprepared("DELETE FROM `announcement`").await?;
        db.execute_unprepared("DELETE FROM `channel`").await?;
        db.execute_unprepared("DELETE FROM `chain_info`").await?;
        db.execute_unprepared("DELETE FROM `network_eligibility`").await?;
        db.execute_unprepared("DELETE FROM `network_registry`").await?;
        db.execute_unprepared("DELETE FROM `node_info`").await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // no-op
        Ok(())
    }
}
