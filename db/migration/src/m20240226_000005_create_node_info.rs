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
                    .col(ColumnDef::new(NodeInfo::SafeBalance).binary_len(12).not_null().default(vec![0u8; 12]))
                    .col(ColumnDef::new(NodeInfo::SafeAllowance).binary_len(12).not_null().default(vec![0u8; 12]))
                    .col(ColumnDef::new(NodeInfo::SafeAddress).string_len(40).null())
                    .col(ColumnDef::new(NodeInfo::ModuleAddress).string_len(40).null())
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
    SafeBalance,
    SafeAllowance,
    SafeAddress,
    ModuleAddress,
}
