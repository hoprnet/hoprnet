use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove log table entries to reset the log tracking for the indexer.
        let db = manager.get_connection();
        db.execute_unprepared("DELETE FROM `log`").await?;
        db.execute_unprepared("DELETE FROM `log_status`").await?;
        db.execute_unprepared("DELETE FROM `log_topic_info`").await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // no-op
        Ok(())
    }
}
