//! Internal attributes of the form `#[auto_impl(name(...))]` that can be
//! attached to trait items.

use proc_macro2::{Delimiter, TokenTree};
use proc_macro_error::{abort, emit_error};
use syn::{
    spanned::Spanned,
    visit_mut::{visit_item_trait_mut, VisitMut},
    Attribute, TraitItem,
};

use crate::proxy::{parse_types, ProxyType};

/// Removes all `#[auto_impl]` attributes that are attached to methods of the
/// given trait.
pub(crate) fn remove_our_attrs(trait_def: &mut syn::ItemTrait) {
    struct AttrRemover;
    impl VisitMut for AttrRemover {
        fn visit_trait_item_mut(&mut self, item: &mut TraitItem) {
            let item_span = item.span();
            let (attrs, is_method) = match item {
                TraitItem::Method(m) => (&mut m.attrs, true),
                TraitItem::Const(c) => (&mut c.attrs, false),
                TraitItem::Type(t) => (&mut t.attrs, false),
                TraitItem::Macro(m) => (&mut m.attrs, false),
                _ => abort!(
                    item.span(),
                    "encountered unexpected `TraitItem`, cannot handle that, sorry!";
                    note = "auto-impl supports only methods, consts, types and macros currently";
                ),
            };

            // Make sure non-methods do not have our attributes.
            if !is_method && attrs.iter().any(|a| is_our_attr(a)) {
                emit_error!(
                    item_span,
                    "`#[auto_impl]` attributes are only allowed on methods",
                );
            }

            attrs.retain(|a| !is_our_attr(a));
        }
    }

    visit_item_trait_mut(&mut AttrRemover, trait_def);
}

/// Checks if the given attribute is "our" attribute. That means that it's path
/// is `auto_impl`.
pub(crate) fn is_our_attr(attr: &Attribute) -> bool {
    attr.path.is_ident("auto_impl")
}

/// Tries to parse the given attribute as one of our own `auto_impl`
/// attributes. If it's invalid, an error is emitted and `Err(())` is returned.
/// You have to make sure that `attr` is one of our attrs with `is_our_attr`
/// before calling this function!
pub(crate) fn parse_our_attr(attr: &Attribute) -> Result<OurAttr, ()> {
    assert!(is_our_attr(attr));

    // Get the body of the attribute (which has to be a ground, because we
    // required the syntax `auto_impl(...)` and forbid stuff like
    // `auto_impl = ...`).
    let tokens = attr.tokens.clone().into_iter().collect::<Vec<_>>();
    let body = match &*tokens {
        [TokenTree::Group(g)] => g.stream(),
        _ => {
            emit_error!(
                attr.tokens.span(),
                "expected single group delimited by `()`, found '{:?}'",
                tokens,
            );
            return Err(());
        }
    };

    let mut it = body.clone().into_iter();

    // Try to extract the name (we require the body to be `name(...)`).
    let name = match it.next() {
        Some(TokenTree::Ident(x)) => x,
        Some(other) => {
            emit_error!(other.span(), "expected ident, found '{}'", other);
            return Err(());
        }
        None => {
            emit_error!(attr.tokens.span(), "expected ident, found nothing");
            return Err(());
        }
    };

    // Extract the parameters (which again, have to be a group delimited by
    // `()`)
    let params = match it.next() {
        Some(TokenTree::Group(ref g)) if g.delimiter() == Delimiter::Parenthesis => g.stream(),
        Some(other) => {
            emit_error!(
                other.span(),
                "expected arguments for '{}' in parenthesis `()`, found `{}`",
                name,
                other,
            );
            return Err(());
        }
        None => {
            emit_error!(
                body.span(),
                "expected arguments for '{}' in parenthesis `()`, found nothing",
                name,
            );
            return Err(());
        }
    };

    // Finally match over the name of the attribute.
    let out = if name == "keep_default_for" {
        let proxy_types = parse_types(params.into());
        OurAttr::KeepDefaultFor(proxy_types)
    } else {
        emit_error!(
            name.span(), "invalid attribute '{}'", name;
            note = "only `keep_default_for` is supported";
        );
        return Err(());
    };

    Ok(out)
}

/// Attributes of the form `#[auto_impl(...)]` that can be attached to items of
/// the trait.
#[derive(Clone, PartialEq, Debug)]
pub(crate) enum OurAttr {
    KeepDefaultFor(Vec<ProxyType>),
}
