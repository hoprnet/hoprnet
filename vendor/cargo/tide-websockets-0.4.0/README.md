# tide-websockets


## experimental websockets handler for [tide](https://github.com/http-rs/tide) based on [async-tungstenite](https://github.com/sdroege/async-tungstenite)

* [CI ![CI][ci-badge]][ci]
* [API Docs][docs] [![docs.rs docs][docs-badge]][docs]
* [Releases][releases] [![crates.io version][version-badge]][lib-rs]

[ci]: https://github.com/http-rs/tide-websockets/actions?query=workflow%3ACI
[ci-badge]: https://github.com/http-rs/tide-websockets/workflows/CI/badge.svg
[releases]: https://github.com/http-rs/tide-websockets/releases
[docs]: https://docs.rs/tide-websockets
[lib-rs]: https://lib.rs/tide-websockets
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square
[version-badge]: https://img.shields.io/crates/v/tide-websockets.svg?style=flat-square

## Installation
```sh
$ cargo add tide-websockets
```

## Using with tide

This can either be used as a middleware or as an endpoint. If used as a middleware, the endpoint will be executed if the request is not a websocket upgrade. If used as an endpoint but the request is not a websocket request, tide will reply with a `426 Upgrade Required` status code.

see [the example](https://github.com/http-rs/tide-websockets/blob/main/examples/example.rs) for the most up-to-date example of usage

## Safety
This crate uses ``#![deny(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## Alternatives
- [tide-websockets-sink](https://github.com/cryptoquick/tide-websockets-sink) - A fork of this project that implements the Sink trait.

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br/>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

