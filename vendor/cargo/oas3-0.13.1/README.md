# `oas3`

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/oas3?label=latest)](https://crates.io/crates/oas3)
[![Documentation](https://docs.rs/oas3/badge.svg?version=0.13.1)](https://docs.rs/oas3/0.13.1)
[![dependency status](https://deps.rs/crate/oas3/0.13.1/status.svg)](https://deps.rs/crate/oas3/0.13.1)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/oas3.svg)
<br />
[![CI](https://github.com/x52dev/oas3-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/x52dev/oas3-rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/x52dev/oas3-rs/graph/badge.svg?token=OpYe6I7dj5)](https://codecov.io/gh/x52dev/oas3-rs)
![Version](https://img.shields.io/crates/msrv/oas3.svg)
[![Download](https://img.shields.io/crates/d/oas3.svg)](https://crates.io/crates/oas3)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

Structures and tools to parse, navigate and validate [OpenAPI v3.1] specifications.

Note that due to v3.1 being a breaking change from v3.0, you may have trouble correctly parsing
specs in the older format.

## Example

```rust
match oas3::from_path("path/to/openapi.yml") {
  Ok(spec) => println!("spec: {:?}", spec),
  Err(err) => println!("error: {}", err)
}
```

[OpenAPI v3.1]: https://spec.openapis.org/oas/v3.1.0

<!-- cargo-rdme end -->
