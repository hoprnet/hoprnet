use super::*;
#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
/// Information relating to the table, but not its individual components. For information on a
/// table including its columns and constraints, use [`TableDef`]
pub struct TableInfo {
    pub name: String,
    pub of_type: Option<Type>,
    // TODO:
    // pub comment: String
}
