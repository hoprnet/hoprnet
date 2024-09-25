use std::collections::HashSet;

use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, GenericParam, Path, Type};

use crate::common::r#type::{dereference, find_idents_in_type};

#[inline]
pub(crate) fn create_debug_map_builder() -> proc_macro2::TokenStream {
    quote!(
        struct RawString(&'static str);

        impl ::core::fmt::Debug for RawString {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.0)
            }
        }

        let mut builder = f.debug_map();
    )
}

#[inline]
pub(crate) fn create_format_arg(
    params: &Punctuated<GenericParam, Comma>,
    ty: &Type,
    format_method: &Path,
    field: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let ty = dereference(ty);

    let mut idents = HashSet::new();
    find_idents_in_type(&mut idents, ty, Some((true, true, false)));

    // simply support one level generics (without considering bounds that use other generics)
    let mut filtered_params: Punctuated<GenericParam, Comma> = Punctuated::new();

    for param in params.iter() {
        if let GenericParam::Type(ty) = param {
            let ident = &ty.ident;

            if idents.contains(ident) {
                filtered_params.push(param.clone());
            }
        }
    }

    quote!(
        let arg = {
            struct MyDebug<'a, #filtered_params>(&'a #ty);

            impl<'a, #filtered_params> ::core::fmt::Debug for MyDebug<'a, #filtered_params> {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    #format_method(self.0, f)
                }
            }

            MyDebug(#field)
        };
    )
}
