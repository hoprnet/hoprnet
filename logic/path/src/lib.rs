//! This Rust crate contains all the path construction and path selection algorithms in the HOPR mixnet.

/// Defines the graph of HOPR payment channels.
pub mod channel_graph;
pub mod errors;
/// Defines the two most important types: [TransportPath](crate::path::TransportPath) and [ChannelPath](crate::path::ChannelPath).
pub mod path;
/// Implements different path selectors in the [ChannelGraph](crate::channel_graph::ChannelGraph).
pub mod selectors;
