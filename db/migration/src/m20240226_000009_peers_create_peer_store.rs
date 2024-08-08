use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::prelude::ChronoDateTimeUtc;

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
                    .col(ColumnDef::new(NetworkPeer::Version).string_len(50))
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
                    .col(ColumnDef::new(NetworkPeer::Ignored).timestamp().null())
                    .col(ColumnDef::new(NetworkPeer::Public).boolean().not_null().default(true))
                    .col(ColumnDef::new(NetworkPeer::Quality).float().not_null().default(0.0))
                    .col(ColumnDef::new(NetworkPeer::QualitySma).binary().null())
                    .col(ColumnDef::new(NetworkPeer::Backoff).float().null())
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NetworkPeer::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NetworkPeer {
    Table,
    Id,
    PacketKey,
    MultiAddresses,
    Origin,
    Version,
    LastSeen,
    LastSeenLatency,
    Ignored,
    Public,
    Quality,
    QualitySma,
    Backoff,
    HeartbeatsSent,
    HeartbeatsSuccessful,
}
