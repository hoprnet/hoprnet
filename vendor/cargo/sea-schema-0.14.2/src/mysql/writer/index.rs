use crate::mysql::def::{IndexInfo, IndexOrder, IndexType};
use sea_query::{Alias, Iden, Index, IndexCreateStatement, SeaRc};

impl IndexInfo {
    #[allow(clippy::unnecessary_unwrap)]
    pub fn write(&self) -> IndexCreateStatement {
        let mut index = Index::create();
        if self.name == "PRIMARY" {
            index.primary();
        } else {
            index.name(&self.name);
            if self.unique {
                index.unique();
            }
        }
        for part in self.parts.iter() {
            let pre = part.sub_part;
            let ord = if self.parts.len() == 1 {
                match part.order {
                    IndexOrder::Ascending => None,
                    IndexOrder::Descending => Some(sea_query::IndexOrder::Desc),
                    IndexOrder::Unordered => None,
                }
            } else {
                None
            };
            if pre.is_none() && ord.is_none() {
                index.col(Alias::new(&part.column));
            } else if pre.is_none() && ord.is_some() {
                index.col((Alias::new(&part.column), ord.unwrap()));
            } else if pre.is_some() && ord.is_none() {
                index.col((Alias::new(&part.column), pre.unwrap()));
            } else {
                index.col((Alias::new(&part.column), pre.unwrap(), ord.unwrap()));
            }
        }
        match self.idx_type {
            IndexType::BTree => {}
            IndexType::FullText => {
                index.index_type(sea_query::IndexType::FullText);
            }
            IndexType::Hash => {
                index.index_type(sea_query::IndexType::Hash);
            }
            IndexType::RTree => {
                index.index_type(sea_query::IndexType::Custom(SeaRc::new(Alias::new(
                    self.idx_type.to_string(),
                ))));
            }
            IndexType::Spatial => {
                index.index_type(sea_query::IndexType::Custom(SeaRc::new(Alias::new(
                    self.idx_type.to_string(),
                ))));
            }
        }
        index
    }
}
