use sea_query::{MysqlQueryBuilder, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{mysql::MySqlRow, MySqlPool, Row};

use crate::{debug_print, sqlx_types::SqlxError};

pub struct Executor {
    pool: MySqlPool,
}

pub trait IntoExecutor {
    fn into_executor(self) -> Executor;
}

impl IntoExecutor for MySqlPool {
    fn into_executor(self) -> Executor {
        Executor { pool: self }
    }
}

impl Executor {
    pub async fn fetch_all(&self, select: SelectStatement) -> Result<Vec<MySqlRow>, SqlxError> {
        let (sql, values) = select.build_sqlx(MysqlQueryBuilder);
        debug_print!("{}, {:?}", sql, values);

        sqlx::query_with(&sql, values)
            .fetch_all(&mut *self.pool.acquire().await?)
            .await
    }
}

pub trait GetMySqlValue {
    fn get_string(&self, idx: usize) -> String;

    fn get_string_opt(&self, idx: usize) -> Option<String>;
}

impl GetMySqlValue for MySqlRow {
    fn get_string(&self, idx: usize) -> String {
        String::from_utf8(self.get::<Vec<u8>, _>(idx)).unwrap()
    }

    fn get_string_opt(&self, idx: usize) -> Option<String> {
        self.get::<Option<Vec<u8>>, _>(idx)
            .map(|v| String::from_utf8(v).unwrap())
    }
}
