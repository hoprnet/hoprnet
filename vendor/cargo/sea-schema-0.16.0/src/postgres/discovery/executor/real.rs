use sea_query::{PostgresQueryBuilder, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, PgPool};

use crate::{debug_print, sqlx_types::SqlxError};

pub struct Executor {
    pool: PgPool,
}

pub trait IntoExecutor {
    fn into_executor(self) -> Executor;
}

impl IntoExecutor for PgPool {
    fn into_executor(self) -> Executor {
        Executor { pool: self }
    }
}

impl Executor {
    pub async fn fetch_all(&self, select: SelectStatement) -> Result<Vec<PgRow>, SqlxError> {
        let (sql, values) = select.build_sqlx(PostgresQueryBuilder);
        debug_print!("{}, {:?}", sql, values);

        sqlx::query_with(&sql, values)
            .fetch_all(&mut *self.pool.acquire().await?)
            .await
    }
}
