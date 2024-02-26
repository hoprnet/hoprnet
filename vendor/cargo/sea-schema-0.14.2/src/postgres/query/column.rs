use super::{InformationSchema, SchemaQueryBuilder};
use crate::sqlx_types::postgres::PgRow;
use sea_query::{BinOper, Expr, Iden, Query, SeaRc, SelectStatement};

#[derive(Debug, sea_query::Iden)]
/// Ref: https://www.postgresql.org/docs/13/infoschema-columns.html
pub enum ColumnsField {
    TableCatalog,
    TableSchema,
    TableName,
    ColumnName,
    OrdinalPosition,
    ColumnDefault,
    IsNullable,
    DataType,
    CharacterMaximumLength,
    CharacterOctetLength,
    NumericPrecision,
    NumericPrecisionRadix,
    NumericScale,
    DatetimePrecision,
    IntervalType,
    IntervalPrecision,
    CollationCatalog,
    CollationSchema,
    CollationName,
    DomainCatalog,
    DomainSchema,
    DomainName,
    UdtCatalog,
    UdtSchema,
    UdtName,
    DtdIdentifier,
    IsIdentity,
    IdentityGeneration,
    IdentityStart,
    IdentityIncrement,
    IdentityMaximum,
    IdentityMinimum,
    IdentityCycle,
    IsGenerated,
    GenerationExpression,
    IsUpdatable,
}

#[derive(Debug, Default)]
pub struct ColumnQueryResult {
    pub column_name: String,
    pub column_type: String,
    pub column_default: Option<String>,
    pub column_generated: Option<String>,
    pub is_nullable: String,
    pub is_identity: String,

    // Declared or implicit parameters of numeric types; null for other data types
    pub numeric_precision: Option<i32>,
    pub numeric_precision_radix: Option<i32>,
    pub numeric_scale: Option<i32>,

    pub character_maximum_length: Option<i32>,
    pub character_octet_length: Option<i32>,

    pub datetime_precision: Option<i32>,

    pub interval_type: Option<String>,
    pub interval_precision: Option<i32>,

    pub udt_name: Option<String>,
    pub udt_name_regtype: Option<String>,
}

impl SchemaQueryBuilder {
    pub fn query_columns(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
    ) -> SelectStatement {
        Query::select()
            .columns([
                ColumnsField::ColumnName,
                ColumnsField::DataType,
                ColumnsField::ColumnDefault,
                ColumnsField::GenerationExpression,
                ColumnsField::IsNullable,
                ColumnsField::IsIdentity,
                ColumnsField::NumericPrecision,
                ColumnsField::NumericPrecisionRadix,
                ColumnsField::NumericScale,
                ColumnsField::CharacterMaximumLength,
                ColumnsField::CharacterOctetLength,
                ColumnsField::DatetimePrecision,
                ColumnsField::IntervalType,
                ColumnsField::IntervalPrecision,
                ColumnsField::UdtName,
            ])
            .expr(
                // The double quotes are required to correctly handle user types containing
                // upper case letters.
                Expr::expr(Expr::cust("CONCAT('\"', udt_name, '\"')::regtype").cast_as(Text))
                    .binary(BinOper::As, Expr::col(UdtNameRegtype)),
            )
            .from((InformationSchema::Schema, InformationSchema::Columns))
            .and_where(Expr::col(ColumnsField::TableSchema).eq(schema.to_string()))
            .and_where(Expr::col(ColumnsField::TableName).eq(table.to_string()))
            .take()
    }
}

#[cfg(feature = "sqlx-postgres")]
impl From<&PgRow> for ColumnQueryResult {
    fn from(row: &PgRow) -> Self {
        use crate::sqlx_types::Row;
        Self {
            column_name: row.get(0),
            column_type: row.get(1),
            column_default: row.get(2),
            column_generated: row.get(3),
            is_nullable: row.get(4),
            is_identity: row.get(5),
            numeric_precision: row.get(6),
            numeric_precision_radix: row.get(7),
            numeric_scale: row.get(8),
            character_maximum_length: row.get(9),
            character_octet_length: row.get(10),
            datetime_precision: row.get(11),
            interval_type: row.get(12),
            interval_precision: row.get(13),
            udt_name: row.get(14),
            udt_name_regtype: row.get(15),
        }
    }
}

#[cfg(not(feature = "sqlx-postgres"))]
impl From<&PgRow> for ColumnQueryResult {
    fn from(_: &PgRow) -> Self {
        Self::default()
    }
}

#[derive(Iden)]
struct Text;
#[derive(Iden)]
struct UdtNameRegtype;
