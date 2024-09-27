#[derive(Debug, sea_query::Iden)]
/// Ref: https://www.postgresql.org/docs/13/infoschema-table-constraints.html
pub enum TableConstraintsField {
    ConstraintCatalog,
    ConstraintSchema,
    ConstraintName,
    TableCatalog,
    TableSchema,
    TableName,
    ConstraintType,
    IsDeferrable,
    InitiallyDeferred,
}
