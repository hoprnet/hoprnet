//! Collection of useful macros.
//!
//! Includes macros for easyily implementing `SerHex`, as well as a number of
//! macros for defining useful triats on byte-array style newtypes
//! (e.g.; `Foo([u8;n])`).

#[macro_use]
pub mod misc;

#[macro_use]
pub mod hex;

#[macro_use]
mod local;
