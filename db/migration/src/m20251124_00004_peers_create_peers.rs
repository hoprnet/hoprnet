use sea_orm::prelude::ChronoDateTimeUtc;
use sea_orm_migration::prelude::*;

const IDX_NAME_IGNORED: &str = "idx_ignored";
const IDX_NAME_QUALITY: &str = "idx_quality";
const IDX_NAME_LAST_SEEN: &str = "idx_last_seen";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NetworkPeer::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NetworkPeer::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NetworkPeer::PacketKey)
                            .binary_len(32)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(NetworkPeer::MultiAddresses).json().not_null())
                    .col(ColumnDef::new(NetworkPeer::Origin).tiny_integer().not_null())
                    .col(
                        ColumnDef::new(NetworkPeer::LastSeen)
                            .timestamp()
                            .not_null()
                            .default(ChronoDateTimeUtc::UNIX_EPOCH),
                    )
                    .col(
                        ColumnDef::new(NetworkPeer::LastSeenLatency)
                            .integer()
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(NetworkPeer::IgnoredUntil).timestamp().null())
                    .col(ColumnDef::new(NetworkPeer::Quality).double().not_null().default(0.0))
                    .col(ColumnDef::new(NetworkPeer::QualitySma).binary().null())
                    .col(ColumnDef::new(NetworkPeer::Backoff).double().null())
                    .col(
                        ColumnDef::new(NetworkPeer::HeartbeatsSent)
                            .integer()
                            .unsigned()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(NetworkPeer::HeartbeatsSuccessful)
                            .integer()
                            .unsigned()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

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
            .drop_index(Index::drop().name(IDX_NAME_IGNORED).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(NetworkPeer::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub(crate) enum NetworkPeer {
    Table,
    Id,
    PacketKey,
    MultiAddresses,
    Origin,
    LastSeen,
    LastSeenLatency,
    Ignored,
    IgnoredUntil,
    Quality,
    QualitySma,
    Backoff,
    HeartbeatsSent,
    HeartbeatsSuccessful,
}
