use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChainInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChainInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ChainInfo::LastIndexedBlock)
                            .integer()
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(ChainInfo::TicketPrice).binary_len(12).null())
                    .col(ColumnDef::new(ChainInfo::ChannelsDST).binary_len(32).null())
                    .col(ColumnDef::new(ChainInfo::LedgerDST).binary_len(32).null())
                    .col(ColumnDef::new(ChainInfo::SafeRegistryDST).binary_len(32).null())
                    .col(
                        ColumnDef::new(ChainInfo::NetworkRegistryEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(ChainInfo::ChainChecksum)
                            .binary_len(32)
                            .default(vec![0u8; 32]),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChainInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ChainInfo {
    Table,
    Id,
    LastIndexedBlock,
    TicketPrice,
    ChannelsDST,
    LedgerDST,
    SafeRegistryDST,
    NetworkRegistryEnabled,
    ChainChecksum,
}
