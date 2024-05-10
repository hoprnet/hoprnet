use crate::postgres::def::{ForeignKeyAction, PrimaryKey, References, Unique};
use sea_query::{Alias, ForeignKey, ForeignKeyCreateStatement, Index, IndexCreateStatement};

impl PrimaryKey {
    pub fn write(&self) -> IndexCreateStatement {
        let mut idx = Index::create();
        idx.primary().name(&self.name);
        for col in self.columns.iter() {
            idx.col(Alias::new(col));
        }
        idx.take()
    }
}

impl Unique {
    pub fn write(&self) -> IndexCreateStatement {
        let mut idx = Index::create();
        idx.unique().name(&self.name);
        for col in self.columns.iter() {
            idx.col(Alias::new(col));
        }
        idx.take()
    }
}

impl References {
    pub fn write(&self) -> ForeignKeyCreateStatement {
        let mut key = ForeignKey::create();
        key.name(&self.name);
        key.to_tbl(Alias::new(&self.table));
        for column in self.columns.iter() {
            key.from_col(Alias::new(column.as_str()));
        }
        for ref_col in self.foreign_columns.iter() {
            key.to_col(Alias::new(ref_col.as_str()));
        }
        if let Some(on_update) = &self.on_update {
            key.on_update(match on_update {
                ForeignKeyAction::Cascade => sea_query::ForeignKeyAction::Cascade,
                ForeignKeyAction::SetNull => sea_query::ForeignKeyAction::SetNull,
                ForeignKeyAction::SetDefault => sea_query::ForeignKeyAction::SetDefault,
                ForeignKeyAction::Restrict => sea_query::ForeignKeyAction::Restrict,
                ForeignKeyAction::NoAction => sea_query::ForeignKeyAction::NoAction,
            });
        }
        if let Some(on_delete) = &self.on_delete {
            key.on_delete(match on_delete {
                ForeignKeyAction::Cascade => sea_query::ForeignKeyAction::Cascade,
                ForeignKeyAction::SetNull => sea_query::ForeignKeyAction::SetNull,
                ForeignKeyAction::SetDefault => sea_query::ForeignKeyAction::SetDefault,
                ForeignKeyAction::Restrict => sea_query::ForeignKeyAction::Restrict,
                ForeignKeyAction::NoAction => sea_query::ForeignKeyAction::NoAction,
            });
        }
        key.take()
    }
}
