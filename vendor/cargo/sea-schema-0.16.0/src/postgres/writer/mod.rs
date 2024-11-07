mod column;
mod constraints;
mod enumeration;
mod schema;
mod table;
mod types;

use super::def::Schema;
use sea_query::TableCreateStatement;

impl Schema {
    pub fn write(&self) -> Vec<TableCreateStatement> {
        self.tables.iter().map(|table| table.write()).collect()
    }
}
