# ws_stream_wasm Changelog

## [Unreleased]

  [Unreleased]: https://github.com/najamelan/ws_stream_wasm/compare/release...dev


## [0.7.3] - 2021-06-11

  [0.7.3]: https://github.com/najamelan/ws_stream_wasm/compare/0.7.2...0.7.3

### Added

  - on user demand also make `WsStream` `Sync` again.


## [0.7.2] - 2021-06-26

  [0.7.2]: https://github.com/najamelan/ws_stream_wasm/compare/0.7.1...0.7.2

### Added

  - Make `WsStream` `Send` again.


## [0.7.1] - 2021-06-11

  [0.7.1]: https://github.com/najamelan/ws_stream_wasm/compare/0.7.0...0.7.1

### Updated
  - Switch to asynchronous-codec.
  - Remove external_doc feature for nightly 1.54.

## 0.7.0 - 2021-02-17

  - **BREAKING CHANGE**: Update to tokio v1, pharos to 0.5 and async_io_stream to 0.3.
  - **BREAKING CHANGE**: Browsers stopped raising SECURITY_ERR when trying to connect to a [forbidden port](https://stackoverflow.com/questions/4313403/why-do-browsers-block-some-ports/4314070). It now just returns a connection failed, which is indistinguishable from any other network problems, or simply the server not listening on this port. This is an [intended change](https://bugzilla.mozilla.org/show_bug.cgi?id=1684620).
  -


## 0.7.0-beta.1 - 2020-11-03

  - Update to tokio 0.3, will be out of beta when tokio hits 1.0.


## 0.6.1 - 2020-10-02

  - Remove unnecessary `mut` in recent compiler versions. Travis stable on osx is still on 1.44 and will fail until they upgrade.
  - improve readme

## 0.6.0 - 2020-03-21

  - **BREAKING CHANGE**: rename the basic types. `WsStream` is now called `WsMeta` and `WsIo` is now called `WsStream`.
  - **BREAKING CHANGE**: `WsStream` no longer implements `AsyncRead`/`AsyncWrite` directly, you have to call `into_io()`.
  - **BREAKING CHANGE**: The error type is now renamed to `WsErr` and is an enum directly instead of having an error kind.
  - **BREAKING CHANGE**: Fix: `From<MessageEvent> for WsMessage` has become `TryFrom`. This is because things actually could
    go wrong here.

  - Implement tokio `AsyncRead`/`AsyncWrite` for WsStream (Behind a feature flag).
  - delegate implementation of `AsyncRead`/`AsyncWrite`/`AsyncBufRead` to _async_io_stream_. This allows
    sharing the functionality with _ws_stream_tungstenite_, fleshing it out to always fill and use entire buffers,
    polling the underlying stream several times if needed.
  - only build for default target on docs.rs.
  - exclude unneeded files from package build.
  - remove trace and debug statements.
  - `WsMessage` now implements `From<Vec<u8>>` and `From<String>`.
  - `WsMeta` and `WsStream` are now `Send`. You should still only use them in a single thread though. This is fine because
    WASM has no threads, and is sometimes necessary because all the underlying types of _web-sys_ are `!Send`.
  - No longer set a close code if the user doesn't set one.
  - Fix: Make sure `WsStream` continues to function correctly if you drop `WsMeta`.


## 0.5.2 - 2020-01-06

  - fix version of futures-codec because they didn't bump their major version number after making a breaking change.


## 0.5.1 - 2019-11-14

  - update futures to 0.3.1.


## 0.5 - 2019-09-28

  - **BREAKING CHANGE**: update to pharos 0.4. Observable::observe is now fallible, so that is a breaking change for ws_stream_wasm
  - update to futures-codec 0.3


## 0.4.1 - 2019-09-23

  - fix some more errors in the readme

## 0.4.0 - 2019-09-23

  - **BREAKING CHANGE**: use the new filter feature from pharos, making `NextEvent` and `WsEventType` redundant. Those
    types have been removed from the library. The `observe` and method off `WsStream` now takes a `pharos::ObserveConfig` to filter event types. Please refer to the documentation of [pharos](https://docs.rs/pharos) for how to use them.
  - spell check all docs

## 0.3.0 - 2019-09-08

  - drop dependencies on async_runtime and failure and switch to std::error::Error for error handling
  - add a fullstack chat example (still needs documentation and cleanup)

## 0.2.1 - 2019-08-02

  - Fix incorrect link to changelog in readme


## 0.2.0 - 2019-08-02

  - **BREAKING CHANGE**: Fix: Correctly wake up tasks waiting for a next message if the connection gets closed externally.
    This prevents these tasks from hanging indefinitely.
    As a consequence, `WsStream::close` now returns a `Result`, taking into account that if the connection is already
    closed, we don't have the `CloseEvent`. Instead a `WsErr` of kind `WsErrKind::ConnectionNotOpen` is returned.
  - update to async_runtime 0.3
