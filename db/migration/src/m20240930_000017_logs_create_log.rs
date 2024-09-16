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
                    .table(Log::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Log::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Log::TransactionIndex)
                            .not_null()
                            .binary_len(8)
                            .default(vec![0u8; 8]),
                    )
                    .col(
                        ColumnDef::new(Log::LogIndex)
                            .not_null()
                            .binary_len(8)
                            .default(vec![0u8; 8]),
                    )
                    .col(
                        ColumnDef::new(Log::BlockNumber)
                            .not_null()
                            .binary_len(8)
                            .default(vec![0u8; 8]),
                    )
                    .col(ColumnDef::new(Log::BlockHash).string_len(64).not_null())
                    .col(ColumnDef::new(Log::TransactionHash).string_len(64).not_null())
                    .col(ColumnDef::new(Log::Address).string_len(40).not_null())
                    .col(ColumnDef::new(Log::Topics).json().not_null())
                    .col(ColumnDef::new(Log::Data).binary().not_null())
                    .col(ColumnDef::new(Log::Removed).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_log_unique")
                    .if_not_exists()
                    .unique()
                    .table(Log::Table)
                    // We use the number columns to keep the index small.
                    // Full hashes would blow up the size.
                    .col(Log::BlockNumber)
                    .col(Log::LogIndex)
                    .col(Log::TransactionIndex)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(LogStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LogStatus::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LogStatus::TransactionIndex)
                            .not_null()
                            .binary_len(8)
                            .default(vec![0u8; 8]),
                    )
                    .col(
                        ColumnDef::new(LogStatus::LogIndex)
                            .not_null()
                            .binary_len(8)
                            .default(vec![0u8; 8]),
                    )
                    .col(
                        ColumnDef::new(LogStatus::BlockNumber)
                            .not_null()
                            .binary_len(8)
                            .default(vec![0u8; 8]),
                    )
                    .col(ColumnDef::new(LogStatus::Processed).boolean().not_null().default(false))
                    .col(ColumnDef::new(LogStatus::ProcessedAt).timestamp())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_log_status_unique")
                    .if_not_exists()
                    .unique()
                    .table(LogStatus::Table)
                    // We use the number columns to keep the index small.
                    // Full hashes would blow up the size.
                    .col(Log::BlockNumber)
                    .col(Log::LogIndex)
                    .col(Log::TransactionIndex)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_log_unique").to_owned())
            .await?;
        manager.drop_table(Table::drop().table(Log::Table).to_owned()).await?;
        manager
            .drop_index(Index::drop().name("idx_log_status_unique").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(LogStatus::Table).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Log {
    Table,
    // Primary key, auto-incremented.
    Id,
    // address from which this log originated.
    Address,
    // Array of 0 to 4 32 Bytes DATA of indexed log arguments. The first topic is the
    // hash of the signature of the event.
    Topics,
    // contains zero or more 32 Bytes non-indexed arguments of the log.
    Data,
    // the block number where this log was in. null when its pending. null when its pending log.
    BlockNumber,
    // hash of the transactions this log was created from. null when its pending log.
    TransactionHash,
    // integer of the transactions index position log was created from. null when its pending log.
    TransactionIndex,
    // hash of the block where this log was in. null when its pending. null when its pending log.
    BlockHash,
    // integer of the log index position in the block. null when its pending log.
    LogIndex,
    // true when the log was removed, due to a chain reorganization. false if its a valid log.
    Removed,
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum LogStatus {
    Table,
    // Primary key, auto-incremented.
    Id,
    // Values to identify the log.
    BlockNumber,
    TransactionIndex,
    LogIndex,
    // Status of the log.
    Processed,
    // Time when the log was processed.
    ProcessedAt,
}
