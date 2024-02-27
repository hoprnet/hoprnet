use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NodeInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NodeInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NodeInfo::ChainAddress)
                            .string_len(40)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(NodeInfo::PacketKey).binary_len(32))
                    .col(ColumnDef::new(NodeInfo::SafeBalance).binary_len(12).not_null())
                    .col(ColumnDef::new(NodeInfo::SafeAllowance).binary_len(12).not_null())
                    .col(ColumnDef::new(NodeInfo::SafeAddress).string_len(40).not_null())
                    .col(ColumnDef::new(NodeInfo::ModuleAddress).string_len(40).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NodeInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NodeInfo {
    Table,
    Id,
    ChainAddress,
    PacketKey,
    SafeBalance,
    SafeAllowance,
    SafeAddress,
    ModuleAddress,
}
