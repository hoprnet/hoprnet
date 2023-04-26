#![cfg_attr(test, deny(warnings))]

pub mod helpers;

use crate::helpers::{ ResultExt, test_err };
use asn1_der::{ DerObject, der };


#[test]
fn length() {
	for test in test_err::load().length {
		der::length::decode(&mut test.bytes.iter()).assert_err(&test.err, &test.name);
	}
}


#[test]
fn object() {
	for test in test_err::load().object {
		DerObject::decode(test.bytes.as_slice()).assert_err(test.err(), &test.name);
	}
}