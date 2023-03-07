// Copyright 2016-2017 Jonathan Creekmore
//
// Licensed under the MIT license <LICENSE.md or
// http://opensource.org/licenses/MIT>. This file may not be
// copied, modified, or distributed except according to those terms.

//! This crate provides a parser and encoder for PEM-encoded binary data.
//! PEM-encoded binary data is essentially a beginning and matching end
//! tag that encloses base64-encoded binary data (see:
//! https://en.wikipedia.org/wiki/Privacy-enhanced_Electronic_Mail).
//!
//! This crate's documentation provides a few simple examples along with
//! documentation on the public methods for the crate.
//!
//! # Usage
//!
//! This crate is [on crates.io](https://crates.io/crates/pem) and can be used
//! by adding `pem` to your dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! pem = "1"
//! ```
//!
//! and this to your crate root:
//!
//! ```rust
//! extern crate pem;
//! ```
//!
//! Using the `serde` feature will implement the serde traits for
//! the `Pem` struct.
//!
//! # Example: parse a single chunk of PEM-encoded text
//!
//! Generally, PEM-encoded files contain a single chunk of PEM-encoded
//! text. Commonly, this is in some sort of a key file or an x.509
//! certificate.
//!
//! ```rust
//!
//! use pem::parse;
//!
//! const SAMPLE: &'static str = "-----BEGIN RSA PRIVATE KEY-----
//! MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
//! dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
//! 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
//! AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
//! DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
//! TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
//! ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
//! -----END RSA PRIVATE KEY-----
//! ";
//!
//!  let pem = parse(SAMPLE).unwrap();
//!  assert_eq!(pem.tag, "RSA PRIVATE KEY");
//! ```
//!
//! # Example: parse a set of PEM-encoded test
//!
//! Sometimes, PEM-encoded files contain multiple chunks of PEM-encoded
//! text. You might see this if you have an x.509 certificate file that
//! also includes intermediate certificates.
//!
//! ```rust
//!
//! use pem::parse_many;
//!
//! const SAMPLE: &'static str = "-----BEGIN INTERMEDIATE CERT-----
//! MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
//! dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
//! 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
//! AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
//! DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
//! TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
//! ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
//! -----END INTERMEDIATE CERT-----
//!
//! -----BEGIN CERTIFICATE-----
//! MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
//! dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
//! 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
//! AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
//! DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
//! TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
//! ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
//! -----END CERTIFICATE-----
//! ";
//!
//!  let pems = parse_many(SAMPLE).unwrap();
//!  assert_eq!(pems.len(), 2);
//!  assert_eq!(pems[0].tag, "INTERMEDIATE CERT");
//!  assert_eq!(pems[1].tag, "CERTIFICATE");
//! ```

#![recursion_limit = "1024"]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

mod errors;
mod parser;
use parser::{parse_captures, parse_captures_iter, Captures};

pub use crate::errors::{PemError, Result};
use std::str;

/// The line length for PEM encoding
const LINE_WRAP: usize = 64;

/// Enum describing line endings
#[derive(Debug, Clone, Copy)]
pub enum LineEnding {
    /// Windows-like (`\r\n`)
    CRLF,
    /// Unix-like (`\n`)
    LF,
}

/// Configuration for Pem encoding
#[derive(Debug, Clone, Copy)]
pub struct EncodeConfig {
    /// Line ending to use during encoding
    pub line_ending: LineEnding,
}

/// A representation of Pem-encoded data
#[derive(PartialEq, Debug, Clone)]
pub struct Pem {
    /// The tag extracted from the Pem-encoded data
    pub tag: String,
    /// The binary contents of the Pem-encoded data
    pub contents: Vec<u8>,
}

fn decode_data(raw_data: &str) -> Result<Vec<u8>> {
    // We need to get rid of newlines for base64::decode
    // As base64 requires an AsRef<[u8]>, this must involve a copy
    let data: String = raw_data.lines().map(str::trim_end).collect();

    // And decode it from Base64 into a vector of u8
    let contents = base64::decode_config(&data, base64::STANDARD).map_err(PemError::InvalidData)?;

    Ok(contents)
}

