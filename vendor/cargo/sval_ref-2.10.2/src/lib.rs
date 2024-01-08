/*!
A variant of [`sval::Value`] for types that store references internally.
*/

#![cfg_attr(not(test), no_std)]
#![deny(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate core;

#[cfg(all(feature = "alloc", not(feature = "std")))]
mod std {
    pub use crate::{
        alloc::{borrow, boxed, collections, string, vec},
        core::{convert, fmt, hash, marker, mem, ops, result, str, write},
    };
}

#[cfg(all(not(feature = "alloc"), not(feature = "std")))]
extern crate core as std;

mod seq;

/**
Stream a value through a stream.
*/
pub fn stream_ref<'sval>(
    stream: &mut (impl Stream<'sval> + ?Sized),
    value: impl ValueRef<'sval>,
) -> Result {
    value.stream_ref(stream)
}

use sval::{Result, Stream, Value};

/**
A producer of structured data that stores a reference internally.

This trait is a variant of [`Value`] for wrapper types that keep a reference to a value internally.
In `Value`, the `'sval` lifetime comes from the borrow of `&'sval self`. In `ValueRef`, it comes
from the `'sval` lifetime in the trait itself.
*/
pub trait ValueRef<'sval>: Value {
    /**
    Stream this value through a [`Stream`].
    */
    fn stream_ref<S: Stream<'sval> + ?Sized>(&self, stream: &mut S) -> Result;
}

macro_rules! impl_value_ref_forward {
    ({ $($r:tt)* } => $bind:ident => { $($forward:tt)* }) => {
        $($r)* {
            fn stream_ref<S: Stream<'sval> + ?Sized>(&self, stream: &mut S) -> Result {
                let $bind = self;
                ($($forward)*).stream_ref(stream)
            }
        }
    };
}

impl_value_ref_forward!({impl<'sval, 'a, T: ValueRef<'sval> + ?Sized> ValueRef<'sval> for &'a T} => x => { **x });

#[cfg(feature = "alloc")]
mod alloc_support {
    use super::*;

    use crate::std::boxed::Box;

    impl_value_ref_forward!({impl<'sval, T: ValueRef<'sval> + ?Sized> ValueRef<'sval> for Box<T>} => x => { **x });
}

#[cfg(test)]
mod test {
    use crate::ValueRef;
    pub(crate) use sval_test::{assert_tokens, Token};

    pub(crate) fn assert_tokens_ref<'sval>(
        value: impl ValueRef<'sval>,
        tokens: &[sval_test::Token<'sval>],
    ) {
        let mut actual = sval_test::TokenBuf::new();
        value.stream_ref(&mut actual).unwrap();

        assert_eq!(tokens, actual.as_tokens());
    }

    pub(crate) struct Ref<T>(pub(crate) T);

    impl<T: sval::Value> sval::Value for Ref<T> {
        fn stream<'sval, S: sval::Stream<'sval> + ?Sized>(
            &'sval self,
            stream: &mut S,
        ) -> sval::Result {
            self.0.stream(stream)
        }
    }

    impl<'sval, T: sval::Value + ?Sized> ValueRef<'sval> for Ref<&'sval T> {
        fn stream_ref<S: sval::Stream<'sval> + ?Sized>(&self, stream: &mut S) -> sval::Result {
            self.0.stream(stream)
        }
    }

    pub(crate) fn compat_case<'sval>(
        v: &'sval (impl sval::Value + ValueRef<'sval> + ?Sized),
        tokens: &[Token<'sval>],
    ) {
        assert_tokens_ref(v, tokens);
        assert_tokens(v, tokens);
    }
}
