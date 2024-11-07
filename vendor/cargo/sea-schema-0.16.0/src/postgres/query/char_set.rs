#[derive(Debug, sea_query::Iden)]
/// Ref: https://www.postgresql.org/docs/13/infoschema-character-sets.html
pub enum CharacterSetFields {
    /// This column is null
    CharacterSetCatalog,
    /// This column is null
    CharacterSetSchema,

    CharacterSetName,
    ChacterRepetoire,
    FormOfUse,
    DefaultCollateCatalog,
    DefaultCollateSchema,
    DefaultCollateName,
}
