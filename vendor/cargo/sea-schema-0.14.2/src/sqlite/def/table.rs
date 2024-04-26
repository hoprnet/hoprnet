use sea_query::{
    Alias, ColumnDef, Expr, ForeignKey, Index, Keyword, Query, Table, TableCreateStatement, Value,
};

use super::{
    ColumnInfo, DefaultType, ForeignKeysInfo, IndexInfo, IndexedColumns, PartialIndexInfo,
};
use crate::sqlite::query::SqliteMaster;
use crate::sqlite::{error::DiscoveryResult, executor::Executor};

#[allow(unused_imports)]
use crate::sqlx_types::{sqlite::SqliteRow, Row};

/// Defines a table for SQLite
#[derive(Debug, Default, Clone)]
pub struct TableDef {
    /// The table name
    pub name: String,
    /// A list of foreign keys in the table
    pub foreign_keys: Vec<ForeignKeysInfo>,
    /// A list of the indexes in the table
    pub indexes: Vec<IndexInfo>,
    /// A list of UNIQUE and PRIMARY KEY constraints on the table
    pub constraints: Vec<IndexInfo>,
    /// A list of all the columns and their types
    pub columns: Vec<ColumnInfo>,
    /// Whether the primary key should autoincrement
    pub auto_increment: bool,
}

#[cfg(feature = "sqlx-sqlite")]
/// Gets the table name from a `SqliteRow` and maps it to the [TableDef]
impl From<&SqliteRow> for TableDef {
    fn from(row: &SqliteRow) -> Self {
        let row: String = row.get(0);
        TableDef {
            name: row,
            foreign_keys: Vec::default(),
            indexes: Vec::default(),
            constraints: Vec::default(),
            columns: Vec::default(),
            auto_increment: bool::default(),
        }
    }
}

#[cfg(not(feature = "sqlx-sqlite"))]
/// Gets the table name from a `SqliteRow` and maps it to the [TableDef]
impl From<&SqliteRow> for TableDef {
    fn from(_: &SqliteRow) -> Self {
        Self::default()
    }
}

impl TableDef {
    /// Check if the primary key in the table is set to autoincrement as a result of using query
    /// `SELECT COUNT(*) from sqlite_sequence where name = 'table_name';
    pub async fn pk_is_autoincrement(&mut self, executor: &Executor) -> DiscoveryResult<&mut Self> {
        let check_autoincrement = Query::select()
            .expr(Expr::val(1))
            .from(SqliteMaster)
            .and_where(Expr::col(Alias::new("type")).eq("table"))
            .and_where(Expr::col(Alias::new("name")).eq(self.name.as_str()))
            .and_where(Expr::col(Alias::new("sql")).like("%AUTOINCREMENT%"))
            .to_owned();

        if !executor.fetch_all(check_autoincrement).await?.is_empty() {
            self.auto_increment = true;
        }

        Ok(self)
    }

    /// Get a list of most of the UNIQUE and PRIMARY KEY constraints on the table.
    /// These are implemented by indexes in most cases. These indexes have type "u" or "pk".
    /// Note that this does not get the column name mapped by the index.
    /// To get the column name mapped by the index, the `self.get_single_indexinfo` method is invoked
    pub async fn get_constraints(&mut self, executor: &Executor) -> DiscoveryResult<()> {
        let mut index_query = String::default();
        index_query.push_str("PRAGMA index_list('");
        index_query.push_str(&self.name);
        index_query.push_str("')");

        let partial_index_info_rows = executor.fetch_all_raw(index_query).await?;
        let mut partial_indexes: Vec<PartialIndexInfo> = Vec::default();

        partial_index_info_rows.iter().for_each(|info| {
            let partial_index_info: PartialIndexInfo = info.into();

            if partial_index_info.origin.as_str() == "u" {
                partial_indexes.push(partial_index_info);
            }
        });

        for partial_index in partial_indexes {
            let partial_index_column: IndexedColumns = self
                .get_single_indexinfo(executor, &partial_index.name)
                .await?;

            self.constraints.push(IndexInfo {
                r#type: partial_index_column.r#type,
                index_name: partial_index_column.name,
                table_name: partial_index_column.table,
                unique: partial_index.unique,
                origin: partial_index.origin,
                partial: partial_index.partial,
                columns: partial_index_column.indexed_columns,
            });
        }

        Ok(())
    }

    /// Get a list of all the indexes in the table.
    /// Note that this does not get the column name mapped by the index.
    /// To get the column name mapped by the index, the `self.get_single_indexinfo` method is invoked
    pub async fn get_indexes(&mut self, executor: &Executor) -> DiscoveryResult<()> {
        let mut index_query = String::default();
        index_query.push_str("PRAGMA index_list('");
        index_query.push_str(&self.name);
        index_query.push_str("')");

        let partial_index_info_rows = executor.fetch_all_raw(index_query).await?;
        let mut partial_indexes: Vec<PartialIndexInfo> = Vec::default();

        partial_index_info_rows.iter().for_each(|info| {
            let partial_index_info: PartialIndexInfo = info.into();

            if partial_index_info.origin.as_str() == "c" {
                partial_indexes.push(partial_index_info);
            }
        });

        for partial_index in partial_indexes {
            let partial_index_column: IndexedColumns = self
                .get_single_indexinfo(executor, &partial_index.name)
                .await?;

            self.indexes.push(IndexInfo {
                r#type: partial_index_column.r#type,
                index_name: partial_index_column.name,
                table_name: partial_index_column.table,
                unique: partial_index.unique,
                origin: partial_index.origin,
                partial: partial_index.partial,
                columns: partial_index_column.indexed_columns,
            });
        }

        Ok(())
    }

