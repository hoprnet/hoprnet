use hopr_primitive_types::prelude::{BinarySerializable, U256};
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
                            .integer()
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
                    .col(ColumnDef::new(Channel::Balance).binary_len(12).not_null())
                    .col(ColumnDef::new(Channel::Status).tiny_unsigned().not_null())
                    .col(
                        ColumnDef::new(Channel::Epoch)
                            .binary_len(8)
                            .not_null()
                            .default(U256::one().to_bytes().to_vec()), // Default set in the SC
                    )
                    .col(
                        ColumnDef::new(Channel::TicketIndex)
                            .binary_len(8)
                            .not_null()
                            .default(U256::zero().to_bytes().to_vec()), // Default set in the SC
                    )
                    .col(ColumnDef::new(Channel::ClosureTime).timestamp().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_channel_id_channel_epoch")
                    .if_not_exists()
                    .table(Channel::Table)
                    .col(Channel::ChannelId)
                    .col(Channel::Epoch)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_channel_source_destination")
                    .if_not_exists()
                    .unique()
                    .table(Channel::Table)
                    .col(Channel::Source)
                    .col(Channel::Destination)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_channel_source")
                    .if_not_exists()
                    .table(Channel::Table)
                    .col(Channel::Source)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_channel_destination")
                    .if_not_exists()
                    .table(Channel::Table)
                    .col(Channel::Destination)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_channel_destination").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_channel_source").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_channel_source_destination").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_channel_id_channel_epoch").to_owned())
            .await?;

        manager.drop_table(Table::drop().table(Channel::Table).to_owned()).await
    }
}

#[allow(clippy::enum_variant_names)]
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
    ClosureTime,
}
