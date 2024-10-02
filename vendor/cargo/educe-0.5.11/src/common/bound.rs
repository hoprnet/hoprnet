use syn::{punctuated::Punctuated, token::Comma, GenericParam, Meta, Path, Type, WherePredicate};

use crate::common::where_predicates_bool::{
    create_where_predicates_from_generic_parameters,
    create_where_predicates_from_generic_parameters_check_types, meta_2_where_predicates,
    WherePredicates, WherePredicatesOrBool,
};

pub(crate) enum Bound {
    Disabled,
    Auto,
    Custom(WherePredicates),
}

impl Bound {
    #[inline]
    pub(crate) fn from_meta(meta: &Meta) -> syn::Result<Self> {
        debug_assert!(meta.path().is_ident("bound"));

        Ok(match meta_2_where_predicates(meta)? {
            WherePredicatesOrBool::WherePredicates(where_predicates) => {
                Self::Custom(where_predicates)
            },
            WherePredicatesOrBool::Bool(b) => {
                if b {
                    Self::Auto
                } else {
                    Self::Disabled
                }
            },
        })
    }
}

impl Bound {
    #[inline]
    pub(crate) fn into_where_predicates_by_generic_parameters(
        self,
        params: &Punctuated<GenericParam, Comma>,
        bound_trait: &Path,
    ) -> Punctuated<WherePredicate, Comma> {
        match self {
            Self::Disabled => Punctuated::new(),
            Self::Auto => create_where_predicates_from_generic_parameters(params, bound_trait),
            Self::Custom(where_predicates) => where_predicates,
        }
    }

    #[inline]
    pub(crate) fn into_where_predicates_by_generic_parameters_check_types(
        self,
        params: &Punctuated<GenericParam, Comma>,
        bound_trait: &Path,
        types: &[&Type],
        recursive: Option<(bool, bool, bool)>,
    ) -> Punctuated<WherePredicate, Comma> {
        match self {
            Self::Disabled => Punctuated::new(),
            Self::Auto => create_where_predicates_from_generic_parameters_check_types(
                params,
                bound_trait,
                types,
                recursive,
            ),
            Self::Custom(where_predicates) => where_predicates,
        }
    }
}
