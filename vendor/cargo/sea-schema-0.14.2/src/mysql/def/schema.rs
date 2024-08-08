#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Schema {
    pub schema: String,
    pub system: SystemInfo,
    pub tables: Vec<TableDef>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct TableDef {
    pub info: TableInfo,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}
