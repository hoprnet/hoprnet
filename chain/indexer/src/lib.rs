//! The chain indexer package encapsulates utilities responsible for processing the
//! on-chain data.
//!
//! The processing pipeline uses the RPC endpoint to extract a stream of blocks with logs
//! with the [`block::Indexer`] continually processing the stream utilizing
//! spawned threads.
//!
//! The processing itself transforms the on-chain data into hoprd specific data, while also
//! triggering specific actions for each event, ensuring finality, storing the data in the
//! local storage and further reactively triggering higher level components of the business
//! logic.

pub mod block;
pub mod config;
pub mod constants;
pub mod errors;
pub mod handlers;
pub mod traits;

pub use config::IndexerConfig;
