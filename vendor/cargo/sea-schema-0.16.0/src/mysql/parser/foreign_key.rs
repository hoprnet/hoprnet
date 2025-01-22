use crate::mysql::def::*;
use crate::mysql::query::ForeignKeyQueryResult;
use crate::Name;

pub struct ForeignKeyQueryResultParser {
    curr: Option<ForeignKeyInfo>,
    results: Box<dyn Iterator<Item = ForeignKeyQueryResult>>,
}

/// ForeignKeyQueryResult must be sorted by (TableName, ConstraintName, OrdinalPosition)
pub fn parse_foreign_key_query_results(
    results: Box<dyn Iterator<Item = ForeignKeyQueryResult>>,
) -> impl Iterator<Item = ForeignKeyInfo> {
    ForeignKeyQueryResultParser {
        curr: None,
        results,
    }
}

impl Iterator for ForeignKeyQueryResultParser {
    type Item = ForeignKeyInfo;

    fn next(&mut self) -> Option<Self::Item> {
        for result in self.results.by_ref() {
            let mut foreign_key = parse_foreign_key_query_result(result);
            if let Some(curr) = &mut self.curr {
                // group by `foreign_key.name`
                if curr.name == foreign_key.name {
                    curr.columns.push(foreign_key.columns.pop().unwrap());
                    curr.referenced_columns
                        .push(foreign_key.referenced_columns.pop().unwrap());
                } else {
                    let prev = self.curr.take();
                    self.curr = Some(foreign_key);
                    return prev;
                }
            } else {
                self.curr = Some(foreign_key);
            }
        }
        self.curr.take()
    }
}

pub fn parse_foreign_key_query_result(result: ForeignKeyQueryResult) -> ForeignKeyInfo {
    ForeignKeyInfo {
        name: result.constraint_name,
        columns: vec![result.column_name],
        referenced_table: result.referenced_table_name,
        referenced_columns: vec![result.referenced_column_name],
        on_update: parse_foreign_key_action(result.update_rule.as_str()),
        on_delete: parse_foreign_key_action(result.delete_rule.as_str()),
    }
}

pub fn parse_foreign_key_action(string: &str) -> ForeignKeyAction {
    ForeignKeyAction::from_str(string).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        assert_eq!(
            parse_foreign_key_query_results(Box::new(
                vec![
                    ForeignKeyQueryResult {
                        constraint_name: "fk-cat-dog".to_owned(),
                        column_name: "d1".to_owned(),
                        referenced_table_name: "cat".to_owned(),
                        referenced_column_name: "c1".to_owned(),
                        update_rule: "CASCADE".to_owned(),
                        delete_rule: "NO ACTION".to_owned(),
                    },
                    ForeignKeyQueryResult {
                        constraint_name: "fk-cat-dog".to_owned(),
                        column_name: "d2".to_owned(),
                        referenced_table_name: "cat".to_owned(),
                        referenced_column_name: "c2".to_owned(),
                        update_rule: "CASCADE".to_owned(),
                        delete_rule: "NO ACTION".to_owned(),
                    },
                ]
                .into_iter()
            ))
            .collect::<Vec<ForeignKeyInfo>>(),
            vec![ForeignKeyInfo {
                name: "fk-cat-dog".to_owned(),
                columns: vec!["d1".to_owned(), "d2".to_owned()],
                referenced_table: "cat".to_owned(),
                referenced_columns: vec!["c1".to_owned(), "c2".to_owned()],
                on_update: ForeignKeyAction::Cascade,
                on_delete: ForeignKeyAction::NoAction,
            }]
        );
    }
}
