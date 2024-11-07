use crate::postgres::{def::*, query::TableConstraintsQueryResult};
use crate::Name;

pub struct TableConstraintsQueryResultParser {
    curr: Option<TableConstraintsQueryResult>,
    results: Box<dyn Iterator<Item = TableConstraintsQueryResult>>,
}

/// Assumed to be ordered by table name, then constraint name, then ordinal position, then the
/// constraint name of the foreign key, then the ordinal position of the foreign key
pub fn parse_table_constraint_query_results(
    results: Box<dyn Iterator<Item = TableConstraintsQueryResult>>,
) -> impl Iterator<Item = Constraint> {
    TableConstraintsQueryResultParser {
        curr: None,
        results,
    }
}

impl Iterator for TableConstraintsQueryResultParser {
    type Item = Constraint;

    // FIXME/TODO: How to handle invalid input
    fn next(&mut self) -> Option<Self::Item> {
        let result = if let Some(result) = self.curr.take() {
            result
        } else {
            self.results.next()?
        };

        let constraint_name = result.constraint_name;
        match result.constraint_type.as_str() {
            "CHECK" => {
                match result.check_clause {
                    Some(check_clause) => {
                        Some(Constraint::Check(Check {
                            name: constraint_name,
                            expr: check_clause,
                            // TODO: How to find?
                            no_inherit: false,
                        }))
                    }
                    None => self.next(),
                }
            }

            "FOREIGN KEY" => {
                let mut columns = Vec::new();
                let mut foreign_columns = Vec::new();

                columns.push(result.column_name.unwrap());
                let table = result.referential_key_table_name.unwrap();
                foreign_columns.push(result.referential_key_column_name.unwrap());
                let on_update =
                    ForeignKeyAction::from_str(&result.update_rule.clone().unwrap_or_default());
                let on_delete =
                    ForeignKeyAction::from_str(&result.delete_rule.clone().unwrap_or_default());

                for result in self.results.by_ref() {
                    if result.constraint_name != constraint_name {
                        self.curr = Some(result);
                        return Some(Constraint::References(References {
                            name: constraint_name,
                            columns,
                            table,
                            foreign_columns,
                            on_update,
                            on_delete,
                        }));
                    }

                    if result.column_name.is_some() && result.referential_key_column_name.is_some()
                    {
                        columns.push(result.column_name.unwrap());
                        foreign_columns.push(result.referential_key_column_name.unwrap());
                    }
                }

                Some(Constraint::References(References {
                    name: constraint_name,
                    columns,
                    table,
                    foreign_columns,
                    on_update,
                    on_delete,
                }))
            }

            "PRIMARY KEY" => {
                let mut columns = vec![result.column_name.unwrap()];

                for result in self.results.by_ref() {
                    if result.constraint_name != constraint_name {
                        self.curr = Some(result);
                        return Some(Constraint::PrimaryKey(PrimaryKey {
                            name: constraint_name,
                            columns,
                        }));
                    }

                    columns.push(result.column_name.unwrap());
                }

                Some(Constraint::PrimaryKey(PrimaryKey {
                    name: constraint_name,
                    columns,
                }))
            }

            "UNIQUE" => {
                let mut columns = vec![result.column_name.unwrap()];

                for result in self.results.by_ref() {
                    if result.constraint_name != constraint_name {
                        self.curr = Some(result);
                        return Some(Constraint::Unique(Unique {
                            name: constraint_name,
                            columns,
                        }));
                    }

                    columns.push(result.column_name.unwrap());
                }

                Some(Constraint::Unique(Unique {
                    name: constraint_name,
                    columns,
                }))
            }

            _ => {
                // FIXME: Invalid input error handling
                None
            }
        }
    }
}
