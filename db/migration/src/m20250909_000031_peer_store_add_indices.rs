use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const IDX_NAME_IGNORED: &str = "idx_ignored";
const IDX_NAME_QUALITY: &str = "idx_quality";
const IDX_NAME_LAST_SEEN: &str = "idx_last_seen";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The PacketKey column already has a UNIQUE constraint, which automatically creates an index.

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_NAME_IGNORED)
                    .table(NetworkPeer::Table)
                    .col((NetworkPeer::Ignored, IndexOrder::Asc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_NAME_LAST_SEEN)
                    .table(NetworkPeer::Table)
                    .col((NetworkPeer::LastSeen, IndexOrder::Asc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name(IDX_NAME_QUALITY)
                    .table(NetworkPeer::Table)
                    .col((NetworkPeer::Quality, IndexOrder::Asc))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(IDX_NAME_QUALITY).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(IDX_NAME_LAST_SEEN).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(IDX_NAME_IGNORED_UNTIL).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum NetworkPeer {
    Table,
    Ignored,
    LastSeen,
    Quality,
}
