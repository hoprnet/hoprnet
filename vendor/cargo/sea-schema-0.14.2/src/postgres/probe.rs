use sea_query::{Condition, Expr, Query, SelectStatement, SimpleExpr};

use super::query::{InformationSchema as Schema, PgIndexes, TablesFields};
use super::Postgres;
use crate::probe::{Has, SchemaProbe};

impl SchemaProbe for Postgres {
    fn get_current_schema() -> SimpleExpr {
        Expr::cust("CURRENT_SCHEMA()")
    }

    fn query_tables() -> SelectStatement {
        Query::select()
            .expr_as(Expr::col(TablesFields::TableName), TablesFields::TableName)
            .from((Schema::Schema, Schema::Tables))
            .cond_where(
                Condition::all()
                    .add(
                        Expr::expr(Self::get_current_schema())
                            .equals((Schema::Tables, TablesFields::TableSchema)),
                    )
                    .add(Expr::col(TablesFields::TableType).eq("BASE TABLE")),
            )
            .take()
    }

    fn has_index<T, C>(table: T, index: C) -> SelectStatement
    where
        T: AsRef<str>,
        C: AsRef<str>,
    {
        Query::select()
            .expr_as(Expr::cust("COUNT(*) > 0"), Has::Index)
            .from(PgIndexes::Table)
            .cond_where(
                Condition::all()
                    .add(Expr::col(PgIndexes::SchemaName).eq(Self::get_current_schema()))
                    .add(Expr::col(PgIndexes::TableName).eq(table.as_ref()))
                    .add(Expr::col(PgIndexes::IndexName).eq(index.as_ref())),
            )
            .take()
    }
}
