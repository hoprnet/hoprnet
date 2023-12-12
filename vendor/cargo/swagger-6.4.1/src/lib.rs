//! Support crate for Swagger codegen.
//!
//! # Crate features
//!
//! Crate features exist to reduce the dependencies on the crate. Most features
//! should be enabled by the generator when relevant.
//!
//! By default, the **serdejson** feature is enabled.
//! 
//! ## Format support
//!
//! - **multipart_form** - Enable support for `multipart/form-data` as described in RFC 7578
//! - **multipart_related** - Enable support for `multipart/related` as described in RFC 2387
//! - **serdejson** - Enable JSON serialization/deserialization support using serde.
//!
//! ## Feature support
//!
//! - **serdevalid** - Enable support for JSON schema based validation
//! - **conversion** - Enable support for Frunk-based conversion - in particular,
//!   [transmogrification](https://docs.rs/frunk/latest/frunk/#transmogrifying)
//!
//! ## Use case support
//!
//! - **client** - Enable support for providing an OpenAPI client
//! - **server** - Enable support for providing an OpenAPI server
//! - **http1** - Enable support for HTTP/1 based APIs - RFC 9112
//! - **http2** - Enable support for HTTP/2 based APIs - RFC 9113
//! - **tcp** - Enable support for HTTP over TCP
//! - **tls** - Enable support for HTTP over TLS (HTTPS)
//! - **uds** - Enable support for HTTP over UDS (Unix Domain Sockets)

#![deny(
    missing_docs,
    missing_debug_implementations,
    unused_extern_crates,
    unused_qualifications
)]
// Enable doc_auto_cfg, but only on doc builds
// See https://github.com/rust-lang/rust/issues/43781 for details
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use std::error;
use std::fmt;

/// Module for encoding API properties in base64.
pub mod base64_format;
pub use base64_format::ByteArray;

/// Module for encoding Nullable properties.
pub mod nullable_format;
pub use nullable_format::Nullable;

mod body;
pub use body::BodyExt;

pub mod auth;
pub use auth::{AuthData, Authorization};

pub mod context;
pub use context::{ContextBuilder, ContextWrapper, EmptyContext, Has, Pop, Push};

/// Module with utilities for creating connectors with hyper.
#[cfg(feature = "client")]
pub mod connector;
#[cfg(feature = "client")]
pub use connector::Connector;

#[cfg(all(feature = "server", any(feature = "http1", feature = "http2")))]
pub mod composites;
#[cfg(all(feature = "server", any(feature = "http1", feature = "http2")))]
pub use composites::{CompositeMakeService, CompositeMakeServiceEntry, CompositeService, NotFound};

pub mod add_context;
pub use add_context::{AddContextMakeService, AddContextService};

pub mod drop_context;
pub use drop_context::{DropContextMakeService, DropContextService};

pub mod request_parser;
pub use request_parser::RequestParser;

mod header;
pub use header::{XSpanIdString, X_SPAN_ID};

pub mod multipart;

mod one_any_of;
pub use one_any_of::*;

/// Helper Bound for Errors for MakeService/Service wrappers
pub trait ErrorBound: Into<Box<dyn std::error::Error + Send + Sync>> {}

impl<T> ErrorBound for T where T: Into<Box<dyn std::error::Error + Send + Sync>> {}

/// Very simple error type - just holds a description of the error. This is useful for human
/// diagnosis and troubleshooting, but not for applications to parse. The justification for this
/// is to deny applications visibility into the communication layer, forcing the application code
/// to act solely on the logical responses that the API provides, promoting abstraction in the
/// application code.
#[derive(Clone, Debug)]
pub struct ApiError(pub String);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug: &dyn fmt::Debug = self;
        debug.fmt(f)
    }
}

impl error::Error for ApiError {
    fn description(&self) -> &str {
        "Failed to produce a valid response."
    }
}