impl Pem {
    fn new_from_captures(caps: Captures) -> Result<Pem> {
        fn as_utf8<'a>(bytes: &'a [u8]) -> Result<&'a str> {
            Ok(str::from_utf8(bytes).map_err(PemError::NotUtf8)?)
        }

        // Verify that the begin section exists
        let tag = as_utf8(caps.begin)?;
        if tag.is_empty() {
            return Err(PemError::MissingBeginTag);
        }

        // as well as the end section
        let tag_end = as_utf8(caps.end)?;
        if tag_end.is_empty() {
            return Err(PemError::MissingEndTag);
        }

        // The beginning and the end sections must match
        if tag != tag_end {
            return Err(PemError::MismatchedTags(tag.into(), tag_end.into()));
        }

        // If they did, then we can grab the data section
        let raw_data = as_utf8(caps.data)?;
        let contents = decode_data(raw_data)?;

        Ok(Pem {
            tag: tag.to_owned(),
            contents,
        })
    }
}

/// Parses a single PEM-encoded data from a data-type that can be dereferenced as a [u8].
///
/// # Example: parse PEM-encoded data from a Vec<u8>
/// ```rust
///
/// use pem::parse;
///
/// const SAMPLE: &'static str = "-----BEGIN RSA PRIVATE KEY-----
/// MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
/// dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
/// 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
/// AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
/// DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
/// TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
/// ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
/// -----END RSA PRIVATE KEY-----
/// ";
/// let SAMPLE_BYTES: Vec<u8> = SAMPLE.into();
///
///  let pem = parse(SAMPLE_BYTES).unwrap();
///  assert_eq!(pem.tag, "RSA PRIVATE KEY");
/// ```
///
/// # Example: parse PEM-encoded data from a String
/// ```rust
///
/// use pem::parse;
///
/// const SAMPLE: &'static str = "-----BEGIN RSA PRIVATE KEY-----
/// MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
/// dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
/// 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
/// AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
/// DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
/// TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
/// ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
/// -----END RSA PRIVATE KEY-----
/// ";
/// let SAMPLE_STRING: String = SAMPLE.into();
///
///  let pem = parse(SAMPLE_STRING).unwrap();
///  assert_eq!(pem.tag, "RSA PRIVATE KEY");
/// ```
pub fn parse<B: AsRef<[u8]>>(input: B) -> Result<Pem> {
    parse_captures(&input.as_ref())
        .ok_or_else(|| PemError::MalformedFraming)
        .and_then(Pem::new_from_captures)
}

/// Parses a set of PEM-encoded data from a data-type that can be dereferenced as a [u8].
///
/// # Example: parse a set of PEM-encoded data from a Vec<u8>
///
/// ```rust
///
/// use pem::parse_many;
///
/// const SAMPLE: &'static str = "-----BEGIN INTERMEDIATE CERT-----
/// MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
/// dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
/// 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
/// AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
/// DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
/// TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
/// ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
/// -----END INTERMEDIATE CERT-----
///
/// -----BEGIN CERTIFICATE-----
/// MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
/// dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
/// 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
/// AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
/// DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
/// TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
/// ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
/// -----END CERTIFICATE-----
/// ";
/// let SAMPLE_BYTES: Vec<u8> = SAMPLE.into();
///
///  let pems = parse_many(SAMPLE_BYTES).unwrap();
///  assert_eq!(pems.len(), 2);
///  assert_eq!(pems[0].tag, "INTERMEDIATE CERT");
///  assert_eq!(pems[1].tag, "CERTIFICATE");
/// ```
///
/// # Example: parse a set of PEM-encoded data from a String
///
/// ```rust
///
/// use pem::parse_many;
///
/// const SAMPLE: &'static str = "-----BEGIN INTERMEDIATE CERT-----
/// MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
/// dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
/// 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
/// AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
/// DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
/// TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
/// ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
/// -----END INTERMEDIATE CERT-----
///
/// -----BEGIN CERTIFICATE-----
/// MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
/// dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
/// 2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
/// AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
/// DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
/// TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
/// ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
/// -----END CERTIFICATE-----
/// ";
///  let SAMPLE_STRING: Vec<u8> = SAMPLE.into();
///
///  let pems = parse_many(SAMPLE_STRING).unwrap();
///  assert_eq!(pems.len(), 2);
///  assert_eq!(pems[0].tag, "INTERMEDIATE CERT");
///  assert_eq!(pems[1].tag, "CERTIFICATE");
/// ```
pub fn parse_many<B: AsRef<[u8]>>(input: B) -> Result<Vec<Pem>> {
    // Each time our regex matches a PEM section, we need to decode it.
    parse_captures_iter(&input.as_ref())
        .map(|caps| Pem::new_from_captures(caps))
        .collect()
}

