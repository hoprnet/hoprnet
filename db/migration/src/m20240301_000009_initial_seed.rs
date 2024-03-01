use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Seed initial TicketStatistics entry with default values
        manager.exec_stmt(
            Query::insert()
                .into_table(TicketStatistics::Table)
                .columns([TicketStatistics::Id])
                .values_panic([1.into()])
                .to_owned()
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.exec_stmt(
            Query::delete()
                .from_table(TicketStatistics::Table)
                .and_where(Expr::col(TicketStatistics::Id).eq(1))
                .to_owned()
        ).await
    }
}

#[derive(DeriveIden)]
enum TicketStatistics {
    Table,
    Id,
}
