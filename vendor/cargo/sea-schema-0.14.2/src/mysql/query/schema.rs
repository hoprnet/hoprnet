use crate::mysql::def::SystemInfo;

#[derive(Debug, Default)]
pub struct SchemaQueryBuilder {
    pub system: SystemInfo,
}

impl SchemaQueryBuilder {
    pub fn new(system: SystemInfo) -> Self {
        Self { system }
    }
}

#[derive(Debug, sea_query::Iden)]
/// Ref: https://dev.mysql.com/doc/refman/8.0/en/information-schema.html
pub enum InformationSchema {
    #[iden = "information_schema"]
    Schema,
    Tables,
    Columns,
    Statistics,
    KeyColumnUsage,
    ReferentialConstraints,
    #[iden = "collation_character_set_applicability"]
    CollationCharacterSet,
}
