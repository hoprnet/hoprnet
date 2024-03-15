//! To query & parse MySQL's INFORMATION_SCHEMA and construct a [`Schema`]

use crate::debug_print;
use crate::mysql::def::*;
use crate::mysql::parser::{parse_foreign_key_query_results, parse_index_query_results};
use crate::mysql::query::{
    ColumnQueryResult, ForeignKeyQueryResult, IndexQueryResult, SchemaQueryBuilder,
    TableQueryResult, VersionQueryResult,
};
use crate::sqlx_types::SqlxError;
use futures::future;
use sea_query::{Alias, Iden, IntoIden, SeaRc};

mod executor;
pub use executor::*;

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

    pub async fn discover(mut self) -> Result<Schema, SqlxError> {
        self.query = SchemaQueryBuilder::new(self.discover_system().await?);
        let tables = self.discover_tables().await?;
        let tables = future::try_join_all(
            tables
                .into_iter()
                .map(|t| (&self, t))
                .map(Self::discover_table_static),
        )
        .await?;

        Ok(Schema {
            schema: self.schema.to_string(),
            system: self.query.system,
            tables,
        })
    }

    pub async fn discover_system(&mut self) -> Result<SystemInfo, SqlxError> {
        let rows = self.executor.fetch_all(self.query.query_version()).await?;

        #[allow(clippy::never_loop)]
        for row in rows.iter() {
            let result: VersionQueryResult = row.into();
            debug_print!("{:?}", result);
            let version = result.parse();
            debug_print!("{:?}", version);
            return Ok(version);
        }
        Err(SqlxError::RowNotFound)
    }

    pub async fn discover_tables(&mut self) -> Result<Vec<TableInfo>, SqlxError> {
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

    async fn discover_table_static(params: (&Self, TableInfo)) -> Result<TableDef, SqlxError> {
        let this = params.0;
        let info = params.1;
        Self::discover_table(this, info).await
    }

    pub async fn discover_table(&self, info: TableInfo) -> Result<TableDef, SqlxError> {
        let table = SeaRc::new(Alias::new(info.name.as_str()));
        let columns = self
            .discover_columns(self.schema.clone(), table.clone(), &self.query.system)
            .await?;
        let indexes = self
            .discover_indexes(self.schema.clone(), table.clone())
            .await?;
        let foreign_keys = self
            .discover_foreign_keys(self.schema.clone(), table.clone())
            .await?;

        Ok(TableDef {
            info,
            columns,
            indexes,
            foreign_keys,
        })
    }

    pub async fn discover_columns(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
        system: &SystemInfo,
    ) -> Result<Vec<ColumnInfo>, SqlxError> {
        let rows = self
            .executor
            .fetch_all(self.query.query_columns(schema.clone(), table.clone()))
            .await?;

        let columns = rows
            .iter()
            .map(|row| {
                let result: ColumnQueryResult = row.into();
                debug_print!("{:?}", result);
                let column = result.parse(system);
                debug_print!("{:?}", column);
                column
            })
            .collect::<Vec<_>>();

        Ok(columns)
    }

    pub async fn discover_indexes(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
    ) -> Result<Vec<IndexInfo>, SqlxError> {
        let rows = self
            .executor
            .fetch_all(self.query.query_indexes(schema.clone(), table.clone()))
            .await?;

        let results = rows.into_iter().map(|row| {
            let result: IndexQueryResult = (&row).into();
            debug_print!("{:?}", result);
            result
        });

        Ok(parse_index_query_results(Box::new(results))
            .map(|index| {
                debug_print!("{:?}", index);
                index
            })
            .collect())
    }

    pub async fn discover_foreign_keys(
        &self,
        schema: SeaRc<dyn Iden>,
        table: SeaRc<dyn Iden>,
    ) -> Result<Vec<ForeignKeyInfo>, SqlxError> {
        let rows = self
            .executor
            .fetch_all(self.query.query_foreign_key(schema.clone(), table.clone()))
            .await?;

        let results = rows.into_iter().map(|row| {
            let result: ForeignKeyQueryResult = (&row).into();
            debug_print!("{:?}", result);
            result
        });

        Ok(parse_foreign_key_query_results(Box::new(results))
            .map(|index| {
                debug_print!("{:?}", index);
                index
            })
            .collect())
    }
}
