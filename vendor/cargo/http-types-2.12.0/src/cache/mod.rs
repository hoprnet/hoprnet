//! HTTP caching.
//!
//! Web page performance can be significantly improved by caching resources.
//! This submodule includes headers and types to communicate how and when to
//! cache resources.
//!
//! # Further Reading
//!
//! - [MDN: HTTP Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)

mod age;
mod cache_control;
mod clear_site_data;
mod expires;

pub use age::Age;
pub use cache_control::CacheControl;
pub use cache_control::CacheDirective;
pub use clear_site_data::{ClearDirective, ClearSiteData};
pub use expires::Expires;
