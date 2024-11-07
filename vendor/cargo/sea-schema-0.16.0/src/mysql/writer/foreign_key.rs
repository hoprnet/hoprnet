use crate::mysql::def::{ForeignKeyAction, ForeignKeyInfo};
use sea_query::{Alias, ForeignKey, ForeignKeyCreateStatement};

impl ForeignKeyInfo {
    pub fn write(&self) -> ForeignKeyCreateStatement {
        let mut key = ForeignKey::create();
        key.name(&self.name)
            .to_tbl(Alias::new(&self.referenced_table));
        for column in self.columns.iter() {
            key.from_col(Alias::new(column.as_str()));
        }
        for ref_col in self.referenced_columns.iter() {
            key.to_col(Alias::new(ref_col.as_str()));
        }
        key.on_update(match self.on_update {
            ForeignKeyAction::Cascade => sea_query::ForeignKeyAction::Cascade,
            ForeignKeyAction::SetNull => sea_query::ForeignKeyAction::SetNull,
            ForeignKeyAction::SetDefault => sea_query::ForeignKeyAction::SetDefault,
            ForeignKeyAction::Restrict => sea_query::ForeignKeyAction::Restrict,
            ForeignKeyAction::NoAction => sea_query::ForeignKeyAction::NoAction,
        });
        key.on_delete(match self.on_delete {
            ForeignKeyAction::Cascade => sea_query::ForeignKeyAction::Cascade,
            ForeignKeyAction::SetNull => sea_query::ForeignKeyAction::SetNull,
            ForeignKeyAction::SetDefault => sea_query::ForeignKeyAction::SetDefault,
            ForeignKeyAction::Restrict => sea_query::ForeignKeyAction::Restrict,
            ForeignKeyAction::NoAction => sea_query::ForeignKeyAction::NoAction,
        });
        key.to_owned()
    }
}
