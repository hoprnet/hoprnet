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
                    .col(ColumnDef::new(Ticket::Amount).string_len(50).not_null())
                    .col(ColumnDef::new(Ticket::Index).integer().unsigned().not_null())
                    .col(ColumnDef::new(Ticket::IndexOffset).integer().unsigned().not_null())
                    .col(ColumnDef::new(Ticket::WinningProbability).binary_len(7).not_null())
                    .col(
                        ColumnDef::new(Ticket::ChannelEpoch)
                            .integer()
                            .unsigned()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(Ticket::EthereumChallenge).binary_len(64).not_null())
                    .col(ColumnDef::new(Ticket::Signature).binary_len(60).not_null())
                    .col(ColumnDef::new(Ticket::AcknowledgementData).binary().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_ticket_channel")
                            .from(Ticket::Table, Ticket::ChannelId)
                            .to(Channel::Table, Channel::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
    EthereumChallenge,
    Signature,
    AcknowledgementData,
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    Id,
}
