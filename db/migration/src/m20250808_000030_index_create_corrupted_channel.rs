use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .drop_column(Channel::Corrupted)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CorruptedChannel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CorruptedChannel::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CorruptedChannel::ChannelId)
                            .string_len(64)
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CorruptedChannel::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CorruptedChannel {
    Table,
    Id,
    ChannelId,
}

#[derive(DeriveIden)]
enum Channel {
    Table,
    Corrupted,
}
