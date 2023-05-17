// Copyright 2015-2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::ops;

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
use crate::{Hash, Token};

/// Raw topic filter.
#[derive(Debug, PartialEq, Default)]
pub struct RawTopicFilter {
	/// Topic.
	pub topic0: Topic<Token>,
	/// Topic.
	pub topic1: Topic<Token>,
	/// Topic.
	pub topic2: Topic<Token>,
}

/// Topic filter.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct TopicFilter {
	/// Usually (for not-anonymous transactions) the first topic is event signature.
	pub topic0: Topic<Hash>,
	/// Second topic.
	pub topic1: Topic<Hash>,
	/// Third topic.
	pub topic2: Topic<Hash>,
	/// Fourth topic.
	pub topic3: Topic<Hash>,
}

#[cfg(feature = "serde")]
impl Serialize for TopicFilter {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		vec![&self.topic0, &self.topic1, &self.topic2, &self.topic3].serialize(serializer)
	}
}

/// Acceptable topic possibilities.
#[derive(Debug, PartialEq, Eq)]
pub enum Topic<T> {
	/// Match any.
	Any,
	/// Match any of the hashes.
	OneOf(Vec<T>),
	/// Match only this hash.
	This(T),
}

impl<T> Topic<T> {
	/// Map
	pub fn map<F, O>(self, f: F) -> Topic<O>
	where
		F: Fn(T) -> O,
	{
		match self {
			Topic::Any => Topic::Any,
			Topic::OneOf(topics) => Topic::OneOf(topics.into_iter().map(f).collect()),
			Topic::This(topic) => Topic::This(f(topic)),
		}
	}

	/// Returns true if topic is empty (Topic::Any)
	pub fn is_any(&self) -> bool {
		match *self {
			Topic::Any => true,
			Topic::This(_) | Topic::OneOf(_) => false,
		}
	}
}

impl<T> Default for Topic<T> {
	fn default() -> Self {
		Topic::Any
	}
}

impl<T> From<Option<T>> for Topic<T> {
	fn from(o: Option<T>) -> Self {
		match o {
			Some(topic) => Topic::This(topic),
			None => Topic::Any,
		}
	}
}

impl<T> From<T> for Topic<T> {
	fn from(topic: T) -> Self {
		Topic::This(topic)
	}
}

impl<T> From<Vec<T>> for Topic<T> {
	fn from(topics: Vec<T>) -> Self {
		Topic::OneOf(topics)
	}
}

impl<T> From<Topic<T>> for Vec<T> {
	fn from(topic: Topic<T>) -> Self {
		match topic {
			Topic::Any => vec![],
			Topic::This(topic) => vec![topic],
			Topic::OneOf(topics) => topics,
		}
	}
}

#[cfg(feature = "serde")]
impl Serialize for Topic<Hash> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match *self {
			Topic::Any => Option::<()>::None.serialize(serializer),
			Topic::OneOf(ref vec) => vec.serialize(serializer),
			Topic::This(ref hash) => hash.serialize(serializer),
		}
	}
}

impl<T> ops::Index<usize> for Topic<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		match *self {
			Topic::Any => panic!("Topic unavailable"),
			Topic::This(ref topic) => {
				if index != 0 {
					panic!("Topic unavailable");
				}
				topic
			}
			Topic::OneOf(ref topics) => topics.index(index),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Topic;
	#[cfg(feature = "serde")]
	use super::TopicFilter;
	#[cfg(not(feature = "std"))]
	use crate::no_std_prelude::*;
	#[cfg(feature = "serde")]
	use crate::Hash;

	#[cfg(feature = "serde")]
	fn hash(s: &'static str) -> Hash {
		s.parse().unwrap()
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_topic_filter_serialization() {
		let expected = r#"["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b",null,["0x000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b","0x0000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc"],null]"#;

		let topic = TopicFilter {
			topic0: Topic::This(hash("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b")),
			topic1: Topic::Any,
			topic2: Topic::OneOf(vec![
				hash("000000000000000000000000a94f5374fce5edbc8e2a8697c15331677e6ebf0b"),
				hash("0000000000000000000000000aff3454fce5edbc8cca8697c15331677e6ebccc"),
			]),
			topic3: Topic::Any,
		};

		let topic_str = serde_json::to_string(&topic).unwrap();
		assert_eq!(expected, &topic_str);
	}

	#[test]
	fn test_topic_from() {
		assert_eq!(Topic::Any as Topic<u64>, None.into());
		assert_eq!(Topic::This(10u64), 10u64.into());
		assert_eq!(Topic::OneOf(vec![10u64, 20]), vec![10u64, 20].into());
	}

	#[test]
	fn test_topic_into_vec() {
		let expected: Vec<u64> = vec![];
		let is: Vec<u64> = (Topic::Any as Topic<u64>).into();
		assert_eq!(expected, is);
		let expected: Vec<u64> = vec![10];
		let is: Vec<u64> = Topic::This(10u64).into();
		assert_eq!(expected, is);
		let expected: Vec<u64> = vec![10, 20];
		let is: Vec<u64> = Topic::OneOf(vec![10u64, 20]).into();
		assert_eq!(expected, is);
	}

	#[test]
	fn test_topic_is_any() {
		assert!((Topic::Any as Topic<u8>).is_any());
		assert!(!Topic::OneOf(vec![10u64, 20]).is_any());
		assert!(!Topic::This(10u64).is_any());
	}

	#[test]
	fn test_topic_index() {
		assert_eq!(Topic::OneOf(vec![10u64, 20])[0], 10);
		assert_eq!(Topic::OneOf(vec![10u64, 20])[1], 20);
		assert_eq!(Topic::This(10u64)[0], 10);
	}

	#[test]
	#[should_panic(expected = "Topic unavailable")]
	fn test_topic_index_panic() {
		let _ = (Topic::Any as Topic<u8>)[0];
	}

	#[test]
	#[should_panic(expected = "Topic unavailable")]
	fn test_topic_index_panic2() {
		assert_eq!(Topic::This(10u64)[1], 10);
	}
}
