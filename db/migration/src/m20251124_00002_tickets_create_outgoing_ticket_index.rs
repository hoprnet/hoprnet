use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OutgoingTicketIndex::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OutgoingTicketIndex::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OutgoingTicketIndex::ChannelId)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(OutgoingTicketIndex::Index)
                            .not_null()
                            .binary_len(8)
                            .default(vec![0u8; 8]),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OutgoingTicketIndex::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OutgoingTicketIndex {
    Table,
    Id,
    ChannelId,
    Index,
}
