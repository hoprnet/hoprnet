#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use super::{CharSet, Collation, StorageEngine};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct TableInfo {
    /// The name of the table
    pub name: String,
    pub engine: StorageEngine,
    pub auto_increment: Option<u64>,
    pub char_set: CharSet,
    pub collation: Collation,
    pub comment: String,
}
