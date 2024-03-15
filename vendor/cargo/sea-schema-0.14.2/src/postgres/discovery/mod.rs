//! To query & parse MySQL's INFORMATION_SCHEMA and construct a [`Schema`]

use crate::debug_print;
use crate::postgres::def::*;
use crate::postgres::parser::parse_table_constraint_query_results;
use crate::postgres::query::{
    ColumnQueryResult, EnumQueryResult, SchemaQueryBuilder, TableConstraintsQueryResult,
    TableQueryResult,
};
use crate::sqlx_types::SqlxError;
use futures::future;
use sea_query::{Alias, Iden, IntoIden, SeaRc};
use std::collections::HashMap;

mod executor;
pub use executor::*;

pub(crate) type EnumVariantMap = HashMap<String, Vec<String>>;

pub struct SchemaDiscovery {
    pub query: SchemaQueryBuilder,
    pub executor: Executor,
    pub schema: SeaRc<dyn Iden>,
}

impl SchemaDiscovery {
    pub fn new<E>(executor: E, schema: &str) -> Self
    where
        E: IntoExecutor,
    {
        Self {
            query: SchemaQueryBuilder::default(),
            executor: executor.into_executor(),
            schema: Alias::new(schema).into_iden(),
        }
    }

    pub async fn discover(&self) -> Result<Schema, SqlxError> {
        let enums: EnumVariantMap = self
            .discover_enums()
            .await?
            .into_iter()
            .map(|enum_def| (enum_def.typename, enum_def.values))
            .collect();
        let tables = future::try_join_all(
            self.discover_tables()
                .await?
                .into_iter()
                .map(|t| (self, t, &enums))
                .map(Self::discover_table_static),
        )
        .await?;

        Ok(Schema {
            schema: self.schema.to_string(),
            tables,
        })
    }

    pub async fn discover_tables(&self) -> Result<Vec<TableInfo>, SqlxError> {
        let rows = self
            .executor
            .fetch_all(self.query.query_tables(self.schema.clone()))
            .await?;

        let tables: Vec<TableInfo> = rows
            .iter()
            .map(|row| {
                let result: TableQueryResult = row.into();
                debug_print!("{:?}", result);
                let table = result.parse();
                debug_print!("{:?}", table);
                table
            })
            .collect();

        Ok(tables)
    }

    async fn discover_table_static(
        params: (&Self, TableInfo, &EnumVariantMap),
    ) -> Result<TableDef, SqlxError> {
        let this = params.0;
        let info = params.1;
        let enums = params.2;
        Self::discover_table(this, info, enums).await
    }

    pub async fn discover_table(
        &self,
        info: TableInfo,
        enums: &EnumVariantMap,
    ) -> Result<TableDef, SqlxError> {
        let table = SeaRc::new(Alias::new(info.name.as_str()));
        let columns = self
            .discover_columns(self.schema.clone(), table.clone(), enums)
            .await?;
        let constraints = self
            .discover_constraints(self.schema.clone(), table.clone())
            .await?;
        let (
            check_constraints,
            not_null_constraints,
            unique_constraints,
            primary_key_constraints,
            reference_constraints,
            exclusion_constraints,
        ) = constraints.into_iter().fold(
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            |mut acc, constraint| {
                match constraint {
                    Constraint::Check(check) => acc.0.push(check),
                    Constraint::NotNull(not_null) => acc.1.push(not_null),
                    Constraint::Unique(unique) => acc.2.push(unique),
                    Constraint::PrimaryKey(primary_key) => acc.3.push(primary_key),
                    Constraint::References(references) => acc.4.push(references),
                    Constraint::Exclusion(exclusion) => acc.5.push(exclusion),
                }
                acc
            },
        );

        Ok(TableDef {
            info,
            columns,
            check_constraints,
            not_null_constraints,
            unique_constraints,
            primary_key_constraints,
            reference_constraints,
            exclusion_constraints,
        })
    }

    pub async fn discover_columns(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
        enums: &EnumVariantMap,
    ) -> Result<Vec<ColumnInfo>, SqlxError> {
        let rows = self
            .executor
            .fetch_all(self.query.query_columns(schema.clone(), table.clone()))
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let result: ColumnQueryResult = (&row).into();
                debug_print!("{:?}", result);
                let column = result.parse(enums);
                debug_print!("{:?}", column);
                column
            })
            .collect())
    }

    pub async fn discover_constraints(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
    ) -> Result<Vec<Constraint>, SqlxError> {
        let rows = self
            .executor
            .fetch_all(
                self.query
                    .query_table_constriants(schema.clone(), table.clone()),
            )
            .await?;

        let results = rows.into_iter().map(|row| {
            let result: TableConstraintsQueryResult = (&row).into();
            debug_print!("{:?}", result);
            result
        });

        Ok(parse_table_constraint_query_results(Box::new(results))
            .map(|index| {
                debug_print!("{:?}", index);
                index
            })
            .collect())
    }

    pub async fn discover_enums(&self) -> Result<Vec<EnumDef>, SqlxError> {
        let rows = self.executor.fetch_all(self.query.query_enums()).await?;

        let enum_rows = rows.into_iter().map(|row| {
            let result: EnumQueryResult = (&row).into();
            debug_print!("{:?}", result);
            result
        });

        let map = enum_rows.fold(
            HashMap::new(),
            |mut map: HashMap<String, Vec<String>>,
             EnumQueryResult {
                 typename,
                 enumlabel,
             }| {
                if let Some(entry_exists) = map.get_mut(&typename) {
                    entry_exists.push(enumlabel);
                } else {
                    map.insert(typename, vec![enumlabel]);
                }
                map
            },
        );

        Ok(map
            .into_iter()
            .map(|(typename, values)| EnumDef { values, typename })
            .collect())
    }
}
