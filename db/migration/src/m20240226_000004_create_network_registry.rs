use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NetworkRegistryEntry::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NetworkRegistryEntry::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NetworkRegistryEntry::ChainAddress).string_len(40).unique_key().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NetworkRegistryEntry::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NetworkRegistryEntry {
    Table,
    Id,
    ChainAddress,
}
