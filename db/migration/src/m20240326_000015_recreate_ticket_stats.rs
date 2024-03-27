use crate::BackendType;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration(pub BackendType);

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // TODO: we have to drop here, if the previous non-release migration is retained.
        // Remove, if the previous ticket_stats migration is compacted before the release
        // and compact this change as well.
        manager
            .get_connection()
            .execute_unprepared("DROP TRIGGER trig_ticket_stats_update_timestamp;")
            .await?;

        manager
            .drop_table(Table::drop().table(TicketStatistics::Table).to_owned())
            .await?;

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
                    .col(
                        ColumnDef::new(TicketStatistics::ChannelId)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(TicketStatistics::LastUpdated).timestamp().null())
                    .col(
                        ColumnDef::new(TicketStatistics::WinningTickets)
                            .not_null()
                            .integer()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(TicketStatistics::RedeemedValue)
                            .binary_len(12)
                            .not_null()
                            .default(vec![0u8; 12]),
                    )
                    .col(
                        ColumnDef::new(TicketStatistics::NeglectedValue)
                            .binary_len(12)
                            .not_null()
                            .default(vec![0u8; 12]),
                    )
                    .col(
                        ColumnDef::new(TicketStatistics::RejectedValue)
                            .binary_len(12)
                            .not_null()
                            .default(vec![0u8; 12]),
                    )
                    .to_owned(),
            )
            .await?;

        let conn = manager.get_connection();

        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS trig_ticket_stats_update_timestamp
                AFTER UPDATE
                ON ticket_statistics
                FOR EACH ROW
                WHEN OLD.last_updated IS NULL OR NEW.last_updated < OLD.last_updated --- avoid infinite loop
            BEGIN
                UPDATE ticket_statistics SET last_updated=CURRENT_TIMESTAMP WHERE id=OLD.id;
            END;
        "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared("DROP TRIGGER trig_ticket_stats_update_timestamp;")
            .await?;

        manager
            .drop_table(Table::drop().table(TicketStatistics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TicketStatistics {
    Table,
    Id,
    ChannelId,
    LastUpdated,
    WinningTickets,
    RedeemedValue,
    NeglectedValue,
    RejectedValue,
}
