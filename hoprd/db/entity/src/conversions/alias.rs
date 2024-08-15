use hopr_primitive_types::primitives::Alias;

impl TryFrom<crate::codegen::sqlite::aliases::Model> for Alias {
    type Error = crate::errors::DbEntityError;

    fn try_from(value: crate::codegen::sqlite::aliases::Model) -> std::result::Result<Self, Self::Error> {
        Ok(Alias::new(value.peer_id, value.alias))
    }
}
