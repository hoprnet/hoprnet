//! HTTP Content headers.
//!
//! These headers are used for "content negotiation": the client shares information
//! about which content it prefers, and the server responds by sharing which
//! content it's chosen to share. This enables clients to receive resources with the
//! best available compression, in the preferred language, and more.
//!
//! # Further Reading
//!
//! - [MDN: Content Negotiation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Content_negotiation)
//!
//! # Examples
//!
//! ```
//! # fn main() -> http_types::Result<()> {
//! #
//! use http_types::content::{Accept, MediaTypeProposal};
//! use http_types::{mime, Response};
//!
//! let mut accept = Accept::new();
//! accept.push(MediaTypeProposal::new(mime::HTML, Some(0.8))?);
//! accept.push(MediaTypeProposal::new(mime::XML, Some(0.4))?);
//! accept.push(mime::PLAIN);
//!
//! let mut res = Response::new(200);
//! let content_type = accept.negotiate(&[mime::XML])?;
//! content_type.apply(&mut res);
//!
//! assert_eq!(res["Content-Type"], "application/xml;charset=utf-8");
//! #
//! # Ok(()) }
//! ```

pub mod accept;
pub mod accept_encoding;
pub mod content_encoding;

mod content_length;
mod content_location;
mod content_type;
mod encoding;
mod encoding_proposal;
mod media_type_proposal;

#[doc(inline)]
pub use accept::Accept;
#[doc(inline)]
pub use accept_encoding::AcceptEncoding;
#[doc(inline)]
pub use content_encoding::ContentEncoding;
pub use content_length::ContentLength;
pub use content_location::ContentLocation;
pub use content_type::ContentType;
pub use encoding::Encoding;
pub use encoding_proposal::EncodingProposal;
pub use media_type_proposal::MediaTypeProposal;
