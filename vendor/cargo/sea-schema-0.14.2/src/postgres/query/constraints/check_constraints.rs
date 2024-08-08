#[derive(Debug, sea_query::Iden)]
/// Ref: https://www.postgresql.org/docs/13/infoschema-check-constraints.html
pub enum CheckConstraintsFields {
    ConstraintCatalog,
    ConstraintSchema,
    ConstraintName,
    CheckClause,
}
