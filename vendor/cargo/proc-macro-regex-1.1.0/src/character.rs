use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex_syntax::hir::{Class, ClassBytes, ClassUnicode, Literal};
use std::collections::BTreeSet;
use thiserror::Error;

fn to_byte(c: char) -> CharacterResult<u8> {
    if c.len_utf8() == 1 {
        let mut bytes = [0; 1];
        c.encode_utf8(&mut bytes);
        Ok(bytes[0])
    } else {
        Err(CharacterError::Unicode(c))
    }
}

fn to_char(b: u8) -> CharacterResult<char> {
    match char::try_from(b) {
        Ok(c) => Ok(c),
        Err(_) => Err(CharacterError::Byte(b)),
    }
}

#[derive(Debug, Error)]
pub enum CharacterError {
    #[error("got byte: {0}")]
    Byte(u8),
    #[error("got class bytes: {0:?}")]
    ClassBytes(ClassBytes),
    #[error("got unicode: {0}")]
    Unicode(char),
    #[error("got class unicode: {0:?}")]
    ClassUnicode(ClassUnicode),
}

pub type CharacterResult<T> = Result<T, CharacterError>;

pub trait Character: Sized + Ord + TryFrom<u32> + Into<u32> {
    fn new_line() -> Self;

    fn from_literal(literal: Literal) -> CharacterResult<Self>;

    fn from_class(class: Class) -> CharacterResult<BTreeSet<Self>>;

    fn to_byte(&self) -> Option<u8>;

    fn is_byte(&self) -> bool;

    fn is_next(&self, other: &Self) -> bool;

    fn get_iterator_function(is_byte: bool) -> Ident;

    fn to_usize(element: Ident, is_byte: bool) -> TokenStream;

    fn allow_invalid_utf8() -> bool;

    fn unicode() -> bool;
}

impl Character for char {
    fn new_line() -> Self {
        '\n'
    }

    fn from_literal(literal: Literal) -> CharacterResult<Self> {
        match literal {
            Literal::Unicode(c) => Ok(c),
            Literal::Byte(b) => to_char(b),
        }
    }

    fn from_class(class: Class) -> CharacterResult<BTreeSet<Self>> {
        let mut cs = BTreeSet::new();
        match class {
            Class::Unicode(class_unicode) => {
                for class_unicode_range in class_unicode.iter() {
                    let start = class_unicode_range.start();
                    let end = class_unicode_range.end();
                    for c in start..=end {
                        cs.insert(c);
                    }
                }
            }
            Class::Bytes(class_bytes) => {
                for class_bytes_range in class_bytes.iter() {
                    let start = class_bytes_range.start();
                    let end = class_bytes_range.end();
                    for b in start..=end {
                        let c = to_char(b)?;
                        cs.insert(c);
                    }
                }
            }
        }
        Ok(cs)
    }

    fn to_byte(&self) -> Option<u8> {
        to_byte(*self).ok()
    }

    fn is_byte(&self) -> bool {
        self.len_utf8() == 1
    }

    fn is_next(&self, other: &Self) -> bool {
        let self_u32: u32 = *self as u32;
        if let Some(next) = self_u32.checked_add(1) {
            let other_u32 = *other as u32;
            next == other_u32
        } else {
            false
        }
    }

    fn get_iterator_function(is_byte: bool) -> Ident {
        if is_byte {
            Ident::new("bytes", Span::call_site())
        } else {
            Ident::new("chars", Span::call_site())
        }
    }

    fn to_usize(element: Ident, _is_byte: bool) -> TokenStream {
        quote! {
            #element as usize
        }
    }

    fn allow_invalid_utf8() -> bool {
        false
    }

    fn unicode() -> bool {
        true
    }
}

impl Character for u8 {
    fn new_line() -> Self {
        b'\n'
    }

    fn from_literal(literal: Literal) -> CharacterResult<Self> {
        match literal {
            Literal::Unicode(c) => to_byte(c),
            Literal::Byte(b) => Ok(b),
        }
    }

    fn from_class(class: Class) -> CharacterResult<BTreeSet<Self>> {
        let mut bs = BTreeSet::new();
        match class {
            Class::Unicode(class_unicode) => {
                for class_unicode_range in class_unicode.iter() {
                    let start = class_unicode_range.start();
                    let end = class_unicode_range.end();
                    for c in start..=end {
                        let b = to_byte(c)?;
                        bs.insert(b);
                    }
                }
            }
            Class::Bytes(class_bytes) => {
                for class_bytes_range in class_bytes.iter() {
                    let start = class_bytes_range.start();
                    let end = class_bytes_range.end();
                    for b in start..=end {
                        bs.insert(b);
                    }
                }
            }
        }
        Ok(bs)
    }

    fn to_byte(&self) -> Option<u8> {
        Some(*self)
    }

    fn is_byte(&self) -> bool {
        true
    }

    fn is_next(&self, other: &u8) -> bool {
        if let Some(next) = other.checked_add(1) {
            next == *other
        } else {
            false
        }
    }

    fn get_iterator_function(_is_byte: bool) -> Ident {
        Ident::new("into_iter", Span::call_site())
    }

    fn to_usize(element: Ident, _is_byte: bool) -> TokenStream {
        quote! {
            *#element as usize
        }
    }

    fn allow_invalid_utf8() -> bool {
        true
    }

    fn unicode() -> bool {
        false
    }
}
