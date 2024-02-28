use super::{CharacterSetFields, InformationSchema, SchemaQueryBuilder};
use crate::sqlx_types::mysql::MySqlRow;
use sea_query::{Expr, Iden, Order, Query, SeaRc, SelectStatement};

#[derive(Debug, sea_query::Iden)]
/// Ref: https://dev.mysql.com/doc/refman/8.0/en/information-schema-tables-table.html
pub enum TablesFields {
    TableCatalog,
    TableSchema,
    TableName,
    TableType,
    Engine,
    Version,
    RowFormat,
    TableRows,
    AvgRowLength,
    DataLength,
    MaxDataLength,
    IndexLength,
    DataFree,
    AutoIncrement,
    CreateTime,
    UpdateTime,
    CheckTime,
    TableCollation,
    Checksum,
    CreateOptions,
    TableComment,
}

#[derive(Debug, sea_query::Iden)]
pub enum TableType {
    #[iden = "BASE TABLE"]
    BaseTable,
    #[iden = "VIEW"]
    View,
    #[iden = "SYSTEM VIEW"]
    SystemView,
    #[iden = "SYSTEM VERSIONED"]
    SystemVersioned,
}

#[derive(Debug, Default)]
pub struct TableQueryResult {
    pub table_name: String,
    pub engine: String,
    pub auto_increment: Option<u64>,
    pub table_char_set: String,
    pub table_collation: String,
    pub table_comment: String,
    pub create_options: String,
}

impl SchemaQueryBuilder {
    pub fn query_tables(&self, schema: SeaRc<dyn Iden>) -> SelectStatement {
        type Schema = InformationSchema;
        Query::select()
            .columns(vec![
                TablesFields::TableName,
                TablesFields::Engine,
                TablesFields::AutoIncrement,
                TablesFields::TableCollation,
                TablesFields::TableComment,
                TablesFields::CreateOptions,
            ])
            .column((
                Schema::CollationCharacterSet,
                CharacterSetFields::CharacterSetName,
            ))
            .from((Schema::Schema, Schema::Tables))
            .left_join(
                (Schema::Schema, Schema::CollationCharacterSet),
                Expr::col((
                    Schema::CollationCharacterSet,
                    CharacterSetFields::CollationName,
                ))
                .equals((Schema::Tables, TablesFields::TableCollation)),
            )
            .and_where(Expr::col(TablesFields::TableSchema).eq(schema.to_string()))
            .and_where(Expr::col(TablesFields::TableType).is_in([
                TableType::BaseTable.to_string(),
                TableType::SystemVersioned.to_string(),
            ]))
            .order_by(TablesFields::TableName, Order::Asc)
            .take()
    }
}

#[cfg(feature = "sqlx-mysql")]
impl From<&MySqlRow> for TableQueryResult {
    fn from(row: &MySqlRow) -> Self {
        use crate::sqlx_types::Row;
        Self {
            table_name: row.get(0),
            engine: row.get(1),
            auto_increment: row.get(2),
            table_collation: row.get(3),
            table_comment: row.get(4),
            create_options: row.get(5),
            table_char_set: row.get(6),
        }
    }
}

#[cfg(not(feature = "sqlx-mysql"))]
impl From<&MySqlRow> for TableQueryResult {
    fn from(_: &MySqlRow) -> Self {
        Self::default()
    }
}
