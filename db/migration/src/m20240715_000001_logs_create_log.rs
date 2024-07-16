use hopr_primitive_types::prelude::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Log::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Log::TransactionIndex).integer().not_null())
                    .col(ColumnDef::new(Log::LogIndex).integer().not_null())
                    .col(ColumnDef::new(Log::BlockNumber).integer().not_null())
                    .col(ColumnDef::new(Log::BlockHash).string_len(64).not_null())
                    .col(ColumnDef::new(Log::TransactionHash).string_len(64).not_null())
                    .col(ColumnDef::new(Log::Address).string_len(40).not_null())
                    .col(ColumnDef::new(Log::Topics).string_len(40).not_null())
                    .col(ColumnDef::new(Log::Data).string_len(40).not_null())
                    .col(ColumnDef::new(Log::Removed).string_len(40).not_null())
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
                    // We use the number columns to keep the index small. Full hashes would blow up
                    // the size.
                    .col(Log::BlockNumber)
                    .col(Log::LogIndex)
                    .col(Log::TransactionIndex)
                    .to_owned(),
            )
            .await?;
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Log::Table).to_owned()).await
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
