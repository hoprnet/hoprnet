// Copyright (C) 2018-2022 Stephane Raux. Distributed under the 0BSD license.

//! # Overview
//! - [📦 crates.io](https://crates.io/crates/enum-iterator-derive)
//! - [📖 Documentation](https://docs.rs/enum-iterator-derive)
//! - [⚖ 0BSD license](https://spdx.org/licenses/0BSD.html)
//!
//! Procedural macro to derive `Sequence`.
//!
//! See crate [`enum-iterator`](https://docs.rs/enum-iterator) for details.
//!
//! # Contribute
//! All contributions shall be licensed under the [0BSD license](https://spdx.org/licenses/0BSD.html).

#![recursion_limit = "128"]
#![deny(warnings)]

extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    iter::{self, once, repeat},
};
use syn::{
    punctuated::Punctuated, token::Comma, DeriveInput, Field, Fields, Generics, Ident, Member,
    Path, PathSegment, PredicateType, TraitBound, TraitBoundModifier, Type, TypeParamBound,
    Variant, WhereClause, WherePredicate,
};

/// Derives `Sequence`.
#[proc_macro_derive(Sequence)]
pub fn derive_sequence(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn derive(input: proc_macro::TokenStream) -> Result<TokenStream, syn::Error> {
    derive_for_ast(syn::parse(input)?)
}

fn derive_for_ast(ast: DeriveInput) -> Result<TokenStream, syn::Error> {
    let ty = &ast.ident;
    let generics = &ast.generics;
    match &ast.data {
        syn::Data::Struct(s) => derive_for_struct(ty, generics, &s.fields),
        syn::Data::Enum(e) => derive_for_enum(ty, generics, &e.variants),
        syn::Data::Union(_) => Err(Error::UnsupportedUnion.with_tokens(&ast)),
    }
}

fn derive_for_struct(
    ty: &Ident,
    generics: &Generics,
    fields: &Fields,
) -> Result<TokenStream, syn::Error> {
    let cardinality = tuple_cardinality(fields);
    let first = init_value(ty, None, fields, Direction::Forward);
    let last = init_value(ty, None, fields, Direction::Backward);
    let next_body = advance_struct(ty, fields, Direction::Forward);
    let previous_body = advance_struct(ty, fields, Direction::Backward);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = if generics.params.is_empty() {
        where_clause.cloned()
    } else {
        let mut clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        clause.predicates.extend(
            trait_bounds(group_type_requirements(
                fields.iter().rev().zip(tuple_type_requirements()),
            ))
            .map(WherePredicate::Type),
        );
        Some(clause)
    };
    let tokens = quote! {
        impl #impl_generics ::enum_iterator::Sequence for #ty #ty_generics #where_clause {
            #[allow(clippy::identity_op)]
            const CARDINALITY: usize = #cardinality;

            fn next(&self) -> ::core::option::Option<Self> {
                #next_body
            }

            fn previous(&self) -> ::core::option::Option<Self> {
                #previous_body
            }

            fn first() -> ::core::option::Option<Self> {
                #first
            }

            fn last() -> ::core::option::Option<Self> {
                #last
            }
        }
    };
    Ok(tokens)
}

