use sea_query::Iden;

#[derive(Debug, Iden)]
pub enum PgIndexes {
    Table,
    #[iden = "tablename"]
    TableName,
    #[iden = "schemaname"]
    SchemaName,
    #[iden = "indexname"]
    IndexName,
}
