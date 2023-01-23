use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    spanned::Spanned,
    Ident, LitByteStr, LitInt, LitStr, Visibility,
};

const DEFAULT_LIMIT: usize = 65536;

pub enum Regex {
    LitStr(LitStr),
    LitByteStr(LitByteStr),
}

impl Regex {
    fn is_str(&self) -> bool {
        match self {
            Regex::LitStr(_) => true,
            Regex::LitByteStr(_) => false,
        }
    }

    fn get_regex(&self) -> String {
        match self {
            Regex::LitStr(lit_str) => lit_str.value(),
            Regex::LitByteStr(lit_byte_str) => {
                let bytes = lit_byte_str.value();
                String::from_utf8(bytes).unwrap()
            }
        }
    }
}

impl Parse for Regex {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let lookahead = input.lookahead1();
        let result = if lookahead.peek(LitStr) {
            Regex::LitStr(input.parse()?)
        } else {
            Regex::LitByteStr(input.parse()?)
        };
        Ok(result)
    }
}

impl Spanned for Regex {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Regex::LitStr(lit_str) => lit_str.span(),
            Regex::LitByteStr(lit_byte_str) => lit_byte_str.span(),
        }
    }
}

pub struct MacroInput {
    visibility: Visibility,
    name: Ident,
    regex: Regex,
    threshold: usize,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let visibility: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        let regex = input.parse()?;
        let lookahead = input.lookahead1();
        let threshold = if lookahead.peek(LitInt) {
            let threshold: LitInt = input.parse()?;
            threshold.base10_parse()?
        } else {
            DEFAULT_LIMIT
        };
        Ok(MacroInput {
            visibility,
            name,
            regex,
            threshold,
        })
    }
}

impl MacroInput {
    pub fn is_str(&self) -> bool {
        self.regex.is_str()
    }

    pub fn get_regex(&self) -> String {
        self.regex.get_regex()
    }

    pub fn get_regex_span(&self) -> Span {
        self.regex.span()
    }

    pub fn get_name(&self) -> &Ident {
        &self.name
    }

    pub fn get_visibility(&self) -> &Visibility {
        &self.visibility
    }

    pub fn get_threshold(&self) -> usize {
        self.threshold
    }
}
