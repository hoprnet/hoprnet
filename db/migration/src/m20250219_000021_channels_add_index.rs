use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const IDX_NAME_CLOSURE_TIME: &str = "idx_channel_closure_time";
const IDX_NAME_CHANNEL_STATUS: &str = "idx_channel_status";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name(IDX_NAME_CLOSURE_TIME)
                    .table(Channel::Table)
                    .col((Channel::ClosureTime, IndexOrder::Asc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                sea_query::Index::create()
                    .if_not_exists()
                    .name(IDX_NAME_CHANNEL_STATUS)
                    .table(Channel::Table)
                    .col((Channel::Status, IndexOrder::Asc))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(IDX_NAME_CLOSURE_TIME).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(IDX_NAME_CHANNEL_STATUS).to_owned())
            .await
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(DeriveIden)]
enum Channel {
    Table,
    Status,
    ClosureTime,
}
