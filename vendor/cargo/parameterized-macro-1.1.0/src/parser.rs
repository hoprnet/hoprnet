use std::fmt::Formatter;
use syn::braced;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;

/// An ordered list of attribute arguments, which consists of (id, param-args) pairs.
#[derive(Clone)]
pub(crate) struct AttributeArgList {
    pub(crate) args: Punctuated<IdentifiedArgList, Token![,]>,
}

impl Parse for AttributeArgList {
    /// This part parses
    /// It uses IdentifiedArgList.parse() for each inner argument.
    ///
    /// ['IdentifiedArgList.parse ']: struct.IdentifiedArgList
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(AttributeArgList {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

/// A single (id, param-args) pair which consists of:
///   id: identifier for the list
///   param_args: ordered list arguments formatted using curly-braced list syntax, i.e. "{ 3, 4, 5 }"
#[derive(Clone)]
pub(crate) struct IdentifiedArgList {
    pub(crate) id: syn::Ident,
    _assignment: Token![=],
    _braces: syn::token::Brace,
    pub(crate) param_args: Punctuated<syn::Expr, Token![,]>,
}

impl std::fmt::Debug for IdentifiedArgList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("IdentifiedArgList(id = {:?})", self.id))
    }
}

impl Parse for IdentifiedArgList {
    // parts:
    //
    // v = { a, b, c }
    // $ident $Token![=] ${ $expr, ... }
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(IdentifiedArgList {
            id: input.parse()?,
            _assignment: input.parse()?,
            _braces: braced!(content in input),
            param_args: Punctuated::parse_terminated(&content)?,
        })
    }
}
