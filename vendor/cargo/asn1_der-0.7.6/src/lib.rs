#![doc = include_str!("../README.md")]
// Handle no_std if set
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
#[doc(hidden)]
pub mod error;
mod data;
#[doc(hidden)]
pub mod der;
#[cfg(feature = "native_types")]
pub mod typed;

// Reexport common types
#[cfg(all(feature = "std", not(feature = "no_panic")))]
pub use crate::data::VecBacking;
pub use crate::{
    data::{CopyingSource, CountingSource, Sink, SliceSink, Source},
    der::DerObject,
    error::{Asn1DerError, Asn1DerErrorVariant, ErrorChain},
};
