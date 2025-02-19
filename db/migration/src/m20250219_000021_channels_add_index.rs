use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx_channel_closure_time")
                    .table(Channel::Table)
                    .col((Channel::ClosureTime, IndexOrder::Asc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name("idx_channel_status")
                    .table(Channel::Table)
                    .col((Channel::Status, IndexOrder::Asc))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_channel_closure_time").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_channel_status").to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Channel {
    Table,
    #[allow(dead_code)]
    Id,
    #[allow(dead_code)]
    ChannelId,
    #[allow(dead_code)]
    Source,
    #[allow(dead_code)]
    Destination,
    Status,
    #[allow(dead_code)]
    Balance,
    #[allow(dead_code)]
    TicketIndex,
    #[allow(dead_code)]
    Epoch,
    ClosureTime,
}
