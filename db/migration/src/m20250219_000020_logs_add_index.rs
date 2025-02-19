use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx_log_status_block_number_processed")
                    .table(LogStatus::Table)
                    .col((LogStatus::BlockNumber, IndexOrder::Asc))
                    .col(LogStatus::Processed)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_log_status_block_number_processed").to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LogStatus {
    Table,
    // Values to identify the log.
    BlockNumber,
    #[allow(dead_code)]
    TransactionIndex,
    #[allow(dead_code)]
    LogIndex,
    // Indicates whether the log has been processed.
    #[allow(dead_code)]
    Processed,
    // Time when the log was processed.
    #[allow(dead_code)]
    ProcessedAt,
    // Computed checksum of this log and previous logs
    #[allow(dead_code)]
    Checksum,
}
