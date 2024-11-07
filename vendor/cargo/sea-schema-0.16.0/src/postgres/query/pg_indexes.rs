use super::SchemaQueryBuilder;
use crate::sqlx_types::postgres::PgRow;
use sea_query::{Alias, Condition, Expr, Iden, JoinType, Order, Query, SeaRc, SelectStatement};

#[derive(Debug, Iden)]
pub enum PgIndexes {
    Table,
    #[iden = "tablename"]
    TableName,
    #[iden = "schemaname"]
    SchemaName,
    #[iden = "indexname"]
    IndexName,
}

#[derive(Debug, Iden)]
pub enum PgIndex {
    Table,
    #[iden = "indexrelid"]
    IndexRelId,
    #[iden = "indrelid"]
    IndRelId,
    #[iden = "indisunique"]
    IndIsUnique,
    #[iden = "indisprimary"]
    IndIsPrimary,
}

#[derive(Debug, Iden)]
pub enum PgClass {
    Table,
    Oid,
    #[iden = "relnamespace"]
    RelNamespace,
    #[iden = "relname"]
    RelName,
}

#[derive(Debug, Iden)]
pub enum PgNamespace {
    Table,
    Oid,
    #[iden = "nspname"]
    NspName,
}

#[derive(Debug, Iden)]
pub enum PgAttribute {
    Table,
    Oid,
    #[iden = "attrelid"]
    AttRelId,
    #[iden = "attname"]
    AttName,
}

#[derive(Debug, Default)]
pub struct UniqueIndexQueryResult {
    pub index_name: String,
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
}

impl SchemaQueryBuilder {
    pub fn query_table_unique_indexes(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
    ) -> SelectStatement {
        let idx = Alias::new("idx");
        let insp = Alias::new("insp");
        let tbl = Alias::new("tbl");
        let tnsp = Alias::new("tnsp");
        let col = Alias::new("col");

        Query::select()
            .column((idx.clone(), PgClass::RelName))
            .column((insp.clone(), PgNamespace::NspName))
            .column((tbl.clone(), PgClass::RelName))
            .column((col.clone(), PgAttribute::AttName))
            .from(PgIndex::Table)
            .join_as(
                JoinType::Join,
                PgClass::Table,
                idx.clone(),
                Expr::col((idx.clone(), PgClass::Oid))
                    .equals((PgIndex::Table, PgIndex::IndexRelId)),
            )
            .join_as(
                JoinType::Join,
                PgNamespace::Table,
                insp.clone(),
                Expr::col((insp.clone(), PgNamespace::Oid))
                    .equals((idx.clone(), PgClass::RelNamespace)),
            )
            .join_as(
                JoinType::Join,
                PgClass::Table,
                tbl.clone(),
                Expr::col((tbl.clone(), PgClass::Oid)).equals((PgIndex::Table, PgIndex::IndRelId)),
            )
            .join_as(
                JoinType::Join,
                PgNamespace::Table,
                tnsp.clone(),
                Expr::col((tnsp.clone(), PgNamespace::Oid))
                    .equals((tbl.clone(), PgClass::RelNamespace)),
            )
            .join_as(
                JoinType::Join,
                PgAttribute::Table,
                col.clone(),
                Expr::col((col.clone(), PgAttribute::AttRelId))
                    .equals((idx.clone(), PgAttribute::Oid)),
            )
            .cond_where(
                Condition::all()
                    .add(Expr::col((PgIndex::Table, PgIndex::IndIsUnique)).eq(true))
                    .add(Expr::col((PgIndex::Table, PgIndex::IndIsPrimary)).eq(false))
                    .add(Expr::col((tbl.clone(), PgClass::RelName)).eq(table.to_string()))
                    .add(Expr::col((tnsp.clone(), PgNamespace::NspName)).eq(schema.to_string())),
            )
            .order_by((PgIndex::Table, PgIndex::IndexRelId), Order::Asc)
            .take()
    }
}

#[cfg(feature = "sqlx-postgres")]
impl From<&PgRow> for UniqueIndexQueryResult {
    fn from(row: &PgRow) -> Self {
        use crate::sqlx_types::Row;
        Self {
            index_name: row.get(0),
            table_schema: row.get(1),
            table_name: row.get(2),
            column_name: row.get(3),
        }
    }
}

#[cfg(not(feature = "sqlx-postgres"))]
impl From<&PgRow> for UniqueIndexQueryResult {
    fn from(_: &PgRow) -> Self {
        Self::default()
    }
}
