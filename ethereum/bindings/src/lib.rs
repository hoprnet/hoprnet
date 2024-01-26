#![allow(clippy::all)]
//! This lib re-exports abigen! generated bindings for solidity contracts.

#[cfg_attr(rustfmt, rustfmt_skip)]
mod codegen;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub use codegen::*;
