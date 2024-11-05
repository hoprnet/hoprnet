#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Schema {
    pub schema: String,
    pub tables: Vec<TableDef>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct TableDef {
    pub info: TableInfo,
    pub columns: Vec<ColumnInfo>,

    pub check_constraints: Vec<Check>,
    pub not_null_constraints: Vec<NotNull>,
    pub unique_constraints: Vec<Unique>,
    pub primary_key_constraints: Vec<PrimaryKey>,
    pub reference_constraints: Vec<References>,
    pub exclusion_constraints: Vec<Exclusion>,
    // FIXME: Duplication? TableInfo also have of_type
    // pub of_type: Option<Type>,
    // TODO:
    // pub inherets: Vec<String>,
}
