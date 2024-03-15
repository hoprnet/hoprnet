use crate::mysql::def::*;
use crate::mysql::query::IndexQueryResult;
use crate::Name;

pub struct IndexQueryResultParser {
    curr: Option<IndexInfo>,
    results: Box<dyn Iterator<Item = IndexQueryResult>>,
}

/// IndexQueryResult must be sorted by (TableName, IndexName, SeqInIndex)
pub fn parse_index_query_results(
    results: Box<dyn Iterator<Item = IndexQueryResult>>,
) -> impl Iterator<Item = IndexInfo> {
    IndexQueryResultParser {
        curr: None,
        results,
    }
}

impl Iterator for IndexQueryResultParser {
    type Item = IndexInfo;

    fn next(&mut self) -> Option<Self::Item> {
        for result in self.results.by_ref() {
            let mut index = parse_index_query_result(result);
            if let Some(curr) = &mut self.curr {
                // group by `index.name`, consolidate to `index.parts`
                if curr.name == index.name {
                    curr.parts.push(index.parts.pop().unwrap());
                    curr.functional |= index.functional;
                } else {
                    let prev = self.curr.take();
                    self.curr = Some(index);
                    return prev;
                }
            } else {
                self.curr = Some(index);
            }
        }
        self.curr.take()
    }
}

pub fn parse_index_query_result(mut result: IndexQueryResult) -> IndexInfo {
    IndexInfo {
        unique: match result.non_unique {
            0 => true,
            1 => false,
            _ => unimplemented!(),
        },
        name: result.index_name,
        parts: vec![IndexPart {
            column: if result.column_name.is_some() {
                result.column_name.take().unwrap()
            } else if result.expression.is_some() {
                result.expression.take().unwrap()
            } else {
                panic!("index column error")
            },
            order: match result.collation {
                Some(collation) => match collation.as_str() {
                    "A" => IndexOrder::Ascending,
                    "D" => IndexOrder::Descending,
                    _ => unimplemented!(),
                },
                None => IndexOrder::Unordered,
            },
            sub_part: result.sub_part.map(|v| v as u32),
        }],
        nullable: matches!(result.nullable.as_str(), "YES"),
        idx_type: IndexType::from_str(result.index_type.as_str()).unwrap(),
        comment: result.index_comment,
        functional: result.expression.is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        assert_eq!(
            parse_index_query_results(Box::new(
                vec![IndexQueryResult {
                    non_unique: 0,
                    index_name: "PRIMARY".to_owned(),
                    column_name: Some("film_id".to_owned()),
                    collation: Some("A".to_owned()),
                    sub_part: None,
                    nullable: "".to_owned(),
                    index_type: "BTREE".to_owned(),
                    index_comment: "".to_owned(),
                    expression: None
                }]
                .into_iter()
            ))
            .collect::<Vec<IndexInfo>>(),
            vec![IndexInfo {
                unique: true,
                name: "PRIMARY".to_owned(),
                parts: vec![IndexPart {
                    column: "film_id".to_owned(),
                    order: IndexOrder::Ascending,
                    sub_part: None,
                },],
                nullable: false,
                idx_type: IndexType::BTree,
                comment: "".to_owned(),
                functional: false,
            }]
        );
    }

    #[test]
    fn test_2() {
        assert_eq!(
            parse_index_query_results(Box::new(
                vec![IndexQueryResult {
                    non_unique: 1,
                    index_name: "idx_title".to_owned(),
                    column_name: Some("title".to_owned()),
                    collation: Some("A".to_owned()),
                    sub_part: None,
                    nullable: "".to_owned(),
                    index_type: "BTREE".to_owned(),
                    index_comment: "".to_owned(),
                    expression: None
                }]
                .into_iter()
            ))
            .collect::<Vec<IndexInfo>>(),
            vec![IndexInfo {
                unique: false,
                name: "idx_title".to_owned(),
                parts: vec![IndexPart {
                    column: "title".to_owned(),
                    order: IndexOrder::Ascending,
                    sub_part: None,
                },],
                nullable: false,
                idx_type: IndexType::BTree,
                comment: "".to_owned(),
                functional: false,
            }]
        );
    }

    #[test]
    fn test_3() {
        assert_eq!(
            parse_index_query_results(Box::new(
                vec![
                    IndexQueryResult {
                        non_unique: 0,
                        index_name: "rental_date".to_owned(),
                        column_name: Some("rental_date".to_owned()),
                        collation: Some("A".to_owned()),
                        sub_part: None,
                        nullable: "".to_owned(),
                        index_type: "BTREE".to_owned(),
                        index_comment: "".to_owned(),
                        expression: None
                    },
                    IndexQueryResult {
                        non_unique: 0,
                        index_name: "rental_date".to_owned(),
                        column_name: Some("inventory_id".to_owned()),
                        collation: Some("D".to_owned()),
                        sub_part: None,
                        nullable: "".to_owned(),
                        index_type: "BTREE".to_owned(),
                        index_comment: "".to_owned(),
                        expression: None
                    },
                    IndexQueryResult {
                        non_unique: 0,
                        index_name: "rental_date".to_owned(),
                        column_name: Some("customer_id".to_owned()),
                        collation: Some("A".to_owned()),
                        sub_part: None,
                        nullable: "".to_owned(),
                        index_type: "BTREE".to_owned(),
                        index_comment: "".to_owned(),
                        expression: None
                    },
                ]
                .into_iter()
            ))
            .collect::<Vec<IndexInfo>>(),
            vec![IndexInfo {
                unique: true,
                name: "rental_date".to_owned(),
                parts: vec![
                    IndexPart {
                        column: "rental_date".to_owned(),
                        order: IndexOrder::Ascending,
                        sub_part: None,
                    },
                    IndexPart {
                        column: "inventory_id".to_owned(),
                        order: IndexOrder::Descending,
                        sub_part: None,
                    },
                    IndexPart {
                        column: "customer_id".to_owned(),
                        order: IndexOrder::Ascending,
                        sub_part: None,
                    },
                ],
                nullable: false,
                idx_type: IndexType::BTree,
                comment: "".to_owned(),
                functional: false
            }]
        );
    }

    #[test]
    fn test_4() {
        assert_eq!(
            parse_index_query_results(Box::new(
                vec![IndexQueryResult {
                    non_unique: 1,
                    index_name: "idx_location".to_owned(),
                    column_name: Some("location".to_owned()),
                    collation: Some("A".to_owned()),
                    sub_part: Some(32),
                    nullable: "".to_owned(),
                    index_type: "SPATIAL".to_owned(),
                    index_comment: "".to_owned(),
                    expression: None
                }]
                .into_iter()
            ))
            .collect::<Vec<IndexInfo>>(),
            vec![IndexInfo {
                unique: false,
                name: "idx_location".to_owned(),
                parts: vec![IndexPart {
                    column: "location".to_owned(),
                    order: IndexOrder::Ascending,
                    sub_part: Some(32),
                },],
                nullable: false,
                idx_type: IndexType::Spatial,
                comment: "".to_owned(),
                functional: false,
            }]
        );
    }
}
