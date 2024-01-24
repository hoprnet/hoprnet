# openssl-probe

Tool for helping to find SSL certificate locations on the system for OpenSSL

[![Crates.io](https://img.shields.io/crates/v/openssl-probe.svg?maxAge=2592000)](https://crates.io/crates/openssl-probe)
[![docs.rs](https://docs.rs/openssl-probe/badge.svg)](https://docs.rs/openssl-probe/)

## Usage

First, add this to your `Cargo.toml`:

```toml
[dependencies]
openssl-probe = "0.1.2"
```

Then add this to your crate:

```rust
extern crate openssl_probe;

fn main() {
    openssl_probe::init_ssl_cert_env_vars();
    //... your code
}
```

## License

`openssl-probe` is primarily distributed under the terms of both the MIT license and the Apache License (Version 2.0),
with portions covered by various BSD-like licenses.

See [LICENSE-APACHE](./LICENSE-APACHE), and [LICENSE-MIT](LICENSE-MIT) for details.
