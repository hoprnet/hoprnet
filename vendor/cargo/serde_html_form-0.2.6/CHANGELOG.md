# 0.2.6

Fix deserialization of optional sequences of a single non-string element.

# 0.2.5

Add `push_to_string` for serializing a struct to the end of an existing `String`
buffer (instead of allocating a fresh one for the serialized output).

# 0.2.4

Fix deserialization of optional sequences of a single element.

# 0.2.3

Improve README and crate documentation (now the exact same, instead of just a
single-line description).

# 0.2.2

This release only upgrades one of the crates' dev-dependencies.

# 0.2.1

This release only upgrades one of the crates' private dependencies.

# 0.2.0

Support deserialization of sequences with duplicate keys.
This used to fail, but passes now:

```rust
let result = vec![("foo".to_owned(), 1), ("bar".to_owned(), 2), ("foo".to_owned(), 3)];
assert_eq!(super::from_str("foo=1&bar=2&foo=3"), Ok(result));
```

This should mainly affect deserialization to a type that's explicitly a sequence, like arrays or `Vec`,
but some other things were changed too so if you are getting unexpected deserialization errors, please open an issue.

This release has a minimum Rust version of 1.56.

# 0.1.1

Support deserialization of `Option`al values to better support forms with optional inputs of non-string types:

```rust
#[derive(Deserialize, PartialEq)]
struct MyForm {
    field: Option<u16>,
}

// What browsers send when a value is given
assert_eq!(serde_html_form::from_str("field=5").unwrap(), MyForm { field: Some(5) });
// What browsers send when no value is given
assert_eq!(serde_html_form::from_str("field=").unwrap(), MyForm { field: None });
// This also works
assert_eq!(serde_html_form::from_str("").unwrap(), MyForm { field: None });
```

# 0.1.0

Initial release.
