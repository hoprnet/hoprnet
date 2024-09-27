use super::{InformationSchema, SchemaQueryBuilder};
use crate::sqlx_types::mysql::MySqlRow;
use sea_query::{Expr, Iden, Order, Query, SeaRc, SelectStatement, Value};

#[derive(Debug, sea_query::Iden)]
/// Ref: https://dev.mysql.com/doc/refman/8.0/en/information-schema-statistics-table.html
pub enum StatisticsFields {
    TableCatalog,
    TableSchema,
    TableName,
    NonUnique,
    IndexSchema,
    IndexName,
    SeqInIndex,
    ColumnName,
    Collation,
    Cardinality,
    SubPart,
    Packed,
    Nullable,
    IndexType,
    Comment,
    IndexComment,
    IsVisible,
    Expression,
}

#[derive(Debug, Default)]
pub struct IndexQueryResult {
    pub non_unique: i32,
    pub index_name: String,
    pub column_name: Option<String>,
    pub collation: Option<String>,
    pub sub_part: Option<i32>,
    pub nullable: String,
    pub index_type: String,
    pub index_comment: String,
    pub expression: Option<String>,
}

impl SchemaQueryBuilder {
    pub fn query_indexes(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
    ) -> SelectStatement {
        Query::select()
            .columns(vec![
                StatisticsFields::NonUnique,
                StatisticsFields::IndexName,
                StatisticsFields::ColumnName,
                StatisticsFields::Collation,
                StatisticsFields::SubPart,
                StatisticsFields::Nullable,
                StatisticsFields::IndexType,
                StatisticsFields::IndexComment,
            ])
            .conditions(
                self.system.is_mysql() && self.system.version >= 80013,
                |q| {
                    q.column(StatisticsFields::Expression);
                },
                |q| {
                    q.expr(Expr::val(Value::String(None)));
                },
            )
            .from((InformationSchema::Schema, InformationSchema::Statistics))
            .and_where(Expr::col(StatisticsFields::TableSchema).eq(schema.to_string()))
            .and_where(Expr::col(StatisticsFields::TableName).eq(table.to_string()))
            .order_by(StatisticsFields::IndexName, Order::Asc)
            .order_by(StatisticsFields::SeqInIndex, Order::Asc)
            .take()
    }
}

#[cfg(feature = "sqlx-mysql")]
impl From<&MySqlRow> for IndexQueryResult {
    fn from(row: &MySqlRow) -> Self {
        use crate::sqlx_types::Row;
        Self {
            non_unique: row.get(0),
            index_name: row.get(1),
            column_name: row.get(2),
            collation: row.get(3),
            sub_part: row.get(4),
            nullable: row.get(5),
            index_type: row.get(6),
            index_comment: row.get(7),
            expression: row.get(8),
        }
    }
}

#[cfg(not(feature = "sqlx-mysql"))]
impl From<&MySqlRow> for IndexQueryResult {
    fn from(_: &MySqlRow) -> Self {
        Self::default()
    }
}
