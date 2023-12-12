#![doc(html_root_url = "https://docs.rs/hyper-old-types/0.11.0")]
#![deny(missing_docs)]
#![cfg_attr(test, deny(warnings))]
#![deny(missing_debug_implementations)]
#![cfg_attr(all(test, feature = "nightly"), feature(test))]

//! # hyper-old-types

extern crate base64;
extern crate bytes;
#[cfg(feature = "compat")]
extern crate http;
extern crate httparse;
extern crate language_tags;
#[macro_use] extern crate log;
pub extern crate mime;
#[macro_use] extern crate percent_encoding;
extern crate time;
extern crate unicase;

#[cfg(all(test, feature = "nightly"))]
extern crate test;

pub use error::{Result, Error};
pub use header::Headers;
pub use method::Method::{self, Get, Head, Post, Put, Delete};
pub use status::StatusCode::{self, Ok, BadRequest, NotFound};
pub use uri::Uri;
pub use version::HttpVersion;

mod common;
pub mod error;
mod method;
pub mod header;
mod status;
mod uri;
mod version;
