#![cfg_attr(test, deny(warnings))]
#![cfg(feature = "native_types")]

pub mod helpers;

use crate::helpers::{test_err, ResultExt};
use asn1_der::typed::{Boolean, DerDecodable, Integer, Null, OctetString, Sequence, Utf8String};

#[test]
fn boolean() {
    for test in test_err::load().typed.bool {
        Boolean::decode(&test.bytes).assert_err(&test.err, &test.name);
        bool::decode(&test.bytes).assert_err(&test.err, &test.name);
    }
}

#[test]
fn integer() {
    for test in test_err::load().typed.integer {
        Integer::decode(&test.bytes).assert_err(&test.err, &test.name);
        macro_rules! native {
			($num:ty) => (<$num>::decode(&test.bytes).assert_err(&test.err, &test.name));
			($( $num:ty ),+) => ($( native!($num); )+);
		}
        native!(u8, u16, u32, u64, u128, usize);
    }
}

#[test]
fn null() {
    for test in test_err::load().typed.null {
        type OptBool = Option<bool>;
        Null::decode(&test.bytes).assert_err(&test.err, &test.name);
        OptBool::decode(&test.bytes).assert_err(&test.err, &test.name);
    }
}

#[test]
fn octet_string() {
    for test in test_err::load().typed.octet_string {
        OctetString::decode(&test.bytes).assert_err(&test.err, &test.name);
        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        Vec::<u8>::decode(&test.bytes).assert_err(&test.err, &test.name);
    }
}

#[test]
fn sequence() {
    for test in test_err::load().typed.sequence {
        Sequence::decode(&test.bytes).assert_err(&test.err, &test.name);
    }
}

#[test]
fn utf8_string() {
    for test in test_err::load().typed.utf8_string {
        Utf8String::decode(&test.bytes).assert_err(&test.err, &test.name);
        #[cfg(all(feature = "std", not(feature = "no_panic")))]
        String::decode(&test.bytes).assert_err(&test.err, &test.name);
    }
}