fn derive_for_enum(
    ty: &Ident,
    generics: &Generics,
    variants: &Punctuated<Variant, Comma>,
) -> Result<TokenStream, syn::Error> {
    let cardinality = enum_cardinality(variants);
    let next_body = advance_enum(ty, variants, Direction::Forward);
    let previous_body = advance_enum(ty, variants, Direction::Backward);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = if generics.params.is_empty() {
        where_clause.cloned()
    } else {
        let mut clause = where_clause.cloned().unwrap_or_else(|| WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        });
        clause.predicates.extend(
            trait_bounds(group_type_requirements(variants.iter().flat_map(
                |variant| variant.fields.iter().rev().zip(tuple_type_requirements()),
            )))
            .map(WherePredicate::Type),
        );
        Some(clause)
    };
    let next_variant_body = next_variant(ty, variants, Direction::Forward);
    let previous_variant_body = next_variant(ty, variants, Direction::Backward);
    let (first, last) = if variants.is_empty() {
        (
            quote! { ::core::option::Option::None },
            quote! { ::core::option::Option::None },
        )
    } else {
        let last_index = variants.len() - 1;
        (
            quote! { next_variant(0) },
            quote! { previous_variant(#last_index) },
        )
    };
    let tokens = quote! {
        impl #impl_generics ::enum_iterator::Sequence for #ty #ty_generics #where_clause {
            #[allow(clippy::identity_op)]
            const CARDINALITY: usize = #cardinality;

            fn next(&self) -> ::core::option::Option<Self> {
                #next_body
            }

            fn previous(&self) -> ::core::option::Option<Self> {
                #previous_body
            }

            fn first() -> ::core::option::Option<Self> {
                #first
            }

            fn last() -> ::core::option::Option<Self> {
                #last
            }
        }

        fn next_variant #impl_generics(
            mut i: usize,
        ) -> ::core::option::Option<#ty #ty_generics> #where_clause {
            #next_variant_body
        }

        fn previous_variant #impl_generics(
            mut i: usize,
        ) -> ::core::option::Option<#ty #ty_generics> #where_clause {
            #previous_variant_body
        }
    };
    let tokens = quote! {
        const _: () = { #tokens };
    };
    Ok(tokens)
}

fn enum_cardinality(variants: &Punctuated<Variant, Comma>) -> TokenStream {
    let terms = variants
        .iter()
        .map(|variant| tuple_cardinality(&variant.fields));
    quote! {
        #((#terms) +)* 0
    }
}

fn tuple_cardinality(fields: &Fields) -> TokenStream {
    let factors = fields.iter().map(|field| {
        let ty = &field.ty;
        quote! {
            <#ty as ::enum_iterator::Sequence>::CARDINALITY
        }
    });
    quote! {
        #(#factors *)* 1
    }
}

fn field_id(field: &Field, index: usize) -> Member {
    field
        .ident
        .clone()
        .map_or_else(|| Member::from(index), Member::from)
}

