//! # hyper-tls
//!
//! An HTTPS connector to be used with [hyper][].
//!
//! [hyper]: https://hyper.rs
//!
//! ## Example
//!
//! ```no_run
//! extern crate futures;
//! extern crate hyper;
//! extern crate hyper_tls;
//! extern crate tokio;
//!
//! use futures::{future, Future};
//! use hyper_tls::HttpsConnector;
//! use hyper::Client;
//!
//! fn main() {
//!     tokio::run(future::lazy(|| {
//!         // 4 is number of blocking DNS threads
//!         let https = HttpsConnector::new(4).unwrap();
//!         let client = Client::builder()
//!             .build::<_, hyper::Body>(https);
//!
//!         client
//!             .get("https://hyper.rs".parse().unwrap())
//!             .map(|res| {
//!                 assert_eq!(res.status(), 200);
//!             })
//!             .map_err(|e| println!("request error: {}", e))
//!     }));
//! }
//! ```
#![doc(html_root_url = "https://docs.rs/hyper-tls/0.3.2")]
#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

extern crate bytes;
extern crate futures;
extern crate hyper;
extern crate native_tls;
#[macro_use]
extern crate tokio_io;

pub use client::{HttpsConnector, HttpsConnecting, Error};
pub use stream::{MaybeHttpsStream, TlsStream};

mod client;
mod stream;
