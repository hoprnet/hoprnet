use sea_query::{Condition, Expr, Query, SelectStatement, SimpleExpr};

use super::query::{SqliteMaster, SqliteSchema};
use super::Sqlite;
use crate::probe::{Has, Schema, SchemaProbe};

impl SchemaProbe for Sqlite {
    fn get_current_schema() -> SimpleExpr {
        unimplemented!()
    }

    fn query_tables() -> SelectStatement {
        Query::select()
            .expr_as(Expr::col(SqliteSchema::Name), Schema::TableName)
            .from(SqliteMaster)
            .cond_where(
                Condition::all()
                    .add(Expr::col(SqliteSchema::Type).eq("table"))
                    .add(Expr::col(SqliteSchema::Name).ne("sqlite_sequence")),
            )
            .take()
    }

    fn has_column<T, C>(table: T, column: C) -> SelectStatement
    where
        T: AsRef<str>,
        C: AsRef<str>,
    {
        Query::select()
            .expr(Expr::cust_with_values(
                "COUNT(*) > 0 AS \"has_column\" FROM pragma_table_info(?)",
                [table.as_ref()],
            ))
            .and_where(Expr::col(SqliteSchema::Name).eq(column.as_ref()))
            .take()
    }

    fn has_index<T, C>(table: T, index: C) -> SelectStatement
    where
        T: AsRef<str>,
        C: AsRef<str>,
    {
        Query::select()
            .expr_as(Expr::cust("COUNT(*) > 0"), Has::Index)
            .from(SqliteMaster)
            .cond_where(
                Condition::all()
                    .add(Expr::col(SqliteSchema::Type).eq("index"))
                    .add(Expr::col(SqliteSchema::TblName).eq(table.as_ref()))
                    .add(Expr::col(SqliteSchema::Name).eq(index.as_ref())),
            )
            .take()
    }
}
