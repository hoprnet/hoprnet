use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
       manager
           .alter_table(
               Table::alter()
                   .table(ChainInfo::Table)
                   .add_column_if_not_exists(
                       ColumnDef::new(ChainInfo::ChainChecksum)
                           .binary_len(32)
                           .default(vec![0u8; 32])
                   )
                   .to_owned()
           )
           .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                 Table::alter()
                     .table(ChainInfo::Table)
                     .drop_column(ChainInfo::ChainChecksum)
                     .to_owned()
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChainInfo {
    Table,
    ChainChecksum
}
