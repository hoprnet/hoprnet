use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
struct NewType<T>(T);

#[test]
fn deserialize_newtype_i32() {
    let result = vec![("field".to_owned(), NewType(11))];

    assert_eq!(super::from_str("field=11"), Ok(result));
}

#[test]
fn deserialize_bytes() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(super::from_bytes(b"first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_str() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(super::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_borrowed_str() {
    let result = vec![("first", 23), ("last", 42)];

    assert_eq!(super::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_reader() {
    let result = vec![("first".to_owned(), 23), ("last".to_owned(), 42)];

    assert_eq!(super::from_reader(b"first=23&last=42" as &[_]), Ok(result));
}

#[test]
fn deserialize_option() {
    let result = vec![("first".to_owned(), Some(23)), ("last".to_owned(), Some(42))];
    assert_eq!(super::from_str("first=23&last=42"), Ok(result));
}

#[test]
fn deserialize_empty_string() {
    let result = vec![("first".to_owned(), "")];
    assert_eq!(super::from_str("first="), Ok(result));
}

#[test]
fn deserialize_map() {
    let result = BTreeMap::from_iter([("first".to_owned(), 23), ("second".to_owned(), 42)]);
    assert_eq!(super::from_str("first=23&second=42"), Ok(result));
}

#[test]
fn deserialize_map_vec() {
    let result =
        BTreeMap::from_iter([("first".to_owned(), vec![23, 1]), ("second".to_owned(), vec![42])]);
    assert_eq!(super::from_str("first=23&second=42&first=1"), Ok(result));
}

#[test]
fn deserialize_tuple_list() {
    let result = vec![("foo".to_owned(), 1), ("bar".to_owned(), 2), ("foo".to_owned(), 3)];
    assert_eq!(super::from_str("foo=1&bar=2&foo=3"), Ok(result));
}

#[test]
fn deserialize_vec_strings() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Vec<String>,
    }

    assert_eq!(
        super::from_str("value=&value=abc"),
        Ok(Form { value: vec!["".to_owned(), "abc".to_owned()] })
    );
}

#[test]
fn deserialize_option_vec() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Option<Vec<String>>,
    }

    assert_eq!(super::from_str(""), Ok(Form { value: None }));
    assert_eq!(super::from_str("value=abc"), Ok(Form { value: Some(vec!["abc".to_owned()]) }));
    assert_eq!(
        super::from_str("value=abc&value=def"),
        Ok(Form { value: Some(vec!["abc".to_owned(), "def".to_owned()]) })
    );
}

#[test]
fn deserialize_option_vec_int() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Option<Vec<i32>>,
    }

    assert_eq!(super::from_str(""), Ok(Form { value: None }));
    assert_eq!(super::from_str("value=0"), Ok(Form { value: Some(vec![0]) }));
    assert_eq!(super::from_str("value=3&value=-1"), Ok(Form { value: Some(vec![3, -1]) }));
}

#[test]
fn deserialize_option_no_value() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Option<f64>,
    }

    assert_eq!(super::from_str("value="), Ok(Form { value: None }));
}

#[test]
fn deserialize_vec_options_no_value() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Vec<Option<f64>>,
    }

    assert_eq!(super::from_str("value=&value=&value="), Ok(Form { value: vec![None, None, None] }));
}

#[test]
fn deserialize_vec_options_some_values() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Vec<Option<f64>>,
    }

    assert_eq!(
        super::from_str("value=&value=4&value="),
        Ok(Form { value: vec![None, Some(4.0), None] })
    );
}

#[test]
fn deserialize_option_vec_no_value() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Option<Vec<f64>>,
    }

    assert_eq!(
        super::from_str::<Form>("value=&value=&value=").unwrap_err().to_string(),
        "cannot parse float from empty string"
    );
}

#[test]
fn deserialize_option_vec_with_values() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: Option<Vec<f64>>,
    }

    assert_eq!(
        super::from_str("value=3&value=4&value=5"),
        Ok(Form { value: Some(vec![3.0, 4.0, 5.0]) })
    );
}

#[test]
fn deserialize_no_value_err() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Form {
        value: f64,
    }

    assert_eq!(
        super::from_str::<Form>("value=").unwrap_err().to_string(),
        "cannot parse float from empty string"
    );
}

#[test]
fn deserialize_unit() {
    assert_eq!(super::from_str(""), Ok(()));
    assert_eq!(super::from_str("&"), Ok(()));
    assert_eq!(super::from_str("&&"), Ok(()));
    assert!(super::from_str::<()>("first=23").is_err());
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
enum X {
    A,
    B,
    C,
}

#[test]
fn deserialize_unit_enum() {
    let result =
        vec![("one".to_owned(), X::A), ("two".to_owned(), X::B), ("three".to_owned(), X::C)];

    assert_eq!(super::from_str("one=A&two=B&three=C"), Ok(result));
}

#[test]
fn deserialize_unit_type() {
    assert_eq!(super::from_str(""), Ok(()));
}
