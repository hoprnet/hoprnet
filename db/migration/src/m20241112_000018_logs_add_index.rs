use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // This index enables fast querying for the next unprocessed log.
        // In a 300MB dataset it reduces the query time from 100ms to 1e-5s.
        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx_unprocessed_log_status")
                    .table(LogStatus::Table)
                    .col((LogStatus::BlockNumber, IndexOrder::Asc))
                    .col((LogStatus::TransactionIndex, IndexOrder::Asc))
                    .col((LogStatus::LogIndex, IndexOrder::Asc))
                    .and_where(Expr::col((LogStatus::Table, LogStatus::Checksum)).is_null())
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_unprocessed_log_status").to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LogStatus {
    Table,
    // Values to identify the log.
    BlockNumber,
    TransactionIndex,
    LogIndex,
    // Indicates whether the log has been processed.
    #[warn(dead_code)]
    Processed,
    // Time when the log was processed.
    #[warn(dead_code)]
    ProcessedAt,
    // Computed checksum of this log and previous logs
    Checksum,
}
