use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    LitStr, Token,
};

use crate::Array;

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SecurityRequirementsAttrItem {
    pub name: Option<String>,
    pub scopes: Option<Vec<String>>,
}

#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SecurityRequirementsAttr(Punctuated<SecurityRequirementsAttrItem, Comma>);

impl Parse for SecurityRequirementsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Punctuated::<SecurityRequirementsAttrItem, Comma>::parse_terminated(input)
            .map(|o| Self(o.into_iter().collect()))
    }
}

impl Parse for SecurityRequirementsAttrItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<LitStr>()?.value();

        input.parse::<Token![=]>()?;

        let scopes_stream;
        bracketed!(scopes_stream in input);

        let scopes = Punctuated::<LitStr, Comma>::parse_terminated(&scopes_stream)?
            .iter()
            .map(LitStr::value)
            .collect::<Vec<_>>();

        Ok(Self {
            name: Some(name),
            scopes: Some(scopes),
        })
    }
}

impl ToTokens for SecurityRequirementsAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            utoipa::openapi::security::SecurityRequirement::default()
        });

        for requirement in &self.0 {
            if let (Some(name), Some(scopes)) = (&requirement.name, &requirement.scopes) {
                let scopes = scopes.iter().collect::<Array<&String>>();
                let scopes_len = scopes.len();

                tokens.extend(quote! {
                    .add::<&str, [&str; #scopes_len], &str>(#name, #scopes)
                });
            }
        }
    }
}
