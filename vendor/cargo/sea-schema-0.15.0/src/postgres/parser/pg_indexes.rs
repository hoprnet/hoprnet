use crate::postgres::{def::*, query::UniqueIndexQueryResult};

pub struct UniqueIndexQueryResultParser {
    curr: Option<UniqueIndexQueryResult>,
    results: Box<dyn Iterator<Item = UniqueIndexQueryResult>>,
}

pub fn parse_unique_index_query_results(
    results: Box<dyn Iterator<Item = UniqueIndexQueryResult>>,
) -> impl Iterator<Item = Unique> {
    UniqueIndexQueryResultParser {
        curr: None,
        results,
    }
}

impl Iterator for UniqueIndexQueryResultParser {
    type Item = Unique;

    fn next(&mut self) -> Option<Self::Item> {
        let result = if let Some(result) = self.curr.take() {
            result
        } else {
            self.results.next()?
        };

        let index_name = result.index_name;
        let mut columns = vec![result.column_name];

        for result in self.results.by_ref() {
            if result.index_name != index_name {
                self.curr = Some(result);
                return Some(Unique {
                    name: index_name,
                    columns,
                });
            }

            columns.push(result.column_name);
        }

        Some(Unique {
            name: index_name,
            columns,
        })
    }
}
