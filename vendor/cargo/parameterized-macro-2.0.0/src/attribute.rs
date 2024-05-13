use quote::quote;
use std::fmt::Formatter;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::{Async, Const, Unsafe};
use syn::{braced, Attribute, Block, ItemFn, Meta, ReturnType, Visibility};

/// An ordered list of attribute arguments, which consists of (id, param-args) pairs.
#[derive(Clone)]
pub struct ParameterizedList {
    pub args: Punctuated<ParameterList, Token![,]>,
}

impl Parse for ParameterizedList {
    /// This part parses
    /// It uses IdentifiedArgList.parse() for each inner argument.
    ///
    /// ['IdentifiedArgList.parse ']: struct.IdentifiedArgList
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ParameterizedList {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

/// A single (id, param-args) pair which consists of:
///   - id: identifier for the list
///   - param_args: ordered list arguments formatted using curly-braced list syntax, i.e. "{ 3, 4, 5 }"
///
/// For example:
/// `parameter_name = { 3, 4, 5}`
#[derive(Clone)]
pub struct ParameterList {
    pub id: syn::Ident,
    _assignment: Token![=],
    _braces: syn::token::Brace,
    pub param_args: Punctuated<syn::Expr, Token![,]>,
}

impl std::fmt::Debug for ParameterList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("ParameterList(id = {:?})", self.id))
    }
}

impl Parse for ParameterList {
    // parts:
    //
    // v = { a, b, c }
    // $ident $Token![=] ${ $expr, ... }
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(ParameterList {
            id: input.parse()?,
            _assignment: input.parse()?,
            _braces: braced!(content in input),
            param_args: Punctuated::parse_terminated(&content)?,
        })
    }
}

// TODO: add to parse, code gen of ParameterizedList
pub enum MacroAttribute {
    /// A `#[parameterized_macro(..)]` attribute
    ///
    /// Example usage: `#[parameterized_macro(tokio::test)]`
    UseTestMacro(Meta),
    /// An attribute unrelated to this crate; to be retained after the generation step
    Unrelated(Attribute),
}

impl MacroAttribute {
    pub fn is_use_test_macro(&self) -> bool {
        matches!(self, Self::UseTestMacro(_))
    }

    pub fn quoted(&self) -> proc_macro2::TokenStream {
        match self {
            Self::UseTestMacro(meta) => quote!(#meta),
            Self::Unrelated(attr) => quote!(#attr),
        }
    }
}

pub struct Fn {
    pub attrs: Vec<MacroAttribute>,
    pub item_fn: ItemFn,
}

impl Parse for Fn {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input
            .call(Attribute::parse_outer)?
            .into_iter()
            .map(|attr| {
                if attr.path().is_ident("parameterized_macro") {
                    attr.parse_args::<Meta>().map(MacroAttribute::UseTestMacro)
                } else {
                    Ok(MacroAttribute::Unrelated(attr))
                }
            })
            .collect::<Result<Vec<MacroAttribute>>>()?;

        Ok(Self {
            attrs,
            item_fn: input.parse()?,
        })
    }
}

impl Fn {
    pub fn constness(&self) -> Option<&Const> {
        self.item_fn.sig.constness.as_ref()
    }

    pub fn asyncness(&self) -> Option<&Async> {
        self.item_fn.sig.asyncness.as_ref()
    }

    pub fn unsafety(&self) -> Option<&Unsafe> {
        self.item_fn.sig.unsafety.as_ref()
    }

    pub fn visibility(&self) -> &Visibility {
        &self.item_fn.vis
    }

    pub fn return_type(&self) -> &ReturnType {
        &self.item_fn.sig.output
    }

    pub fn body(&self) -> &Block {
        &self.item_fn.block
    }
}
