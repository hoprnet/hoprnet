use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add a new column to the chain_info table
        manager
            .alter_table(
                Table::alter()
                    .table(ChainInfo::Table)
                    .add_column(
                        ColumnDef::new(ChainInfo::PreviousIndexedBlockPrioToChecksumUpdate)
                            .integer()
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // manager.drop_table(Table::drop().table(Post::Table).to_owned()).await
        manager
            .alter_table(
                Table::alter()
                    .table(ChainInfo::Table)
                    .drop_column(ChainInfo::PreviousIndexedBlockPrioToChecksumUpdate)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChainInfo {
    Table,
    PreviousIndexedBlockPrioToChecksumUpdate,
}
