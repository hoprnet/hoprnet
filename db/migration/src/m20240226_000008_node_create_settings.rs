use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GlobalSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GlobalSettings::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(GlobalSettings::Key)
                            .string_len(80)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(GlobalSettings::Value).binary().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GlobalSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GlobalSettings {
    Table,
    Id,
    Key,
    Value,
}
