use crate::mysql::def::*;
use crate::mysql::query::TableQueryResult;
use crate::Name;

impl TableQueryResult {
    pub fn parse(self) -> TableInfo {
        parse_table_query_result(self)
    }
}

pub fn parse_table_query_result(result: TableQueryResult) -> TableInfo {
    TableInfo {
        name: result.table_name,
        engine: StorageEngine::from_str(result.engine.as_str()).unwrap(),
        auto_increment: result.auto_increment,
        char_set: CharSet::from_str(result.table_char_set.as_str()).unwrap(),
        collation: Collation::from_str(result.table_collation.as_str()).unwrap(),
        comment: result.table_comment,
    }
}
