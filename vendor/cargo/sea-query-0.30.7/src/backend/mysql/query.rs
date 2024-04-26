use super::*;
use crate::extension::mysql::*;

impl QueryBuilder for MysqlQueryBuilder {
    fn values_list_tuple_prefix(&self) -> &str {
        "ROW"
    }

    fn prepare_select_distinct(&self, select_distinct: &SelectDistinct, sql: &mut dyn SqlWriter) {
        match select_distinct {
            SelectDistinct::All => write!(sql, "ALL").unwrap(),
            SelectDistinct::Distinct => write!(sql, "DISTINCT").unwrap(),
            SelectDistinct::DistinctRow => write!(sql, "DISTINCTROW").unwrap(),
            _ => {}
        };
    }

    fn prepare_index_hints(&self, select: &SelectStatement, sql: &mut dyn SqlWriter) {
        if !select.index_hints.is_empty() {
            write!(sql, " ").unwrap();
        }
        for (i, hint) in select.index_hints.iter().enumerate() {
            if i != 0 {
                write!(sql, " ").unwrap()
            }
            match hint.r#type {
                IndexHintType::Use => {
                    write!(sql, "USE INDEX ",).unwrap();
                    self.prepare_index_hint_scope(&hint.scope, sql);
                    write!(sql, "(").unwrap();
                    hint.index.prepare(sql.as_writer(), self.quote());
                }
                IndexHintType::Ignore => {
                    write!(sql, "IGNORE INDEX ",).unwrap();
                    self.prepare_index_hint_scope(&hint.scope, sql);
                    write!(sql, "(").unwrap();
                    hint.index.prepare(sql.as_writer(), self.quote());
                }
                IndexHintType::Force => {
                    write!(sql, "FORCE INDEX ",).unwrap();
                    self.prepare_index_hint_scope(&hint.scope, sql);
                    write!(sql, "(").unwrap();
                    hint.index.prepare(sql.as_writer(), self.quote());
                }
            }
            write!(sql, ")").unwrap();
        }
    }

    fn prepare_query_statement(&self, query: &SubQueryStatement, sql: &mut dyn SqlWriter) {
        query.prepare_statement(self, sql);
    }

    fn prepare_with_clause_recursive_options(&self, _: &WithClause, _: &mut dyn SqlWriter) {
        // MySQL doesn't support sql recursive with query 'SEARCH' and 'CYCLE' options.
    }

    fn prepare_with_query_clause_materialization(
        &self,
        _: &CommonTableExpression,
        _: &mut dyn SqlWriter,
    ) {
        // MySQL doesn't support declaring materialization in SQL for with query.
    }

    fn prepare_join_type(&self, join_type: &JoinType, sql: &mut dyn SqlWriter) {
        match join_type {
            JoinType::FullOuterJoin => panic!("Mysql does not support FULL OUTER JOIN"),
            _ => self.prepare_join_type_common(join_type, sql),
        }
    }

    fn prepare_order_expr(&self, order_expr: &OrderExpr, sql: &mut dyn SqlWriter) {
        match order_expr.nulls {
            None => (),
            Some(NullOrdering::Last) => {
                self.prepare_simple_expr(&order_expr.expr, sql);
                write!(sql, " IS NULL ASC, ").unwrap()
            }
            Some(NullOrdering::First) => {
                self.prepare_simple_expr(&order_expr.expr, sql);
                write!(sql, " IS NULL DESC, ").unwrap()
            }
        }
        if !matches!(order_expr.order, Order::Field(_)) {
            self.prepare_simple_expr(&order_expr.expr, sql);
        }
        self.prepare_order(order_expr, sql);
    }

    fn prepare_value(&self, value: &Value, sql: &mut dyn SqlWriter) {
        sql.push_param(value.clone(), self as _);
    }

    fn prepare_on_conflict_target(&self, _: &[OnConflictTarget], _: &mut dyn SqlWriter) {
        // MySQL doesn't support declaring ON CONFLICT target.
    }

    fn prepare_on_conflict_keywords(&self, sql: &mut dyn SqlWriter) {
        write!(sql, " ON DUPLICATE KEY").unwrap();
    }

    fn prepare_on_conflict_do_update_keywords(&self, sql: &mut dyn SqlWriter) {
        write!(sql, " UPDATE ").unwrap();
    }

    fn prepare_on_conflict_excluded_table(&self, col: &DynIden, sql: &mut dyn SqlWriter) {
        write!(sql, "VALUES(").unwrap();
        col.prepare(sql.as_writer(), self.quote());
        write!(sql, ")").unwrap();
    }

    fn prepare_on_conflict_condition(&self, _: &ConditionHolder, _: &mut dyn SqlWriter) {}

    fn prepare_returning(&self, _returning: &Option<ReturningClause>, _sql: &mut dyn SqlWriter) {}

    fn random_function(&self) -> &str {
        "RAND"
    }

    fn insert_default_keyword(&self) -> &str {
        "()"
    }
}

impl MysqlQueryBuilder {
    fn prepare_index_hint_scope(&self, index_hint_scope: &IndexHintScope, sql: &mut dyn SqlWriter) {
        match index_hint_scope {
            IndexHintScope::Join => {
                write!(sql, "FOR JOIN ").unwrap();
            }
            IndexHintScope::OrderBy => {
                write!(sql, "FOR ORDER BY ").unwrap();
            }
            IndexHintScope::GroupBy => {
                write!(sql, "FOR GROUP BY ").unwrap();
            }
            IndexHintScope::All => {}
        }
    }
}
