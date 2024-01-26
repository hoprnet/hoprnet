//! Asynchronous byte buffers and pipes for concurrent I/O programming.
//!
//! ## Pipes
//!
//! The primary feature offered by Sluice are _pipes_, which are asynchronous
//! in-memory byte buffers that allow separate tasks to read and write from the
//! buffer in parallel.
//!
//! See the `pipe` module for details.

#![deny(unsafe_code)]
#![warn(
    future_incompatible,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    unused,
    clippy::all,
)]

pub mod pipe;
