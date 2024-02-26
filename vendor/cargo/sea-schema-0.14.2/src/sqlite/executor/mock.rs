use crate::sqlx_types::{sqlite::SqliteRow, SqlitePool};
use sea_query::{SelectStatement, SqliteQueryBuilder};

use crate::{debug_print, sqlx_types::SqlxError};

#[allow(dead_code)]
pub struct Executor {
    pool: SqlitePool,
}

pub trait IntoExecutor {
    fn into_executor(self) -> Executor;
}

impl IntoExecutor for SqlitePool {
    fn into_executor(self) -> Executor {
        Executor { pool: self }
    }
}

impl Executor {
    pub async fn fetch_all(&self, select: SelectStatement) -> Result<Vec<SqliteRow>, SqlxError> {
        let (_sql, _values) = select.build(SqliteQueryBuilder);
        debug_print!("{}, {:?}", _sql, _values);

        panic!("This is a mock Executor");
    }

    pub async fn fetch_one(&self, select: SelectStatement) -> Result<SqliteRow, SqlxError> {
        let (_sql, _values) = select.build(SqliteQueryBuilder);
        debug_print!("{}, {:?}", _sql, _values);

        panic!("This is a mock Executor");
    }

    pub async fn fetch_all_raw(&self, _sql: String) -> Result<Vec<SqliteRow>, SqlxError> {
        debug_print!("{}", _sql);

        panic!("This is a mock Executor");
    }
}
