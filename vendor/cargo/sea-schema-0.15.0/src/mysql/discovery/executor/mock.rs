use crate::sqlx_types::{mysql::MySqlRow, MySqlPool};
use sea_query::{MysqlQueryBuilder, SelectStatement};

use crate::{debug_print, sqlx_types::SqlxError};

#[allow(dead_code)]
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
        let (_sql, _values) = select.build(MysqlQueryBuilder);
        debug_print!("{}, {:?}", _sql, _values);

        panic!("This is a mock Executor");
    }
}
