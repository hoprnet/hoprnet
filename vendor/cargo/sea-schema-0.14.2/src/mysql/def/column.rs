#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use super::Type;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ColumnInfo {
    /// The name of the column
    pub name: String,
    /// The type of the column with additional definitions, e.g. precision, length
    pub col_type: ColumnType,
    /// Can this column contains null
    pub null: bool,
    /// Is this column indexed
    pub key: ColumnKey,
    /// Default value expression for this column, if any
    pub default: Option<ColumnDefault>,
    /// Extra definitions for this column, e.g. auto_increment
    pub extra: ColumnExtra,
    /// The generation expression if this is a generated column
    pub expression: Option<ColumnExpression>,
    /// User comments
    pub comment: String,
}

pub type ColumnType = Type;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum ColumnKey {
    /// This column is not the first column of any key
    NotKey,
    /// This column is part of the primary key
    Primary,
    /// This column is the first column of a unique key
    Unique,
    /// This column is the first column of a non-unique key
    Multiple,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum ColumnDefault {
    Null,
    Int(i64),
    Real(f64),
    String(String),
    CustomExpr(String),
    CurrentTimestamp,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ColumnExpression {
    /// generation expression
    pub expr: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ColumnExtra {
    /// Auto increment
    pub auto_increment: bool,
    /// Only applies to timestamp or datetime
    pub on_update_current_timestamp: bool,
    /// This is a generated column
    pub generated: bool,
    /// This column has a default value expression
    pub default_generated: bool,
}
