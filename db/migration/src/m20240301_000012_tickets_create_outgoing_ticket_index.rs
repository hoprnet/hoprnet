use crate::BackendType;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration(pub crate::BackendType);

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut table = Table::create()
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
            .to_owned();

        manager
            .create_table(if self.0 != BackendType::SQLite {
                table
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ticket_channel")
                            .from_tbl(OutgoingTicketIndex::Table)
                            .from_col(OutgoingTicketIndex::ChannelId)
                            .to_tbl(Channel::Table)
                            .to_col(Channel::ChannelId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .to_owned()
            } else {
                table
            })
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

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Channel {
    Table,
    ChannelId,
}
