#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use super::{NotNull, Type};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ColumnInfo {
    /// The name of the column
    pub name: String,
    /// The type of the column with any additional definitions such as the precision or the character
    /// set
    pub col_type: ColumnType,
    /// The default value experssion for this column, if any
    pub default: Option<ColumnExpression>,
    /// The generation expression for this column, if it is a generated colum
    pub generated: Option<ColumnExpression>,
    pub not_null: Option<NotNull>,
    pub is_identity: bool,
    // TODO:
    // /// A constraint that ensures the value of a column is unique among all other rows in the table
    // pub unique: Option<Vec<constraints::Unique>>,
    // /// A constraint that states that the column is the unique identifier or part of the unique
    // /// identifier of each row for this table
    // pub primary_key: Option<constraints::PrimaryKey>,
    // /// A constraint that ensures that the value of this column must refer to a unique key in another
    // /// table
    // pub references: Option<constraints::References>,

    // FIXME: Include if there's a convenient way to look for this
    // /// Comments on the column made by the user
    // pub comment: String,
}

pub type ColumnType = Type;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct ColumnExpression(pub String);

impl ColumnExpression {
    pub fn from_option_string(maybe_string: Option<String>) -> Option<ColumnExpression> {
        maybe_string.map(ColumnExpression)
    }
}
