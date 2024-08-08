use crate::postgres::def::*;
use crate::postgres::query::TableQueryResult;

impl TableQueryResult {
    pub fn parse(self) -> TableInfo {
        parse_table_query_result(self)
    }
}

pub fn parse_table_query_result(table_query: TableQueryResult) -> TableInfo {
    TableInfo {
        name: table_query.table_name,
        of_type: table_query
            .user_defined_type_name
            .map(|type_name| Type::from_str(&type_name, Some(&type_name), false)),
    }
}
