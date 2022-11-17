//! An asynchronous, HTTP/2.0 server and client implementation.
//!
//! This library implements the [HTTP/2.0] specification. The implementation is
//! asynchronous, using [futures] as the basis for the API. The implementation
//! is also decoupled from TCP or TLS details. The user must handle ALPN and
//! HTTP/1.1 upgrades themselves.
//!
//! # Getting started
//!
//! Add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! h2 = "0.1"
//! ```
//!
//! Next, add this to your crate:
//!
//! ```no_run
//! extern crate h2;
//! ```
//!
//! # Layout
//!
//! The crate is split into [`client`] and [`server`] modules. Types that are
//! common to both clients and servers are located at the root of the crate.
//!
//! See module level documentation for more details on how to use `h2`.
//!
//! # Handshake
//!
//! Both the client and the server require a connection to already be in a state
//! ready to start the HTTP/2.0 handshake. This library does not provide
//! facilities to do this.
//!
//! There are three ways to reach an appropriate state to start the HTTP/2.0
//! handshake.
//!
//! * Opening an HTTP/1.1 connection and performing an [upgrade].
//! * Opening a connection with TLS and use ALPN to negotiate the protocol.
//! * Open a connection with prior knowledge, i.e. both the client and the
//!   server assume that the connection is immediately ready to start the
//!   HTTP/2.0 handshake once opened.
//!
//! Once the connection is ready to start the HTTP/2.0 handshake, it can be
//! passed to [`server::handshake`] or [`client::handshake`]. At this point, the
//! library will start the handshake process, which consists of:
//!
//! * The client sends the connection preface (a predefined sequence of 24
//! octets).
//! * Both the client and the server sending a SETTINGS frame.
//!
//! See the [Starting HTTP/2] in the specification for more details.
//!
//! # Flow control
//!
//! [Flow control] is a fundamental feature of HTTP/2.0. The `h2` library
//! exposes flow control to the user.
//!
//! An HTTP/2.0 client or server may not send unlimited data to the peer. When a
//! stream is initiated, both the client and the server are provided with an
//! initial window size for that stream.  A window size is the number of bytes
//! the endpoint can send to the peer. At any point in time, the peer may
//! increase this window size by sending a `WINDOW_UPDATE` frame. Once a client
//! or server has sent data filling the window for a stream, no further data may
//! be sent on that stream until the peer increases the window.
//!
//! There is also a **connection level** window governing data sent across all
//! streams.
//!
//! Managing flow control for inbound data is done through [`ReleaseCapacity`].
//! Managing flow control for outbound data is done through [`SendStream`]. See
//! the struct level documentation for those two types for more details.
//!
//! [HTTP/2.0]: https://http2.github.io/
//! [futures]: https://docs.rs/futures/
//! [`client`]: client/index.html
//! [`server`]: server/index.html
//! [Flow control]: http://httpwg.org/specs/rfc7540.html#FlowControl
//! [`ReleaseCapacity`]: struct.ReleaseCapacity.html
//! [`SendStream`]: struct.SendStream.html
//! [Starting HTTP/2]: http://httpwg.org/specs/rfc7540.html#starting
//! [upgrade]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Protocol_upgrade_mechanism
//! [`server::handshake`]: server/fn.handshake.html
//! [`client::handshake`]: client/fn.handshake.html

#![doc(html_root_url = "https://docs.rs/h2/0.1.26")]
#![deny(missing_debug_implementations, missing_docs)]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate futures;

#[macro_use]
extern crate tokio_io;

// HTTP types
extern crate http;

// Buffer utilities
extern crate bytes;

// Hash function used for HPACK encoding and tracking stream states.
extern crate fnv;

extern crate byteorder;
extern crate slab;

#[macro_use]
extern crate log;
extern crate string;
extern crate indexmap;

macro_rules! proto_err {
    (conn: $($msg:tt)+) => {
        debug!("connection error PROTOCOL_ERROR -- {};", format_args!($($msg)+))
    };
    (stream: $($msg:tt)+) => {
        debug!("stream error PROTOCOL_ERROR -- {};", format_args!($($msg)+))
    };
}

mod error;
#[cfg_attr(feature = "unstable", allow(missing_docs))]
mod codec;
mod hpack;
mod proto;

#[cfg(not(feature = "unstable"))]
mod frame;

#[cfg(feature = "unstable")]
#[allow(missing_docs)]
pub mod frame;

pub mod client;
pub mod server;
mod share;

pub use error::{Error, Reason};
pub use share::{SendStream, StreamId, RecvStream, ReleaseCapacity, PingPong, Ping, Pong};

#[cfg(feature = "unstable")]
pub use codec::{Codec, RecvError, SendError, UserError};
