use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LogTopicInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LogTopicInfo::Id)
                            .primary_key()
                            .not_null()
                            .integer()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(LogTopicInfo::Address).string_len(40).not_null())
                    .col(ColumnDef::new(LogTopicInfo::Topic).string_len(64).not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LogTopicInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum LogTopicInfo {
    Table,
    Id,
    /// Contract address for filter
    Address,
    /// Topic for the contract on this address
    Topic,
}
