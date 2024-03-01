use sea_orm_migration::prelude::*;
use crate::sea_orm::prelude::ChronoDateTimeUtc;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TicketStatistics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TicketStatistics::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TicketStatistics::LastUpdated).timestamp().not_null().default(ChronoDateTimeUtc::UNIX_EPOCH),)
                    .col(ColumnDef::new(TicketStatistics::LosingTickets).not_null().integer().default(0))
                    .col(ColumnDef::new(TicketStatistics::RedeemedTickets).not_null().integer().default(0))
                    .col(
                        ColumnDef::new(TicketStatistics::RedeemedValue)
                            .binary_len(12)
                            .not_null()
                            .default(vec![0u8; 12]),
                    )
                    .col(ColumnDef::new(TicketStatistics::NeglectedTickets).not_null().integer().default(0))
                    .col(
                        ColumnDef::new(TicketStatistics::NeglectedValue)
                            .binary_len(12)
                            .not_null()
                            .default(vec![0u8; 12]),
                    )
                    .col(ColumnDef::new(TicketStatistics::RejectedTickets).not_null().integer().default(0))
                    .col(
                        ColumnDef::new(TicketStatistics::RejectedValue)
                            .binary_len(12)
                            .not_null()
                            .default(vec![0u8; 12]),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TicketStatistics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TicketStatistics {
    Table,
    Id,
    LastUpdated,
    LosingTickets,
    RedeemedTickets,
    RedeemedValue,
    NeglectedTickets,
    NeglectedValue,
    RejectedTickets,
    RejectedValue,
}
