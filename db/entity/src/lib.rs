#![allow(clippy::all)]
//! This lib re-exports SeaORM generated bindings for HOPR DB.

#[cfg_attr(rustfmt, rustfmt_skip)]
mod codegen;

pub mod conversions;

pub mod errors;

#[cfg(feature = "sqlite")]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub use codegen::sqlite::*;
