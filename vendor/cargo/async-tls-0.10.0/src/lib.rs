//! Asynchronous TLS/SSL streams for async-std and AsyncRead/AsyncWrite sockets using [rustls](https://github.com/ctz/rustls).

#![deny(unsafe_code)]

#[cfg(feature = "server")]
mod acceptor;
#[cfg(feature = "client")]
pub mod client;
mod common;
#[cfg(feature = "client")]
mod connector;
mod rusttls;
#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "server")]
pub use acceptor::{Accept, TlsAcceptor};
#[cfg(feature = "client")]
pub use connector::{Connect, TlsConnector};

#[cfg(all(test, feature = "client", feature = "early-data"))]
mod test_0rtt;
