#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

//! Utilities for working with hyper.
//!
//! This crate is less-stable than [`hyper`](https://docs.rs/hyper). However,
//! does respect Rust's semantic version regarding breaking changes.

#[cfg(feature = "client")]
pub mod client;
mod common;
pub mod rt;
#[cfg(feature = "server")]
pub mod server;
#[cfg(all(
    feature = "service",
    any(feature = "http1", feature = "http2"),
    any(feature = "server", feature = "client")
))]
pub mod service;

mod error;
