//! HOPR daemon application providing a higher level interface for creating a HOPRd with or without
//! a dedicated REST API.
//!
//! When the Rest API is enabled, the node serves a Swagger UI to inspect and test
//! the Rest API v4 at: http://localhost:3001/scalar or http://localhost:3001/swagger-ui
//!
//! NOTE: Hostname and port can be different, since they depend on the settings `--apiHost` and `--apiPort`.
//!
//! ## Usage
//! See `hoprd --help` for full list.

pub mod cli;
pub mod config;
pub mod errors;
pub mod exit;
pub mod gen_test;
