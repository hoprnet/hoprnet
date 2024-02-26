use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Channel::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Channel::ChannelId)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Channel::Source).string_len(40).not_null())
                    .col(ColumnDef::new(Channel::Destination).string_len(40).not_null())
                    .col(ColumnDef::new(Channel::Balance).string_len(50).not_null())
                    .col(ColumnDef::new(Channel::Status).tiny_integer().not_null())
                    .col(
                        ColumnDef::new(Channel::Epoch)
                            .integer()
                            .unsigned()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(Channel::TicketIndex)
                            .integer()
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_channel_source")
                    .table(Channel::Table)
                    .col(Channel::Source)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_channel_destination")
                    .table(Channel::Table)
                    .col(Channel::Destination)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_channel_source").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_channel_destination").to_owned())
            .await?;

        manager.drop_table(Table::drop().table(Channel::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    Id,
    ChannelId,
    Source,
    Destination,
    Status,
    Balance,
    TicketIndex,
    Epoch,
}
