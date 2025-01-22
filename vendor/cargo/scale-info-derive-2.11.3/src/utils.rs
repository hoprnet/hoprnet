// Copyright 2019-2022 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Utility methods to work with `SCALE` attributes relevant for the `TypeInfo` derive..
//!
//! NOTE: The code here is copied verbatim from `parity-scale-codec-derive`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, spanned::Spanned, AttrStyle, Attribute, Lit, Meta, NestedMeta, Variant};

/// Look for a `#[codec(index = $int)]` attribute on a variant. If no attribute
/// is found, fall back to the discriminant or just the variant index.
pub fn variant_index(v: &Variant, i: usize) -> TokenStream {
    // first look for an `index` attribute…
    let index = maybe_index(v);
    // …then fallback to discriminant or just index
    index.map(|i| quote! { #i }).unwrap_or_else(|| {
        v.discriminant
            .as_ref()
            .map(|(_, ref expr)| quote! { #expr })
            .unwrap_or_else(|| quote! { #i })
    })
}

/// Look for a `#[codec(index = $int)]` outer attribute on a variant.
/// If found, it is expected to be a parseable as a `u8` (panics otherwise).
pub fn maybe_index(variant: &Variant) -> Option<u8> {
    let outer_attrs = variant
        .attrs
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer);

    codec_meta_item(outer_attrs, |meta| {
        if let NestedMeta::Meta(Meta::NameValue(ref nv)) = meta {
            if nv.path.is_ident("index") {
                if let Lit::Int(ref v) = nv.lit {
                    let byte = v
                        .base10_parse::<u8>()
                        .expect("Internal error. `#[codec(index = …)]` attribute syntax must be checked in `parity-scale-codec`. This is a bug.");
                    return Some(byte);
                }
            }
        }

        None
    })
}

/// Look for a `#[codec(compact)]` outer attribute on the given `Field`.
pub fn is_compact(field: &syn::Field) -> bool {
    let outer_attrs = field
        .attrs
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer);
    codec_meta_item(outer_attrs, |meta| {
        if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
            if path.is_ident("compact") {
                return Some(());
            }
        }

        None
    })
    .is_some()
}

/// Look for a `#[codec(skip)]` in the given attributes.
pub fn should_skip(attrs: &[Attribute]) -> bool {
    codec_meta_item(attrs.iter(), |meta| {
        if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
            if path.is_ident("skip") {
                return Some(path.span());
            }
        }

        None
    })
    .is_some()
}

fn codec_meta_item<'a, F, R, I, M>(itr: I, pred: F) -> Option<R>
where
    F: FnMut(M) -> Option<R> + Clone,
    I: Iterator<Item = &'a Attribute>,
    M: Parse,
{
    find_meta_item("codec", itr, pred)
}

fn find_meta_item<'a, F, R, I, M>(kind: &str, mut itr: I, mut pred: F) -> Option<R>
where
    F: FnMut(M) -> Option<R> + Clone,
    I: Iterator<Item = &'a Attribute>,
    M: Parse,
{
    itr.find_map(|attr| {
        attr.path
            .is_ident(kind)
            .then(|| pred(attr.parse_args().ok()?))
            .flatten()
    })
}
