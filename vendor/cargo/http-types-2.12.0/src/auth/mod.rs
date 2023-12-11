//! HTTP authentication and authorization.
//!
//! # Examples
//!
//! ```
//! # fn main() -> http_types::Result<()> {
//! #
//! use http_types::Response;
//! use http_types::auth::{AuthenticationScheme, BasicAuth};
//!
//! let username = "nori";
//! let password = "secret_fish!!";
//! let authz = BasicAuth::new(username, password);
//!
//! let mut res = Response::new(200);
//! authz.apply(&mut res);
//!
//! let authz = BasicAuth::from_headers(res)?.unwrap();
//!
//! assert_eq!(authz.username(), username);
//! assert_eq!(authz.password(), password);
//! #
//! # Ok(()) }
//! ```

mod authentication_scheme;
mod authorization;
mod basic_auth;
mod www_authenticate;

pub use authentication_scheme::AuthenticationScheme;
pub use authorization::Authorization;
pub use basic_auth::BasicAuth;
pub use www_authenticate::WwwAuthenticate;
