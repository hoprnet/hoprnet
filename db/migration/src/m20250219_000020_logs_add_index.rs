use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const IDX_NAME: &str = "idx_log_status_block_number_processed";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name(IDX_NAME)
                    .table(LogStatus::Table)
                    .col((LogStatus::BlockNumber, IndexOrder::Asc))
                    .col(LogStatus::Processed)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_index(Index::drop().name(IDX_NAME).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum LogStatus {
    Table,
    BlockNumber,
    Processed,
}
