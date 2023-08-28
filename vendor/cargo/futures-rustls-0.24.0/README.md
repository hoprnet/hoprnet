# futures-rustls
[![github actions](https://github.com/quininer/tls/workflows/Rust/badge.svg)](https://github.com/quininer/futures-rustls/actions)
[![crates](https://img.shields.io/crates/v/futures-rustls.svg)](https://crates.io/crates/futures-rustls)
[![docs.rs](https://docs.rs/futures-rustls/badge.svg)](https://docs.rs/futures-rustls/)

Asynchronous TLS/SSL streams for futures using
[Rustls](https://github.com/ctz/rustls).

### Basic Structure of a Client

```rust
use webpki::DNSNameRef;
use futures_rustls::{ TlsConnector, rustls::ClientConfig };

// ...

let mut config = ClientConfig::new();
config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
let config = TlsConnector::from(Arc::new(config));
let dnsname = DNSNameRef::try_from_ascii_str("www.rust-lang.org").unwrap();

let stream = TcpStream::connect(&addr).await?;
let mut stream = config.connect(dnsname, stream).await?;

// ...
```

### License & Origin

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

This started as a fork of [tokio-rustls](https://github.com/tokio-rs/tls).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in futures-rustls by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
