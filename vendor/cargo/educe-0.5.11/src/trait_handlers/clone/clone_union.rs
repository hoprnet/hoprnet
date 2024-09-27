use quote::quote;
use syn::{Data, DeriveInput, Meta};

use super::{
    models::{FieldAttributeBuilder, TypeAttributeBuilder},
    TraitHandler,
};
use crate::supported_traits::Trait;

pub(crate) struct CloneUnionHandler;

impl TraitHandler for CloneUnionHandler {
    fn trait_meta_handler(
        ast: &mut DeriveInput,
        token_stream: &mut proc_macro2::TokenStream,
        traits: &[Trait],
        meta: &Meta,
    ) -> syn::Result<()> {
        let type_attribute = TypeAttributeBuilder {
            enable_flag: true, enable_bound: true
        }
        .build_from_clone_meta(meta)?;

        if let Data::Union(data) = &ast.data {
            for field in data.fields.named.iter() {
                let _ = FieldAttributeBuilder {
                    enable_method: false
                }
                .build_from_attributes(&field.attrs, traits)?;
            }
        }

        let ident = &ast.ident;

        let bound = type_attribute.bound.into_where_predicates_by_generic_parameters(
            &ast.generics.params,
            &syn::parse2(quote!(::core::marker::Copy)).unwrap(),
        );

        let where_clause = ast.generics.make_where_clause();

        for where_predicate in bound {
            where_clause.predicates.push(where_predicate);
        }

        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

        token_stream.extend(quote! {
            impl #impl_generics ::core::clone::Clone for #ident #ty_generics #where_clause {
                #[inline]
                fn clone(&self) -> Self {
                    *self
                }
            }
        });

        #[cfg(feature = "Copy")]
        if traits.contains(&Trait::Copy) {
            token_stream.extend(quote! {
                impl #impl_generics ::core::marker::Copy for #ident #ty_generics #where_clause {
                }
            });
        }

        Ok(())
    }
}
