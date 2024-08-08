#[derive(Debug, sea_query::Iden)]
/// Ref: https://www.postgresql.org/docs/13/infoschema-key-column-usage.html
pub enum KeyColumnUsageFields {
    ConstraintCatalog,
    ConstraintSchema,
    ConstraintName,
    TableCatalog,
    TableSchema,
    TableName,
    ColumnName,
    OrdinalPosition,
    PositionInUniqueConstraint,
}
