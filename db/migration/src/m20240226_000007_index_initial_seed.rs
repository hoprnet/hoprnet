use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Seed initial ChainInfo entry with default values
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(ChainInfo::Table)
                    .columns([ChainInfo::Id])
                    .values_panic([1.into()])
                    .to_owned(),
            )
            .await?;

        // Seed initial NodeInfo entry with default values
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(NodeInfo::Table)
                    .columns([NodeInfo::Id])
                    .values_panic([1.into()])
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(NodeInfo::Table)
                    .and_where(Expr::col(NodeInfo::Id).eq(1))
                    .to_owned(),
            )
            .await?;

        manager
            .exec_stmt(
                Query::delete()
                    .from_table(ChainInfo::Table)
                    .and_where(Expr::col(ChainInfo::Id).eq(1))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChainInfo {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum NodeInfo {
    Table,
    Id,
}
