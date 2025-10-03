//! The chain indexer package encapsulates utilities responsible for processing the
//! on-chain data.
//!
//! The processing pipeline uses the RPC endpoint to extract a stream of blocks with logs
//! with the [`block::Indexer`] continually processing the stream utilizing
//! spawned threads.
//!
//! The processing itself transforms the on-chain data into hoprd specific data, while also
//! triggering specific actions for each event, ensuring finality, storing the data in the
//! local storage, and further reactively triggering higher level components of the business
//! logic.
//!
//! # Fast Synchronization
//!
//! The indexer supports fast synchronization through pre-built logs database snapshots.
//! The [`snapshot`] module provides secure download, extraction, and installation
//! of compressed database snapshots, allowing nodes to quickly catch up with the
//! blockchain state instead of fetching all historical logs from genesis from an RPC endpoint.

pub mod block;
pub mod config;
pub mod constants;
pub mod errors;
pub mod handlers;
pub mod snapshot;
pub mod traits;

/// Configuration for the chain indexer functionality.
///
/// Includes settings for fast synchronization and snapshot downloads.
pub use config::IndexerConfig;
