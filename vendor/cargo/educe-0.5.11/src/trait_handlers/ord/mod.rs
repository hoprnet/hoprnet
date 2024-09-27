mod models;
mod ord_enum;
mod ord_struct;
mod panic;

use syn::{Data, DeriveInput, Meta};

use super::TraitHandler;
use crate::Trait;

pub(crate) struct OrdHandler;

impl TraitHandler for OrdHandler {
    #[inline]
    fn trait_meta_handler(
        ast: &mut DeriveInput,
        token_stream: &mut proc_macro2::TokenStream,
        traits: &[Trait],
        meta: &Meta,
    ) -> syn::Result<()> {
        match ast.data {
            Data::Struct(_) => {
                ord_struct::OrdStructHandler::trait_meta_handler(ast, token_stream, traits, meta)
            },
            Data::Enum(_) => {
                ord_enum::OrdEnumHandler::trait_meta_handler(ast, token_stream, traits, meta)
            },
            Data::Union(_) => {
                Err(crate::panic::trait_not_support_union(meta.path().get_ident().unwrap()))
            },
        }
    }
}
