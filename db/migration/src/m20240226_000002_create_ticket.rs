use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Ticket::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Ticket::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Ticket::ChannelId).string_len(64).not_null())
                    .col(ColumnDef::new(Ticket::Amount).binary_len(12).not_null())
                    .col(ColumnDef::new(Ticket::Index).binary_len(8).not_null())
                    .col(ColumnDef::new(Ticket::IndexOffset).unsigned().not_null())
                    .col(ColumnDef::new(Ticket::WinningProbability).binary_len(7).not_null())
                    .col(ColumnDef::new(Ticket::ChannelEpoch).binary_len(8).not_null())
                    .col(ColumnDef::new(Ticket::Signature).binary_len(64).not_null())
                    .col(ColumnDef::new(Ticket::Response).binary_len(32).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ticket_channel")
                            .from_tbl(Ticket::Table)
                            .from_col(Ticket::ChannelId)
                            .from_col(Ticket::ChannelEpoch)
                            .to_tbl(Channel::Table)
                            .to_col(Channel::ChannelId)
                            .to_col(Channel::Epoch)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_fk_ticket_channel")
                    .if_not_exists()
                    .table(Ticket::Table)
                    .col(Ticket::ChannelId)
                    .col(Ticket::ChannelEpoch)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_ticket_channel_id_epoch_index")
                    .if_not_exists()
                    .table(Ticket::Table)
                    .col(Ticket::ChannelId)
                    .col(Ticket::ChannelEpoch)
                    .col(Ticket::Index)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_ticket_channel_id_epoch_index").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_fk_ticket_channel").to_owned())
            .await?;

        manager.drop_table(Table::drop().table(Ticket::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Ticket {
    Table,
    Id,
    ChannelId,
    Amount,
    Index,
    IndexOffset,
    WinningProbability,
    ChannelEpoch,
    Signature,
    Response,
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Channel {
    Table,
    ChannelId,
    Epoch,
}
