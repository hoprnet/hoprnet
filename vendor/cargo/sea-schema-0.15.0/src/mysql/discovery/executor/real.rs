use sea_query::{MysqlQueryBuilder, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{mysql::MySqlRow, MySqlPool};

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
