#[cfg(feature = "with-serde")]
use serde::{Deserialize, Serialize};

use crate as sea_schema;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct IndexInfo {
    /// Does this index requires unique values
    pub unique: bool,
    /// The name of the index
    pub name: String,
    /// The parts composing this index
    pub parts: Vec<IndexPart>,
    /// Does this index allow null values
    pub nullable: bool,
    /// BTree (the default), full-text etc
    pub idx_type: IndexType,
    /// User comments
    pub comment: String,
    /// True if part of the index is computed
    pub functional: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct IndexPart {
    /// Identifier for this column. If functional is true, may contain expression.
    pub column: String,
    /// Ascending, descending or unordered
    pub order: IndexOrder,
    /// If the whole column is indexed, this value is null. Otherwise the number indicates number of characters indexed
    pub sub_part: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum IndexOrder {
    Ascending,
    Descending,
    Unordered,
}

#[derive(Clone, Debug, PartialEq, sea_query::Iden, sea_schema_derive::Name)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum IndexType {
    #[iden = "BTREE"]
    BTree,
    #[iden = "FULLTEXT"]
    FullText,
    #[iden = "HASH"]
    Hash,
    #[iden = "RTREE"]
    RTree,
    #[iden = "SPATIAL"]
    Spatial,
}
