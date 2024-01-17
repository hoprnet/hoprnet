//! Basic storage implementations for the Storage mechanism used by the Rust backend.

pub mod constants;
pub mod db;
pub mod errors;

#[cfg(feature = "js")]
pub mod rusty;
#[cfg(not(feature = "js"))]
pub mod sqlite;
pub mod traits;
pub use traits::KVStorage;

#[cfg(not(feature = "js"))]
pub type CurrentDbShim = sqlite::SqliteShim<'static>;

#[cfg(feature = "js")]
pub type CurrentDbShim = rusty::RustyLevelDbShim;
