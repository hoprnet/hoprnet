#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use crate as sea_schema;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ForeignKeyInfo {
    /// The name of the foreign key
    pub name: String,
    /// The columns composing this foreign key
    pub columns: Vec<String>,
    /// Referenced table name
    pub referenced_table: String,
    /// The columns composing the index of the referenced table
    pub referenced_columns: Vec<String>,
    /// Action on update
    pub on_update: ForeignKeyAction,
    /// Action on delete
    pub on_delete: ForeignKeyAction,
}

#[derive(Clone, Debug, PartialEq, sea_schema_derive::Name)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum ForeignKeyAction {
    #[name = "CASCADE"]
    Cascade,
    #[name = "SET NULL"]
    SetNull,
    #[name = "SET DEFAULT"]
    SetDefault,
    #[name = "RESTRICT"]
    Restrict,
    #[name = "NO ACTION"]
    NoAction,
}
