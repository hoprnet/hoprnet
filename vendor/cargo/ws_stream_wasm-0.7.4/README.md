# ws_stream_wasm

[![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
[![Build Status](https://github.com/najamelan/ws_stream_wasm/workflows/ci/badge.svg?branch=release)](https://github.com/najamelan/ws_stream_wasm/actions)
[![Docs](https://docs.rs/ws_stream_wasm/badge.svg)](https://docs.rs/ws_stream_wasm)
[![crates.io](https://img.shields.io/crates/v/ws_stream_wasm.svg)](https://crates.io/crates/ws_stream_wasm)


> A convenience library for using web sockets in WASM

The _web-sys_ bindings for websockets aren't very convenient to use directly. This crates hopes to alleviate that. Browsers can't create direct TCP connections, and by putting `AsyncRead`/`AsyncWrite` on top of websockets, we can use interfaces that work over any async byte streams from within the browser. The crate has 2 main types. The `WsMeta` type exists to allow access to the web API while you pass `WsStream` to combinators that take ownership of the stream.

**features:**
- [`WsMeta`]: A wrapper around [`web_sys::WebSocket`].
- [`WsMessage`]: A simple rusty representation of a WebSocket message.
- [`WsStream`]: A _futures_ `Sink`/`Stream` of `WsMessage`.
                It also has a method `into_io()` which let's you get a wrapper that implements `AsyncRead`/`AsyncWrite`/`AsyncBufRead` (_tokio_ version behind the feature `tokio_io`).
- [`WsEvent`]: [`WsMeta`] is observable with [pharos](https://crates.io/crates/pharos) for events (mainly useful for connection close).

**NOTE:** this crate only works on WASM. If you want a server side equivalent that implements `AsyncRead`/`AsyncWrite` over
WebSockets, check out [ws_stream_tungstenite](https://crates.io/crates/ws_stream_tungstenite).

**missing features:**
- no automatic reconnect
- not all features are thoroughly tested. Notably, I have little use for extensions and sub-protocols. Tungstenite,
  which I use for the server end (and for automated testing) doesn't support these, making it hard to write unit tests.

## Table of Contents

- [Install](#install)
  - [Upgrade](#upgrade)
  - [Dependencies](#dependencies)
- [Usage](#usage)
  - [API](#api)
- [References](#references)
- [Contributing](#contributing)
  - [Code of Conduct](#code-of-conduct)
- [License](#license)


## Install
With [cargo add](https://github.com/killercup/cargo-edit):
`cargo add ws_stream_wasm`

With [cargo yaml](https://gitlab.com/storedbox/cargo-yaml):
```yaml
dependencies:

  ws_stream_wasm: ^0.7
```

In Cargo.toml:
```toml
[dependencies]

   ws_stream_wasm = "0.7"
```

### Upgrade

Please check out the [changelog](https://github.com/najamelan/ws_stream_wasm/blob/release/CHANGELOG.md) when upgrading.

### Dependencies

This crate has few dependencies. Cargo will automatically handle it's dependencies for you.

There is one optional features. The `tokio_io` features causes the `WsIo` returned from [`WsStream::into_io`] to implement the
tokio version of AsyncRead/AsyncWrite.


## Usage

The [integration tests](https://github.com/najamelan/ws_stream_wasm/tree/release/tests) show most features in action. The
example directory doesn't currently hold any interesting examples.

The types in this library are `Send` + `Sync` as far as the compiler is concerned. This is so that you can use them with general purpose
libraries that also work on WASM but that require a connection to be `Send`/`Sync`. Currently WASM has no threads though and most
underlying types we use aren't `Send`. The solution for the moment is to use [`send_wrapper::SendWrapper`]. This will panic
if it's ever dereferenced on a different thread than where it's created. You have to consider that the types aren't `Send`, but
on WASM it's safe to pass them to an API that requires `Send`, because there is not much multi-threading support. Thus passing it to
the bindgen executor will be fine. However with webworkers you can make extra threads nevertheless. The responsibility is on you
to assure you don't try to use the Web Api's on different threads.

The main entrypoint you'll want to use, eg to connect, is [`WsMeta::connect`].

### Basic events example
```rust
use
{
   ws_stream_wasm       :: *                        ,
   pharos               :: *                        ,
   wasm_bindgen         :: UnwrapThrowExt           ,
   wasm_bindgen_futures :: futures_0_3::spawn_local ,
   futures              :: stream::StreamExt        ,
};

let program = async
{
   let (mut ws, _wsio) = WsMeta::connect( "ws://127.0.0.1:3012", None ).await

      .expect_throw( "assume the connection succeeds" );

   let mut evts = ws.observe( ObserveConfig::default() ).expect_throw( "observe" );

   ws.close().await;

   // Note that since WsMeta::connect resolves to an opened connection, we don't see
   // any Open events here.
   //
   assert!( evts.next().await.unwrap_throw().is_closing() );
   assert!( evts.next().await.unwrap_throw().is_closed () );
};

spawn_local( program );
```

### Filter events example

This shows how to filter events. The functionality comes from _pharos_ which we use to make
[`WsMeta`] observable.

```rust
use
{
   ws_stream_wasm       :: *                        ,
   pharos               :: *                        ,
   wasm_bindgen         :: UnwrapThrowExt           ,
   wasm_bindgen_futures :: futures_0_3::spawn_local ,
   futures              :: stream::StreamExt        ,
};

let program = async
{
   let (mut ws, _wsio) = WsMeta::connect( "ws://127.0.0.1:3012", None ).await

      .expect_throw( "assume the connection succeeds" );

   // The Filter type comes from the pharos crate.
   //
   let mut evts = ws.observe( Filter::Pointer( WsEvent::is_closed ).into() ).expect_throw( "observe" );

   ws.close().await;

   // Note we will only get the closed event here, the WsEvent::Closing has been filtered out.
   //
   assert!( evts.next().await.unwrap_throw().is_closed () );
};

spawn_local( program );
```

## API

Api documentation can be found on [docs.rs](https://docs.rs/ws_stream_wasm).


## References
The reference documents for understanding web sockets and how the browser handles them are:
- [HTML Living Standard](https://html.spec.whatwg.org/multipage/web-sockets.html)
- [RFC 6455 - The WebSocket Protocol](https://tools.ietf.org/html/rfc6455)


## Contributing

Please check out the [contribution guidelines](https://github.com/najamelan/ws_stream_wasm/blob/release/CONTRIBUTING.md).


### Testing

For testing we need back-end servers to echo data back to the tests. These are in the `ws_stream_tungstenite` crate.
```bash
git clone https://github.com/najamelan/ws_stream_tungstenite
cd ws_stream_tungstenite
cargo run --example echo --release

# in a different terminal:
cargo run --example echo_tt --release -- "127.0.0.1:3312"

# the second server is pure async-tungstenite without ws_stream_tungstenite wrapping it in AsyncRead/Write. This
# is needed for testing a WsMessage::Text because ws_stream_tungstenite only does binary.

# in a third terminal, in ws_stream_wasm you have different options:
wasm-pack test --firefox [--headless] [--release]
wasm-pack test --chrome  [--headless] [--release]
```

In general chrome is well faster. When running it in the browser (without `--headless`) you get trace logging
in the console, which helps debugging. In chrome you need to enable verbose output in the console,
otherwise only info and up level are reported.

### Code of conduct

Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](https://github.com/stumpsyn/policies/blob/master/citizen_code_of_conduct.md#4-unacceptable-behavior) are not welcome here and might get you banned. If anyone, including maintainers and moderators of the project, fail to respect these/your limits, you are entitled to call them out.

## License

[Unlicence](https://unlicense.org/)

