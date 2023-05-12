// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
use crate::ParamType;

/// Output formatter for param type.
pub struct Writer;

impl Writer {
	/// Returns string which is a formatted represenation of param.
	pub fn write(param: &ParamType) -> String {
		Writer::write_for_abi(param, true)
	}

	/// If `serialize_tuple_contents` is `true`, tuples will be represented
	/// as list of inner types in parens, for example `(int256,bool)`.
	/// If it is `false`, tuples will be represented as keyword `tuple`.
	pub fn write_for_abi(param: &ParamType, serialize_tuple_contents: bool) -> String {
		match *param {
			ParamType::Address => "address".to_owned(),
			ParamType::Bytes => "bytes".to_owned(),
			ParamType::FixedBytes(len) => format!("bytes{len}"),
			ParamType::Int(len) => format!("int{len}"),
			ParamType::Uint(len) => format!("uint{len}"),
			ParamType::Bool => "bool".to_owned(),
			ParamType::String => "string".to_owned(),
			ParamType::FixedArray(ref param, len) => {
				format!("{}[{len}]", Writer::write_for_abi(param, serialize_tuple_contents))
			}
			ParamType::Array(ref param) => {
				format!("{}[]", Writer::write_for_abi(param, serialize_tuple_contents))
			}
			ParamType::Tuple(ref params) => {
				if serialize_tuple_contents {
					let formatted = params
						.iter()
						.map(|t| Writer::write_for_abi(t, serialize_tuple_contents))
						.collect::<Vec<String>>()
						.join(",");
					format!("({formatted})")
				} else {
					"tuple".to_owned()
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Writer;
	#[cfg(not(feature = "std"))]
	use crate::no_std_prelude::*;
	use crate::ParamType;

	#[test]
	fn test_write_param() {
		assert_eq!(Writer::write(&ParamType::Address), "address".to_owned());
		assert_eq!(Writer::write(&ParamType::Bytes), "bytes".to_owned());
		assert_eq!(Writer::write(&ParamType::FixedBytes(32)), "bytes32".to_owned());
		assert_eq!(Writer::write(&ParamType::Uint(256)), "uint256".to_owned());
		assert_eq!(Writer::write(&ParamType::Int(64)), "int64".to_owned());
		assert_eq!(Writer::write(&ParamType::Bool), "bool".to_owned());
		assert_eq!(Writer::write(&ParamType::String), "string".to_owned());
		assert_eq!(Writer::write(&ParamType::Array(Box::new(ParamType::Bool))), "bool[]".to_owned());
		assert_eq!(Writer::write(&ParamType::FixedArray(Box::new(ParamType::String), 2)), "string[2]".to_owned());
		assert_eq!(
			Writer::write(&ParamType::FixedArray(Box::new(ParamType::Array(Box::new(ParamType::Bool))), 2)),
			"bool[][2]".to_owned()
		);
		assert_eq!(
			Writer::write(&ParamType::Array(Box::new(ParamType::Tuple(vec![
				ParamType::Array(Box::new(ParamType::Tuple(vec![ParamType::Int(256), ParamType::Uint(256)]))),
				ParamType::FixedBytes(32),
			])))),
			"((int256,uint256)[],bytes32)[]".to_owned()
		);

		assert_eq!(
			Writer::write_for_abi(
				&ParamType::Array(Box::new(ParamType::Tuple(vec![
					ParamType::Array(Box::new(ParamType::Int(256))),
					ParamType::FixedBytes(32),
				]))),
				false
			),
			"tuple[]".to_owned()
		);
	}
}
