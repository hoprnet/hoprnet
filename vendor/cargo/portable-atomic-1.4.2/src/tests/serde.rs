use crate::*;

use serde_test::{assert_tokens, Token};
use test_helper::serde::DebugPartialEq;

macro_rules! t {
    ($atomic_type:ty, $value_type:ident, $token_type:ident) => {
        assert_tokens(&DebugPartialEq(<$atomic_type>::new($value_type::MAX)), &[
            Token::$token_type($value_type::MAX as _),
        ]);
        assert_tokens(&DebugPartialEq(<$atomic_type>::new($value_type::MIN)), &[
            Token::$token_type($value_type::MIN as _),
        ]);
    };
}

#[test]
fn test() {
    assert_tokens(&DebugPartialEq(AtomicBool::new(true)), &[Token::Bool(true)]);
    assert_tokens(&DebugPartialEq(AtomicBool::new(false)), &[Token::Bool(false)]);
    t!(AtomicIsize, isize, I64);
    t!(AtomicUsize, usize, U64);
    t!(AtomicI8, i8, I8);
    t!(AtomicU8, u8, U8);
    t!(AtomicI16, i16, I16);
    t!(AtomicU16, u16, U16);
    t!(AtomicI32, i32, I32);
    t!(AtomicU32, u32, U32);
    t!(AtomicI64, i64, I64);
    t!(AtomicU64, u64, U64);
    // TODO: serde_test doesn't support Token::{I128,U128}
    // t!(AtomicI128, i128, I128);
    // t!(AtomicU128, u128, U128);
    #[cfg(feature = "float")]
    t!(AtomicF32, f32, F32);
    #[cfg(feature = "float")]
    t!(AtomicF64, f64, F64);
}
