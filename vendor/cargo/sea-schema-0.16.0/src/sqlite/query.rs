use sea_query::Iden;

#[derive(Debug, Iden)]
pub struct SqliteMaster;

#[derive(Debug, Iden)]
pub enum SqliteSchema {
    Type,
    Name,
    TblName,
    #[iden = "rootpage"]
    RootPage,
    Sql,
}
