//! # Common types for HTTP operations.
//!
//! `http-types` provides shared types for HTTP operations. It combines a performant, streaming
//! interface with convenient methods for creating headers, urls, and other standard HTTP types.
//!
//! # Example
//!
//! ```
//! # fn main() -> Result<(), http_types::url::ParseError> {
//! #
//! use http_types::{Method, Request, Response, StatusCode};
//!
//! let mut req = Request::new(Method::Get, "https://example.com");
//! req.set_body("Hello, Nori!");
//!
//! let mut res = Response::new(StatusCode::Ok);
//! res.set_body("Hello, Chashu!");
//! #
//! # Ok(()) }
//! ```
//!
//! # How does HTTP work?
//!
//! We couldn't possibly explain _all_ of HTTP here: there are [5 versions](enum.Version.html) of
//! the protocol now, and lots of extensions. But, at its core, there are only a few concepts you
//! need to know about in order to understand what this crate does.
//!
//! ```txt
//!          request
//! client ----------> server
//!        <----------
//!          response
//! ```
//!
//! HTTP is an [RPC protocol](https://en.wikipedia.org/wiki/Remote_procedure_call). A client
//! creates a [`Request`](struct.Request.html) containing a [`Url`](struct.Url.html),
//! [`Method`](struct.Method.html), [`Headers`](struct.Headers.html), and optional
//! [`Body`](struct.Body.html) and sends this to a server. The server then decodes this `Request`,
//! does some work, and sends back a [`Response`](struct.Response.html).
//!
//! The `Url` works as a way to subdivide an IP address/domain into further addressable resources.
//! The `Method` indicates what kind of operation we're trying to perform (get something, submit
//! something, update something, etc.)
//!
//! ```txt
//!   Request
//! |-----------------|
//! | Url             |
//! | Method          |
//! | Headers         |
//! |-----------------|
//! | Body (optional) |
//! |-----------------|
//! ```
//!
//! A `Response` consists of a [`StatusCode`](enum.StatusCode.html),
//! [`Headers`](struct.Headers.html), and optionally a [`Body`](struct.Body.html). The client then
//! decodes the `Response`, and can then operate on it. Usually the first thing it does is check
//! the status code to see if its `Request` was successful or not, and then moves on to the information contained within the headers.
//!
//! ```txt
//!      Response
//! |-----------------|
//! | StatusCode      |
//! | Headers         |
//! |-----------------|
//! | Body (optional) |
//! |-----------------|
//! ```
//!
//! Both `Request` and `Response` include [`Headers`](struct.Headers.html). This is like key-value metadata for HTTP
//! requests. It needs to be encoded in a specific way (all lowercase ASCII, only some special
//! characters) so we use the [`HeaderName`](headers/struct.HeaderName.html) and
//! [`HeaderValue`](headers/struct.HeaderValue.html) structs rather than strings to ensure that.
//! Another interesting thing about this is that it's valid to have multiple instances of the
//! same header name. This is why `Headers` allows inserting multiple values, and always returns a
//! [`Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) of headers for each key.
//!
//! When reading up on HTTP you might frequently hear a lot of jargon related to ther underlying
//! protocols. But even newer HTTP versions (`HTTP/2`, `HTTP/3`) still fundamentally use the
//! request/response model we've described so far.
//!
//! # The Body Type
//!
//! In HTTP, [`Body`](struct.Body.html) types are optional. The content of a `Body` is a stream of
//! bytes with a specific encoding; this encoding is its [`Mime` type](struct.Mime.html). The `Mime` can
//! be set using the [`set_content_type`](struct.Request.html#method.set_content_type) method, and
//! there are many different possible `Mime` types.
//!
//! `http-types`' `Body` struct can take anything that implements
//! [`AsyncBufRead`](https://docs.rs/futures/0.3.1/futures/io/trait.AsyncBufRead.html) and stream
//! it out. Depending on the version of HTTP used, the underlying bytes will be transmitted
//! differently. As a rule, if you know the size of the body it's usually more efficient to
//! declare it up front. But if you don't, things will still work.

#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs)]
#![allow(clippy::new_without_default)]
#![cfg_attr(backtrace, feature(backtrace))]
#![cfg_attr(feature = "docs", feature(doc_cfg))]
#![doc(html_favicon_url = "https://yoshuawuyts.com/assets/http-rs/favicon.ico")]
#![doc(html_logo_url = "https://yoshuawuyts.com/assets/http-rs/logo-rounded.png")]

/// HTTP cookies.
#[cfg(feature = "cookies")]
pub mod cookies {
    pub use cookie::*;
}

/// URL records.
pub mod url {
    pub use url::{
        EncodingOverride, Host, OpaqueOrigin, Origin, ParseError, ParseOptions, PathSegmentsMut,
        Position, SyntaxViolation, Url, UrlQuery,
    };
}

#[macro_use]
mod utils;

pub mod auth;
pub mod cache;
pub mod conditional;
pub mod content;
pub mod headers;
pub mod mime;
pub mod other;
pub mod proxies;
pub mod server;
pub mod trace;
pub mod transfer;
pub mod upgrade;

mod body;
mod error;
mod extensions;
mod macros;
mod method;
mod parse_utils;
mod request;
mod response;
mod status;
mod status_code;
mod version;

pub use body::Body;
pub use error::{Error, Result};
pub use method::Method;
pub use request::Request;
pub use response::Response;
pub use status::Status;
pub use status_code::StatusCode;
pub use version::Version;

#[doc(inline)]
pub use trailers::Trailers;

#[doc(inline)]
pub use mime::Mime;

#[doc(inline)]
pub use headers::Headers;

#[doc(inline)]
pub use crate::url::Url;

#[cfg(feature = "cookies")]
#[doc(inline)]
pub use crate::cookies::Cookie;

pub mod security;
pub mod trailers;

#[cfg(feature = "hyperium_http")]
mod hyperium_http;

#[doc(inline)]
pub use crate::extensions::Extensions;

/// Traits for conversions between types.
pub mod convert {
    pub use serde::{de::DeserializeOwned, Deserialize, Serialize};
    #[doc(inline)]
    pub use serde_json::json;
}

// Not public API. Referenced by macro-generated code.
#[doc(hidden)]
pub mod private {
    use crate::Error;
    pub use crate::StatusCode;
    use core::fmt::{Debug, Display};
    pub use core::result::Result::Err;

    pub fn new_adhoc<M>(message: M) -> Error
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::new_adhoc(message)
    }
}
