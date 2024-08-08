//! To write [`mysql::Schema`] to SQL statements

mod column;
mod foreign_key;
mod index;
mod table;
mod types;

use super::def::Schema;
use sea_query::TableCreateStatement;

impl Schema {
    pub fn write(&self) -> Vec<TableCreateStatement> {
        self.tables.iter().map(|table| table.write()).collect()
    }
}
