use crate::postgres::def::EnumDef;
use sea_query::{
    extension::postgres::{Type, TypeCreateStatement},
    Alias,
};

impl EnumDef {
    /// Converts the [EnumDef] to a [TypeCreateStatement]
    pub fn write(&self) -> TypeCreateStatement {
        Type::create()
            .as_enum(Alias::new(self.typename.as_str()))
            .values(self.values.iter().map(|val| Alias::new(val.as_str())))
            .to_owned()
    }
}