/// Encode a PEM struct into a PEM-encoded data string
///
/// # Example
/// ```rust
///  use pem::{Pem, encode};
///
///  let pem = Pem {
///     tag: String::from("FOO"),
///     contents: vec![1, 2, 3, 4],
///   };
///   encode(&pem);
/// ```
pub fn encode(pem: &Pem) -> String {
    encode_config(
        pem,
        EncodeConfig {
            line_ending: LineEnding::CRLF,
        },
    )
}

/// Encode a PEM struct into a PEM-encoded data string with additional
/// configuration options
///
/// # Example
/// ```rust
///  use pem::{Pem, encode_config, EncodeConfig, LineEnding};
///
///  let pem = Pem {
///     tag: String::from("FOO"),
///     contents: vec![1, 2, 3, 4],
///   };
///   encode_config(&pem, EncodeConfig { line_ending: LineEnding::LF });
/// ```
pub fn encode_config(pem: &Pem, config: EncodeConfig) -> String {
    let line_ending = match config.line_ending {
        LineEnding::CRLF => "\r\n",
        LineEnding::LF => "\n",
    };

    let mut output = String::new();

    let contents = if pem.contents.is_empty() {
        String::from("")
    } else {
        base64::encode_config(
            &pem.contents,
            base64::Config::new(base64::CharacterSet::Standard, true),
        )
    };

    output.push_str(&format!("-----BEGIN {}-----{}", pem.tag, line_ending));
    for c in contents.as_bytes().chunks(LINE_WRAP) {
        output.push_str(&format!("{}{}", str::from_utf8(c).unwrap(), line_ending));
    }
    output.push_str(&format!("-----END {}-----{}", pem.tag, line_ending));

    output
}

/// Encode multiple PEM structs into a PEM-encoded data string
///
/// # Example
/// ```rust
///  use pem::{Pem, encode_many};
///
///  let data = vec![
///     Pem {
///         tag: String::from("FOO"),
///         contents: vec![1, 2, 3, 4],
///     },
///     Pem {
///         tag: String::from("BAR"),
///         contents: vec![5, 6, 7, 8],
///     },
///   ];
///   encode_many(&data);
/// ```
pub fn encode_many(pems: &[Pem]) -> String {
    pems.iter()
        .map(encode)
        .collect::<Vec<String>>()
        .join("\r\n")
}