    /// Get a list of all the foreign keys in the table
    pub async fn get_foreign_keys(&mut self, executor: &Executor) -> DiscoveryResult<&mut Self> {
        let mut index_query = String::default();
        index_query.push_str("PRAGMA foreign_key_list('");
        index_query.push_str(&self.name);
        index_query.push_str("')");

        let index_info_rows = executor.fetch_all_raw(index_query).await?;

        let mut last_fk_id = None;
        index_info_rows.iter().for_each(|info| {
            let mut index_info: ForeignKeysInfo = info.into();
            let fk_id = index_info.id;
            if last_fk_id == Some(fk_id) {
                let last_fk = self.foreign_keys.last_mut().unwrap();
                last_fk.from.push(index_info.from.pop().unwrap());
                last_fk.to.push(index_info.to.pop().unwrap());
            } else {
                self.foreign_keys.push(index_info);
            }
            last_fk_id = Some(fk_id);
        });

        Ok(self)
    }

    /// Get a list of all the columns in the table mapped as [ColumnInfo]
    pub async fn get_column_info(&mut self, executor: &Executor) -> DiscoveryResult<&TableDef> {
        let mut index_query = String::default();
        index_query.push_str("PRAGMA table_info('");
        index_query.push_str(&self.name);
        index_query.push_str("')");

        let index_info_rows = executor.fetch_all_raw(index_query).await?;

        for info in index_info_rows {
            let column = ColumnInfo::to_column_def(&info)?;
            self.columns.push(column);
        }

        Ok(self)
    }

    /// Gets the columns that are mapped to an index
    pub(crate) async fn get_single_indexinfo(
        &mut self,
        executor: &Executor,
        index_name: &str,
    ) -> DiscoveryResult<IndexedColumns> {
        let index_query = Query::select()
            .expr(Expr::cust("*"))
            .from(SqliteMaster)
            .and_where(Expr::col(Alias::new("name")).eq(index_name))
            .to_owned();

        let index_info = executor.fetch_one(index_query).await?;

        let mut index_column_query = String::default();
        index_column_query.push_str("PRAGMA index_info('");
        index_column_query.push_str(index_name);
        index_column_query.push_str("')");

        let index_column_info_rows = executor.fetch_all_raw(index_column_query).await?;

        Ok((&index_info, index_column_info_rows.as_slice()).into())
    }

    pub fn write(&self) -> TableCreateStatement {
        let mut primary_keys = Vec::new();

        let mut new_table = Table::create();
        new_table.table(Alias::new(&self.name));

        self.columns.iter().for_each(|column_info| {
            let mut new_column = ColumnDef::new(Alias::new(&column_info.name));
            if column_info.not_null {
                new_column.not_null();
            }

            if self.auto_increment && column_info.primary_key {
                new_column.primary_key().auto_increment();
            } else if column_info.primary_key {
                primary_keys.push(column_info.name.clone());
            }

            column_info.r#type.write_type(&mut new_column);

            match &column_info.default_value {
                DefaultType::Integer(integer_value) => {
                    new_column.default(Value::Int(Some(*integer_value)));
                }
                DefaultType::Float(float_value) => {
                    new_column.default(Value::Float(Some(*float_value)));
                }
                DefaultType::String(string_value) => {
                    new_column.default(Value::String(Some(Box::new(string_value.to_string()))));
                }
                DefaultType::Null => (),
                DefaultType::Unspecified => (),
                DefaultType::CurrentTimestamp => {
                    new_column.default(Keyword::CurrentTimestamp);
                }
            }

            new_table.col(&mut new_column);
        });

        self.foreign_keys.iter().for_each(|foreign_key| {
            let mut fk = ForeignKey::create();
            for from in foreign_key.from.iter() {
                fk.from(Alias::new(&self.name), Alias::new(from));
            }
            for to in foreign_key.to.iter() {
                fk.to(Alias::new(&foreign_key.table), Alias::new(to));
            }
            fk.on_delete(foreign_key.on_delete.to_seaquery_foreign_key_action())
                .on_update(foreign_key.on_update.to_seaquery_foreign_key_action());
            new_table.foreign_key(&mut fk);
        });

        self.constraints.iter().for_each(|index| {
            new_table.index(&mut index.write());
        });

        if !primary_keys.is_empty() {
            let mut primary_key_stmt = Index::create();
            for primary_key in primary_keys.iter() {
                primary_key_stmt.col(Alias::new(primary_key));
            }
            new_table.primary_key(&mut primary_key_stmt);
        }

        new_table
    }
}
