#[derive(Debug, sea_query::Iden)]
/// Ref: https://dev.mysql.com/doc/refman/8.0/en/information-schema-collation-character-set-applicability-table.html
pub enum CharacterSetFields {
    CharacterSetName,
    CollationName,
}
