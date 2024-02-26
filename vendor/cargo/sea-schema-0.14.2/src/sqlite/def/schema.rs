use super::{IndexInfo, TableDef};

#[derive(Clone, Debug)]
pub struct Schema {
    pub tables: Vec<TableDef>,
    pub indexes: Vec<IndexInfo>,
}
