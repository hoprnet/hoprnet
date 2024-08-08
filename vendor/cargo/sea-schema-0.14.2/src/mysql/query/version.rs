use super::SchemaQueryBuilder;
use crate::sqlx_types::mysql::MySqlRow;
use sea_query::{Func, Query, SelectStatement};

#[derive(sea_query::Iden)]
enum MysqlFunc {
    Version,
}

#[derive(Debug, Default)]
pub struct VersionQueryResult {
    pub version: String,
}

impl SchemaQueryBuilder {
    pub fn query_version(&self) -> SelectStatement {
        Query::select().expr(Func::cust(MysqlFunc::Version)).take()
    }
}

#[cfg(feature = "sqlx-mysql")]
impl From<&MySqlRow> for VersionQueryResult {
    fn from(row: &MySqlRow) -> Self {
        use crate::sqlx_types::Row;
        Self {
            version: row.get(0),
        }
    }
}

#[cfg(not(feature = "sqlx-mysql"))]
impl From<&MySqlRow> for VersionQueryResult {
    fn from(_: &MySqlRow) -> Self {
        Self::default()
    }
}
