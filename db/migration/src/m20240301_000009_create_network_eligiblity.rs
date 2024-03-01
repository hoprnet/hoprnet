use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NetworkEligibility::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NetworkEligibility::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NetworkEligibility::SafeAddress).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NetworkEligibility::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NetworkEligibility {
    Table,
    Id,
    SafeAddress,
}
