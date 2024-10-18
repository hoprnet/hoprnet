# `serde_html_form`

(De-)serialization support for the `application/x-www-form-urlencoded` format.

This crate is a Rust library for serialising to and deserialising from
the [`application/x-www-form-urlencoded`][urlencoded] format. It is built
upon [Serde], a high performance generic serialization framework and [rust-url],
a URL parser for Rust.

It is a fork of [`serde_urlencoded`], with additional support for maps or
structs with fields of sequence type (e.g. `Vec<String>`). It also supports
`Option` in values, treating `foo=` as `foo: None`.

[rust-url]: https://github.com/servo/rust-url
[Serde]: https://github.com/serde-rs/serde
[urlencoded]: https://url.spec.whatwg.org/#application/x-www-form-urlencoded
[`serde_urlencoded`]: https://github.com/nox/serde_urlencoded

## Examples

Sequences like `value=x&value=y`:

```rust
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
struct Form {
    // By default, at least one occurrence of this field must be present (this
    // is mandated by how serde works).
    //
    // Since this is usually not desired, use `serde(default)` to instantiate
    // this struct's field with a `Default` value if input doesn't contain that
    // field.
    #[serde(default)]
    value: Vec<String>,
}

assert_eq!(
    serde_html_form::from_str("value=&value=abc"),
    Ok(Form { value: vec!["".to_owned(), "abc".to_owned()] })
);
assert_eq!(
    serde_html_form::from_str(""),
    Ok(Form { value: vec![] })
);
```

Sequences like `value[]=x&value[]=y`:

```rust
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
struct Form {
    // If you want to support `value[]=x&value[]=y`, you can use
    // `serde(rename)`. You could even use `serde(alias)` instead to allow both,
    // but note that mixing both in one input string would also be allowed then.
    #[serde(default, rename = "value[]")]
    value: Vec<String>,
}

assert_eq!(
    serde_html_form::from_str("value[]=x&value[]=y"),
    Ok(Form { value: vec!["x".to_owned(), "y".to_owned()] })
);
assert_eq!(
    serde_html_form::from_str("value[]=hello"),
    Ok(Form { value: vec!["hello".to_owned()] })
);
```

Optional values:

```rust
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
struct Form {
    // Finally, this crate also supports deserializing empty values as `None`
    // if your values are `Option`s.
    // Note that serde's `Deserialize` derive implicitly allows omission of
    // `Option`-typed fields (except when combined with some other attributes).
    single: Option<u32>,
    // Not using `serde(default)` here to require at least one occurrence.
    at_least_one: Vec<Option<u32>>,
}

assert_eq!(
    serde_html_form::from_str("at_least_one=5"),
    Ok(Form {
        // Implicit `serde(default)` in action.
        single: None,
        // `serde_html_form`'s support for optional values being used.
        at_least_one: vec![Some(5)],
    })
);
assert_eq!(
    serde_html_form::from_str("at_least_one=&single=1&at_least_one=5"),
    Ok(Form {
        single: Some(1),
        at_least_one: vec![
            // Empty strings get deserialized as `None`.
            None,
            // It's no problem that the `at_least_one` field repetitions are
            // not consecutive (single comes in between).
            Some(5),
        ]
    })
);
assert!(
    serde_html_form::from_str::<Form>("").is_err(),
    "at_least_one is not part of the input"
);
```

## License

This crate is licensed under the MIT license ([LICENSE](LICENSE) or
<https://opensource.org/license/mit/>).