/// Encode multiple PEM structs into a PEM-encoded data string with additional
/// configuration options
///
/// Same config will be used for each PEM struct.
///
/// # Example
/// ```rust
///  use pem::{Pem, encode_many_config, EncodeConfig, LineEnding};
///
///  let data = vec![
///     Pem {
///         tag: String::from("FOO"),
///         contents: vec![1, 2, 3, 4],
///     },
///     Pem {
///         tag: String::from("BAR"),
///         contents: vec![5, 6, 7, 8],
///     },
///   ];
///   encode_many_config(&data, EncodeConfig { line_ending: LineEnding::LF });
/// ```
pub fn encode_many_config(pems: &[Pem], config: EncodeConfig) -> String {
    let line_ending = match config.line_ending {
        LineEnding::CRLF => "\r\n",
        LineEnding::LF => "\n",
    };
    pems.iter()
        .map(|value| encode_config(value, config))
        .collect::<Vec<String>>()
        .join(line_ending)
}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::{encode, parse, Pem};
    use serde::{
        de::{Error, Visitor},
        Deserialize, Serialize,
    };
    use std::fmt;

    impl Serialize for Pem {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&encode(self))
        }
    }

    struct PemVisitor;

    impl<'de> Visitor<'de> for PemVisitor {
        type Value = Pem;

        fn expecting(&self, _formatter: &mut fmt::Formatter) -> fmt::Result {
            Ok(())
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            parse(v).map_err(Error::custom)
        }
    }

    impl<'de> Deserialize<'de> for Pem {
        fn deserialize<D>(deserializer: D) -> Result<Pem, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(PemVisitor)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    const SAMPLE_CRLF: &'static str = "-----BEGIN RSA PRIVATE KEY-----\r
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc\r
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO\r
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei\r
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un\r
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT\r
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh\r
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ\r
-----END RSA PRIVATE KEY-----\r
\r
-----BEGIN RSA PUBLIC KEY-----\r
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo\r
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0\r
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI\r
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk\r
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6\r
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g\r
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg\r
-----END RSA PUBLIC KEY-----\r
";

    const SAMPLE_LF: &'static str = "-----BEGIN RSA PRIVATE KEY-----
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
-----END RSA PRIVATE KEY-----

-----BEGIN RSA PUBLIC KEY-----
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END RSA PUBLIC KEY-----
";

    #[test]
    fn test_parse_works() {
        let pem = parse(SAMPLE_CRLF).unwrap();
        assert_eq!(pem.tag, "RSA PRIVATE KEY");
    }

    #[test]
    fn test_parse_invalid_framing() {
        let input = "--BEGIN data-----
        -----END data-----";
        assert_eq!(parse(&input), Err(PemError::MalformedFraming));
    }

    #[test]
    fn test_parse_invalid_begin() {
        let input = "-----BEGIN -----
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END RSA PUBLIC KEY-----";
        assert_eq!(parse(&input), Err(PemError::MissingBeginTag));
    }

    #[test]
    fn test_parse_invalid_end() {
        let input = "-----BEGIN DATA-----
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END -----";
        assert_eq!(parse(&input), Err(PemError::MissingEndTag));
    }

    #[test]
    fn test_parse_invalid_data() {
        let input = "-----BEGIN DATA-----
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oY?
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END DATA-----";
        match parse(&input) {
            Err(e @ PemError::InvalidData(_)) => {
                assert_eq!(
                    &format!("{}", e.source().unwrap()),
                    "Invalid byte 63, offset 63."
                );
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_empty_data() {
        let input = "-----BEGIN DATA-----
-----END DATA-----";
        let pem = parse(&input).unwrap();
        assert_eq!(pem.contents.len(), 0);
    }

    #[test]
    fn test_parse_many_works() {
        let pems = parse_many(SAMPLE_CRLF).unwrap();
        assert_eq!(pems.len(), 2);
        assert_eq!(pems[0].tag, "RSA PRIVATE KEY");
        assert_eq!(pems[1].tag, "RSA PUBLIC KEY");
    }

    #[test]
    fn test_parse_many_errors_on_invalid_section() {
        let input = SAMPLE_LF.to_owned() + "-----BEGIN -----\n-----END -----";
        assert_eq!(parse_many(&input), Err(PemError::MissingBeginTag));
    }

    #[test]
    fn test_encode_empty_contents() {
        let pem = Pem {
            tag: String::from("FOO"),
            contents: vec![],
        };
        let encoded = encode(&pem);
        assert!(encoded != "");

        let pem_out = parse(&encoded).unwrap();
        assert_eq!(&pem, &pem_out);
    }

    #[test]
    fn test_encode_contents() {
        let pem = Pem {
            tag: String::from("FOO"),
            contents: vec![1, 2, 3, 4],
        };
        let encoded = encode(&pem);
        assert!(encoded != "");

        let pem_out = parse(&encoded).unwrap();
        assert_eq!(&pem, &pem_out);
    }

    #[test]
    fn test_encode_many() {
        let pems = parse_many(SAMPLE_CRLF).unwrap();
        let encoded = encode_many(&pems);

        assert_eq!(SAMPLE_CRLF, encoded);
    }

    #[test]
    fn test_encode_config_contents() {
        let pem = Pem {
            tag: String::from("FOO"),
            contents: vec![1, 2, 3, 4],
        };
        let config = EncodeConfig {
            line_ending: LineEnding::LF,
        };
        let encoded = encode_config(&pem, config);
        assert!(encoded != "");

        let pem_out = parse(&encoded).unwrap();
        assert_eq!(&pem, &pem_out);
    }

    #[test]
    fn test_encode_many_config() {
        let pems = parse_many(SAMPLE_LF).unwrap();
        let config = EncodeConfig {
            line_ending: LineEnding::LF,
        };
        let encoded = encode_many_config(&pems, config);

        assert_eq!(SAMPLE_LF, encoded);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let pem = Pem {
            tag: String::from("Mock tag"),
            contents: "Mock contents".as_bytes().to_vec(),
        };
        let value = serde_json::to_string_pretty(&pem).unwrap();
        let result = serde_json::from_str(&value).unwrap();
        assert_eq!(pem, result);
    }

    const HEADER_CRLF: &'static str = "-----BEGIN CERTIFICATE-----\r
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc\r
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO\r
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei\r
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un\r
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT\r
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh\r
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ\r
-----END CERTIFICATE-----\r
-----BEGIN RSA PRIVATE KEY-----\r
Proc-Type: 4,ENCRYPTED\r
DEK-Info: AES-256-CBC,975C518B7D2CCD1164A3354D1F89C5A6\r
\r
MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo\r
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0\r
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI\r
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk\r
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6\r
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g\r
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg\r
-----END RSA PRIVATE KEY-----\r
";
    const HEADER_CRLF_DATA: [&'static str; 2] = [
        "MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc\r
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO\r
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei\r
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un\r
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT\r
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh\r
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ\r",
        "MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo\r
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0\r
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI\r
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk\r
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6\r
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g\r
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg\r",
    ];

    const HEADER_LF: &'static str = "-----BEGIN CERTIFICATE-----
MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ
-----END CERTIFICATE-----
-----BEGIN RSA PRIVATE KEY-----
Proc-Type: 4,ENCRYPTED
DEK-Info: AES-256-CBC,975C518B7D2CCD1164A3354D1F89C5A6

MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg
-----END RSA PRIVATE KEY-----
";
    const HEADER_LF_DATA: [&'static str; 2] = [
        "MIIBPQIBAAJBAOsfi5AGYhdRs/x6q5H7kScxA0Kzzqe6WI6gf6+tc6IvKQJo5rQc
dWWSQ0nRGt2hOPDO+35NKhQEjBQxPh/v7n0CAwEAAQJBAOGaBAyuw0ICyENy5NsO
2gkT00AWTSzM9Zns0HedY31yEabkuFvrMCHjscEF7u3Y6PB7An3IzooBHchsFDei
AAECIQD/JahddzR5K3A6rzTidmAf1PBtqi7296EnWv8WvpfAAQIhAOvowIXZI4Un
DXjgZ9ekuUjZN+GUQRAVlkEEohGLVy59AiEA90VtqDdQuWWpvJX0cM08V10tLXrT
TTGsEtITid1ogAECIQDAaFl90ZgS5cMrL3wCeatVKzVUmuJmB/VAmlLFFGzK0QIh
ANJGc7AFk4fyFD/OezhwGHbWmo/S+bfeAiIh2Ss2FxKJ",
        "MIIBOgIBAAJBAMIeCnn9G/7g2Z6J+qHOE2XCLLuPoh5NHTO2Fm+PbzBvafBo0oYo
QVVy7frzxmOqx6iIZBxTyfAQqBPO3Br59BMCAwEAAQJAX+PjHPuxdqiwF6blTkS0
RFI1MrnzRbCmOkM6tgVO0cd6r5Z4bDGLusH9yjI9iI84gPRjK0AzymXFmBGuREHI
sQIhAPKf4pp+Prvutgq2ayygleZChBr1DC4XnnufBNtaswyvAiEAzNGVKgNvzuhk
ijoUXIDruJQEGFGvZTsi1D2RehXiT90CIQC4HOQUYKCydB7oWi1SHDokFW2yFyo6
/+lf3fgNjPI6OQIgUPmTFXciXxT1msh3gFLf3qt2Kv8wbr9Ad9SXjULVpGkCIB+g
RzHX0lkJl9Stshd/7Gbt65/QYq+v+xvAeT0CoyIg",
    ];

    fn cmp_data(left: &[u8], right: &[u8]) -> bool {
        if left.len() != right.len() {
            false
        } else {
            left.iter()
                .zip(right.iter())
                .all(|(left, right)| left == right)
        }
    }

    #[test]
    fn test_parse_many_with_headers_crlf() {
        let pems = parse_many(HEADER_CRLF).unwrap();
        assert_eq!(pems.len(), 2);
        assert_eq!(pems[0].tag, "CERTIFICATE");
        assert!(cmp_data(
            &pems[0].contents,
            &decode_data(HEADER_CRLF_DATA[0]).unwrap()
        ));
        assert_eq!(pems[1].tag, "RSA PRIVATE KEY");
        assert!(cmp_data(
            &pems[1].contents,
            &decode_data(HEADER_CRLF_DATA[1]).unwrap()
        ));
    }

    #[test]
    fn test_parse_many_with_headers_lf() {
        let pems = parse_many(HEADER_LF).unwrap();
        assert_eq!(pems.len(), 2);
        assert_eq!(pems[0].tag, "CERTIFICATE");
        assert!(cmp_data(
            &pems[0].contents,
            &decode_data(HEADER_LF_DATA[0]).unwrap()
        ));
        assert_eq!(pems[1].tag, "RSA PRIVATE KEY");
        assert!(cmp_data(
            &pems[1].contents,
            &decode_data(HEADER_LF_DATA[1]).unwrap()
        ));
    }
}
