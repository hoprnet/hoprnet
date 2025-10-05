use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Ticket::Table)
                    .add_column(ColumnDef::new(Ticket::Counterparty)
                        .string_len(40)
                        .not_null())

            )
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

    }
}

#[derive(DeriveIden)]
enum Ticket {
    Table,
    Id,
    Counterparty,
}
