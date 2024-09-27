use std::collections::HashSet;

use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Expr, GenericParam, Lit, Meta, MetaNameValue, Path, Token, Type, WherePredicate,
};

use super::{path::path_to_string, r#type::find_idents_in_type};

pub(crate) type WherePredicates = Punctuated<WherePredicate, Token![,]>;

pub(crate) enum WherePredicatesOrBool {
    WherePredicates(WherePredicates),
    Bool(bool),
}

impl Parse for WherePredicatesOrBool {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(lit) = input.parse::<Lit>() {
            match lit {
                Lit::Bool(lit) => return Ok(Self::Bool(lit.value)),
                Lit::Str(lit) => {
                    return match lit.parse_with(WherePredicates::parse_terminated) {
                        Ok(where_predicates) => Ok(Self::WherePredicates(where_predicates)),
                        Err(_) if lit.value().is_empty() => Ok(Self::Bool(false)),
                        Err(error) => Err(error),
                    }
                },
                _ => (),
            }
        }

        Ok(Self::WherePredicates(input.parse_terminated(WherePredicate::parse, Token![,])?))
    }
}

#[inline]
pub(crate) fn meta_name_value_2_where_predicates_bool(
    name_value: &MetaNameValue,
) -> syn::Result<WherePredicatesOrBool> {
    if let Expr::Lit(lit) = &name_value.value {
        match &lit.lit {
            Lit::Str(lit) => match lit.parse_with(WherePredicates::parse_terminated) {
                Ok(where_predicates) => {
                    return Ok(WherePredicatesOrBool::WherePredicates(where_predicates))
                },
                Err(_) if lit.value().is_empty() => {
                    return Ok(WherePredicatesOrBool::Bool(false));
                },
                Err(error) => {
                    return Err(error);
                },
            },
            Lit::Bool(lit) => {
                return Ok(WherePredicatesOrBool::Bool(lit.value));
            },
            _ => (),
        }
    }

    Err(syn::Error::new(
        name_value.value.span(),
        format!(
            "expected `{path} = \"where_predicates\"` or `{path} = false`",
            path = path_to_string(&name_value.path)
        ),
    ))
}

#[inline]
pub(crate) fn meta_2_where_predicates(meta: &Meta) -> syn::Result<WherePredicatesOrBool> {
    match &meta {
        Meta::NameValue(name_value) => meta_name_value_2_where_predicates_bool(name_value),
        Meta::List(list) => list.parse_args::<WherePredicatesOrBool>(),
        Meta::Path(path) => Err(syn::Error::new(
            path.span(),
            format!(
                "expected `{path} = \"where_predicates\"`, `{path}(where_predicates)`, `{path} = \
                 false`, or `{path}(false)`",
                path = path.clone().into_token_stream()
            ),
        )),
    }
}

#[inline]
pub(crate) fn create_where_predicates_from_generic_parameters(
    params: &Punctuated<GenericParam, Comma>,
    bound_trait: &Path,
) -> WherePredicates {
    let mut where_predicates = Punctuated::new();

    for param in params {
        if let GenericParam::Type(ty) = param {
            let ident = &ty.ident;

            where_predicates.push(syn::parse2(quote! { #ident: #bound_trait }).unwrap());
        }
    }

    where_predicates
}

#[inline]
pub(crate) fn create_where_predicates_from_generic_parameters_check_types(
    params: &Punctuated<GenericParam, Comma>,
    bound_trait: &Path,
    types: &[&Type],
    recursive: Option<(bool, bool, bool)>,
) -> WherePredicates {
    let mut where_predicates = Punctuated::new();

    let mut set = HashSet::new();

    for t in types {
        find_idents_in_type(&mut set, t, recursive);
    }

    for param in params {
        if let GenericParam::Type(ty) = param {
            let ident = &ty.ident;

            if set.contains(ident) {
                where_predicates.push(syn::parse2(quote! { #ident: #bound_trait }).unwrap());
            }
        }
    }

    where_predicates
}
