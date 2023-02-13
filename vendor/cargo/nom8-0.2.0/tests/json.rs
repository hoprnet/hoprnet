#![cfg(feature = "alloc")]

use nom8::{
  branch::alt,
  bytes::{any, none_of, tag, take},
  character::{f64, multispace0},
  error::ParseError,
  multi::{fold_many0, separated_list0},
  sequence::{delimited, preceded, separated_pair},
  IResult, Parser,
};

use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
  Null,
  Bool(bool),
  Str(String),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(HashMap<String, JsonValue>),
}

fn boolean(input: &str) -> IResult<&str, bool> {
  alt((tag("false").value(false), tag("true").value(true)))(input)
}

fn u16_hex(input: &str) -> IResult<&str, u16> {
  take(4usize)
    .map_res(|s| u16::from_str_radix(s, 16))
    .parse(input)
}

fn unicode_escape(input: &str) -> IResult<&str, char> {
  alt((
    // Not a surrogate
    u16_hex
      .verify(|cp| !(0xD800..0xE000).contains(cp))
      .map(|cp| cp as u32),
    // See https://en.wikipedia.org/wiki/UTF-16#Code_points_from_U+010000_to_U+10FFFF for details
    separated_pair(u16_hex, "\\u", u16_hex)
      .verify(|(high, low)| (0xD800..0xDC00).contains(high) && (0xDC00..0xE000).contains(low))
      .map(|(high, low)| {
        let high_ten = (high as u32) - 0xD800;
        let low_ten = (low as u32) - 0xDC00;
        (high_ten << 10) + low_ten + 0x10000
      }),
  ))
  .map_opt(
    // Could be probably replaced with .unwrap() or _unchecked due to the verify checks
    std::char::from_u32,
  )
  .parse(input)
}

fn character(input: &str) -> IResult<&str, char> {
  let (input, c) = none_of("\"")(input)?;
  if c == '\\' {
    alt((
      any.map_res(|c| {
        Ok(match c {
          '"' | '\\' | '/' => c,
          'b' => '\x08',
          'f' => '\x0C',
          'n' => '\n',
          'r' => '\r',
          't' => '\t',
          _ => return Err(()),
        })
      }),
      preceded('u', unicode_escape),
    ))(input)
  } else {
    Ok((input, c))
  }
}

fn string(input: &str) -> IResult<&str, String> {
  delimited(
    '"',
    fold_many0(character, String::new, |mut string, c| {
      string.push(c);
      string
    }),
    '"',
  )(input)
}

fn ws<'a, O, E: ParseError<&'a str>, F: Parser<&'a str, O, E>>(f: F) -> impl Parser<&'a str, O, E> {
  delimited(multispace0, f, multispace0)
}

fn array(input: &str) -> IResult<&str, Vec<JsonValue>> {
  delimited('[', ws(separated_list0(ws(','), json_value)), ']')(input)
}

fn object(input: &str) -> IResult<&str, HashMap<String, JsonValue>> {
  delimited(
    '{',
    ws(separated_list0(
      ws(','),
      separated_pair(string, ws(':'), json_value),
    )),
    '}',
  )
  .map(|key_values| key_values.into_iter().collect())
  .parse(input)
}

fn json_value(input: &str) -> IResult<&str, JsonValue> {
  use JsonValue::*;

  alt((
    tag("null").value(Null),
    boolean.map(Bool),
    string.map(Str),
    f64.map(Num),
    array.map(Array),
    object.map(Object),
  ))(input)
}

fn json(input: &str) -> IResult<&str, JsonValue> {
  ws(json_value).parse(input)
}

#[test]
fn json_string() {
  assert_eq!(string("\"\""), Ok(("", "".to_string())));
  assert_eq!(string("\"abc\""), Ok(("", "abc".to_string())));
  assert_eq!(
    string("\"abc\\\"\\\\\\/\\b\\f\\n\\r\\t\\u0001\\u2014\u{2014}def\""),
    Ok(("", "abc\"\\/\x08\x0C\n\r\t\x01——def".to_string())),
  );
  assert_eq!(string("\"\\uD83D\\uDE10\""), Ok(("", "😐".to_string())));

  assert!(string("\"").is_err());
  assert!(string("\"abc").is_err());
  assert!(string("\"\\\"").is_err());
  assert!(string("\"\\u123\"").is_err());
  assert!(string("\"\\uD800\"").is_err());
  assert!(string("\"\\uD800\\uD800\"").is_err());
  assert!(string("\"\\uDC00\"").is_err());
}

#[test]
fn json_object() {
  use JsonValue::*;

  let input = r#"{"a":42,"b":"x"}"#;

  let expected = Object(
    vec![
      ("a".to_string(), Num(42.0)),
      ("b".to_string(), Str("x".to_string())),
    ]
    .into_iter()
    .collect(),
  );

  assert_eq!(json(input), Ok(("", expected)));
}

#[test]
fn json_array() {
  use JsonValue::*;

  let input = r#"[42,"x"]"#;

  let expected = Array(vec![Num(42.0), Str("x".to_string())]);

  assert_eq!(json(input), Ok(("", expected)));
}

#[test]
fn json_whitespace() {
  use JsonValue::*;

  let input = r#"
  {
    "null" : null,
    "true"  :true ,
    "false":  false  ,
    "number" : 123e4 ,
    "string" : " abc 123 " ,
    "array" : [ false , 1 , "two" ] ,
    "object" : { "a" : 1.0 , "b" : "c" } ,
    "empty_array" : [  ] ,
    "empty_object" : {   }
  }
  "#;

  assert_eq!(
    json(input),
    Ok((
      "",
      Object(
        vec![
          ("null".to_string(), Null),
          ("true".to_string(), Bool(true)),
          ("false".to_string(), Bool(false)),
          ("number".to_string(), Num(123e4)),
          ("string".to_string(), Str(" abc 123 ".to_string())),
          (
            "array".to_string(),
            Array(vec![Bool(false), Num(1.0), Str("two".to_string())])
          ),
          (
            "object".to_string(),
            Object(
              vec![
                ("a".to_string(), Num(1.0)),
                ("b".to_string(), Str("c".to_string())),
              ]
              .into_iter()
              .collect()
            )
          ),
          ("empty_array".to_string(), Array(vec![]),),
          ("empty_object".to_string(), Object(HashMap::new()),),
        ]
        .into_iter()
        .collect()
      )
    ))
  );
}
