//! HTTP timings and traces.
//!
//! This module implements parsers and serializers for timing-related headers.
//! These headers enable tracing and timing requests, and help answer the
//! question of: _"Where is my program spending its time?"_
//!
//! # Specifications
//!
//! - [W3C Trace-Context header](https://w3c.github.io/trace-context/)
//! - [W3C Server-Timing header](https://w3c.github.io/server-timing/#the-server-timing-header-field)

pub mod server_timing;
mod trace_context;

#[doc(inline)]
pub use server_timing::{Metric, ServerTiming};
pub use trace_context::TraceContext;
