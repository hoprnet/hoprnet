use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Log and LogStatus tables are kept separate to allow for easier export of the logs
        // themselves.

        manager
            .create_table(
                Table::create()
                    .table(LogStatus::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .name("pk_log_status")
                            .table(LogStatus::Table)
                            .col(LogStatus::BlockNumber)
                            .col(LogStatus::TransactionIndex)
                            .col(LogStatus::LogIndex),
                    )
                    .col(ColumnDef::new(LogStatus::TransactionIndex).not_null().binary_len(8))
                    .col(ColumnDef::new(LogStatus::LogIndex).not_null().binary_len(8))
                    .col(ColumnDef::new(LogStatus::BlockNumber).not_null().binary_len(8))
                    .col(ColumnDef::new(LogStatus::Processed).boolean().not_null().default(false))
                    .col(ColumnDef::new(LogStatus::ProcessedAt).date_time())
                    .col(ColumnDef::new(LogStatus::Checksum).binary_len(32))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Log::Table)
                    .if_not_exists()
                    .primary_key(
                    .primary_key(
                        Index::create()
                            .name("pk_log")
                            .table(Log::Table)
                            .col(Log::BlockNumber)
                            .col(Log::TransactionIndex)
                            .col(Log::LogIndex),
                    )
                    .col(ColumnDef::new(Log::TransactionIndex).not_null().binary_len(8))
                    .col(ColumnDef::new(Log::LogIndex).not_null().binary_len(8))
                    .col(ColumnDef::new(Log::BlockNumber).not_null().binary_len(8))
                    .col(ColumnDef::new(Log::BlockHash).binary_len(32).not_null())
                    .col(ColumnDef::new(Log::TransactionHash).binary_len(32).not_null())
                    .col(ColumnDef::new(Log::Address).binary_len(20).not_null())
                    .col(ColumnDef::new(Log::Topics).binary().not_null())
                    .col(ColumnDef::new(Log::Data).binary().not_null())
                    .col(ColumnDef::new(Log::Removed).boolean().not_null().default(false))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_log_status_log")
                            .from(Log::Table, (Log::BlockNumber, Log::TransactionIndex, Log::LogIndex))
                            .to(
                                LogStatus::Table,
                                (LogStatus::BlockNumber, LogStatus::TransactionIndex, LogStatus::LogIndex),
                            )
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Log::Table).to_owned()).await?;
        manager
            .drop_table(Table::drop().table(LogStatus::Table).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Log {
    Table,
    // address from which this log originated.
    Address,
    // Array of 0 to 4 32 Bytes DATA of indexed log arguments. The first topic is the
    // hash of the signature of the event.
    Topics,
    // contains zero or more 32 Bytes non-indexed arguments of the log.
    Data,
    // the block number where this log was in. null when it's a pending log.
    BlockNumber,
    // hash of the transactions this log was created from. null when its pending log.
    // hash of the transaction this log was created from. null when it's a pending log.
    TransactionHash,
    // integer of the transaction's index position this log was created from. null when it's a pending log.
    TransactionIndex,
    // hash of the block where this log was in. null when its pending. null when its pending log.
    BlockHash,
    // integer of the log index position in the block. null when its pending log.
    LogIndex,
    // true when the log was removed, due to a chain reorganization. false if its a valid log.
    Removed,
}

#[derive(DeriveIden)]
enum LogStatus {
    Table,
    // Values to identify the log.
    BlockNumber,
    TransactionIndex,
    LogIndex,
    // Indicates whether the log has been processed.
    Processed,
    // Time when the log was processed.
    ProcessedAt,
    // Computed checksum of this log and previous logs
    Checksum,
}