fn init_value(
    ty: &Ident,
    variant: Option<&Ident>,
    fields: &Fields,
    direction: Direction,
) -> TokenStream {
    let id = variant.map_or_else(|| quote! { #ty }, |v| quote! { #ty::#v });
    if fields.is_empty() {
        quote! {
            ::core::option::Option::Some(#id {})
        }
    } else {
        let reset = direction.reset();
        let initialization =
            repeat(quote! { ::enum_iterator::Sequence::#reset() }).take(fields.len());
        let assignments = field_assignments(fields);
        let bindings = bindings().take(fields.len());
        quote! {{
            match (#(#initialization,)*) {
                (#(::core::option::Option::Some(#bindings),)*) => {
                    ::core::option::Option::Some(#id { #assignments })
                }
                _ => ::core::option::Option::None,
            }
        }}
    }
}

fn next_variant(
    ty: &Ident,
    variants: &Punctuated<Variant, Comma>,
    direction: Direction,
) -> TokenStream {
    let advance = match direction {
        Direction::Forward => {
            let last_index = variants.len().saturating_sub(1);
            quote! {
                if i >= #last_index { break ::core::option::Option::None; } else { i+= 1; }
            }
        }
        Direction::Backward => quote! {
            if i == 0 { break ::core::option::Option::None; } else { i -= 1; }
        },
    };
    let arms = variants.iter().enumerate().map(|(i, v)| {
        let id = &v.ident;
        let init = init_value(ty, Some(id), &v.fields, direction);
        quote! {
            #i => #init
        }
    });
    quote! {
        loop {
            let next = match i {
                #(#arms,)*
                _ => ::core::option::Option::None,
            };
            match next {
                ::core::option::Option::Some(_) => break next,
                ::core::option::Option::None => #advance,
            }
        }
    }
}

fn advance_struct(ty: &Ident, fields: &Fields, direction: Direction) -> TokenStream {
    let assignments = field_assignments(fields);
    let bindings = bindings().take(fields.len()).collect::<Vec<_>>();
    let tuple = advance_tuple(&bindings, direction);
    quote! {
        let #ty { #assignments } = self;
        let (#(#bindings,)*) = #tuple?;
        ::core::option::Option::Some(#ty { #assignments })
    }
}

fn advance_enum(
    ty: &Ident,
    variants: &Punctuated<Variant, Comma>,
    direction: Direction,
) -> TokenStream {
    let arms: Vec<_> = match direction {
        Direction::Forward => variants
            .iter()
            .enumerate()
            .map(|(i, variant)| advance_enum_arm(ty, direction, i, variant))
            .collect(),
        Direction::Backward => variants
            .iter()
            .enumerate()
            .rev()
            .map(|(i, variant)| advance_enum_arm(ty, direction, i, variant))
            .collect(),
    };
    quote! {
        match *self {
            #(#arms,)*
        }
    }
}

fn advance_enum_arm(ty: &Ident, direction: Direction, i: usize, variant: &Variant) -> TokenStream {
    let next = match direction {
        Direction::Forward => match i.checked_add(1) {
            Some(next_i) => quote! { next_variant(#next_i) },
            None => quote! { ::core::option::Option::None },
        },
        Direction::Backward => match i.checked_sub(1) {
            Some(prev_i) => quote! { previous_variant(#prev_i) },
            None => quote! { ::core::option::Option::None },
        },
    };
    let id = &variant.ident;
    if variant.fields.is_empty() {
        quote! {
            #ty::#id {} => #next
        }
    } else {
        let destructuring = field_bindings(&variant.fields);
        let assignments = field_assignments(&variant.fields);
        let bindings = bindings().take(variant.fields.len()).collect::<Vec<_>>();
        let tuple = advance_tuple(&bindings, direction);
        quote! {
            #ty::#id { #destructuring } => {
                let y = #tuple;
                match y {
                    ::core::option::Option::Some((#(#bindings,)*)) => {
                        ::core::option::Option::Some(#ty::#id { #assignments })
                    }
                    ::core::option::Option::None => #next,
                }
            }
        }
    }
}

fn advance_tuple(bindings: &[Ident], direction: Direction) -> TokenStream {
    let advance = direction.advance();
    let reset = direction.reset();
    let rev_bindings = bindings.iter().rev().collect::<Vec<_>>();
    let (rev_binding_head, rev_binding_tail) = match rev_bindings.split_first() {
        Some((&head, tail)) => (Some(head), tail),
        None => (None, &*rev_bindings),
    };
    let rev_binding_head = match rev_binding_head {
        Some(head) => quote! {
            let (#head, carry) = match ::enum_iterator::Sequence::#advance(#head) {
                ::core::option::Option::Some(#head) => (::core::option::Option::Some(#head), false),
                ::core::option::Option::None => (::enum_iterator::Sequence::#reset(), true),
            };
        },
        None => quote! {
            let carry = true;
        },
    };
    let body = quote! {
        #rev_binding_head
        #(
            let (#rev_binding_tail, carry) = if carry {
                match ::enum_iterator::Sequence::#advance(#rev_binding_tail) {
                    ::core::option::Option::Some(#rev_binding_tail) => {
                        (::core::option::Option::Some(#rev_binding_tail), false)
                    }
                    ::core::option::Option::None => (::enum_iterator::Sequence::#reset(), true),
                }
            } else {
                (
                    ::core::option::Option::Some(::core::clone::Clone::clone(#rev_binding_tail)),
                    false,
                )
            };
        )*
        if carry {
            ::core::option::Option::None
        } else {
            match (#(#bindings,)*) {
                (#(::core::option::Option::Some(#bindings),)*) => {
                    ::core::option::Option::Some((#(#bindings,)*))
                }
                _ => ::core::option::Option::None,
            }
        }
    };
    quote! {
        { #body }
    }
}

fn field_assignments<'a, I>(fields: I) -> TokenStream
where
    I: IntoIterator<Item = &'a Field>,
{
    fields
        .into_iter()
        .enumerate()
        .zip(bindings())
        .map(|((i, field), binding)| {
            let field_id = field_id(field, i);
            quote! { #field_id: #binding, }
        })
        .collect()
}

fn field_bindings<'a, I>(fields: I) -> TokenStream
where
    I: IntoIterator<Item = &'a Field>,
{
    fields
        .into_iter()
        .enumerate()
        .zip(bindings())
        .map(|((i, field), binding)| {
            let field_id = field_id(field, i);
            quote! { #field_id: ref #binding, }
        })
        .collect()
}

fn bindings() -> impl Iterator<Item = Ident> {
    (0..).map(|i| Ident::new(&format!("x{i}"), Span::call_site()))
}

fn trait_bounds<I>(types: I) -> impl Iterator<Item = PredicateType>
where
    I: IntoIterator<Item = (Type, TypeRequirements)>,
{
    types
        .into_iter()
        .map(|(bounded_ty, requirements)| PredicateType {
            lifetimes: None,
            bounded_ty,
            colon_token: Default::default(),
            bounds: requirements
                .into_iter()
                .map(|req| match req {
                    TypeRequirement::Clone => clone_trait_path(),
                    TypeRequirement::Sequence => trait_path(),
                })
                .map(trait_bound)
                .collect(),
        })
}

fn trait_bound(path: Path) -> TypeParamBound {
    TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path,
    })
}

fn trait_path() -> Path {
    Path {
        leading_colon: Some(Default::default()),
        segments: [
            PathSegment::from(Ident::new("enum_iterator", Span::call_site())),
            Ident::new("Sequence", Span::call_site()).into(),
        ]
        .into_iter()
        .collect(),
    }
}

fn clone_trait_path() -> Path {
    Path {
        leading_colon: Some(Default::default()),
        segments: [
            PathSegment::from(Ident::new("core", Span::call_site())),
            Ident::new("clone", Span::call_site()).into(),
            Ident::new("Clone", Span::call_site()).into(),
        ]
        .into_iter()
        .collect(),
    }
}

fn tuple_type_requirements() -> impl Iterator<Item = TypeRequirements> {
    once([TypeRequirement::Sequence].into()).chain(repeat(
        [TypeRequirement::Sequence, TypeRequirement::Clone].into(),
    ))
}

fn group_type_requirements<'a, I>(bounds: I) -> Vec<(Type, TypeRequirements)>
where
    I: IntoIterator<Item = (&'a Field, TypeRequirements)>,
{
    bounds
        .into_iter()
        .fold(
            (HashMap::<_, usize>::new(), Vec::new()),
            |(mut indexes, mut acc), (field, requirements)| {
                let i = *indexes.entry(field.ty.clone()).or_insert_with(|| {
                    acc.push((field.ty.clone(), TypeRequirements::new()));
                    acc.len() - 1
                });
                acc[i].1.extend(requirements);
                (indexes, acc)
            },
        )
        .1
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TypeRequirement {
    Sequence,
    Clone,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TypeRequirements(u8);

impl TypeRequirements {
    const SEQUENCE: u8 = 0x1;
    const CLONE: u8 = 0x2;

    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, req: TypeRequirement) {
        self.0 |= Self::enum_to_mask(req);
    }

    fn into_iter(self) -> impl Iterator<Item = TypeRequirement> {
        let mut n = self.0;
        iter::from_fn(move || {
            if n & Self::SEQUENCE != 0 {
                n &= !Self::SEQUENCE;
                Some(TypeRequirement::Sequence)
            } else if n & Self::CLONE != 0 {
                n &= !Self::CLONE;
                Some(TypeRequirement::Clone)
            } else {
                None
            }
        })
    }

    fn extend(&mut self, other: Self) {
        self.0 |= other.0;
    }

    fn enum_to_mask(req: TypeRequirement) -> u8 {
        match req {
            TypeRequirement::Sequence => Self::SEQUENCE,
            TypeRequirement::Clone => Self::CLONE,
        }
    }
}

impl<const N: usize> From<[TypeRequirement; N]> for TypeRequirements {
    fn from(reqs: [TypeRequirement; N]) -> Self {
        reqs.into_iter()
            .fold(TypeRequirements::new(), |mut acc, req| {
                acc.insert(req);
                acc
            })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Forward,
    Backward,
}

impl Direction {
    fn advance(self) -> Ident {
        let s = match self {
            Direction::Forward => "next",
            Direction::Backward => "previous",
        };
        Ident::new(s, Span::call_site())
    }

    fn reset(self) -> Ident {
        let s = match self {
            Direction::Forward => "first",
            Direction::Backward => "last",
        };
        Ident::new(s, Span::call_site())
    }
}

#[derive(Debug)]
enum Error {
    UnsupportedUnion,
}

impl Error {
    fn with_tokens<T: ToTokens>(self, tokens: T) -> syn::Error {
        syn::Error::new_spanned(tokens, self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnsupportedUnion => f.write_str("Sequence cannot be derived for union types"),
        }
    }
}
