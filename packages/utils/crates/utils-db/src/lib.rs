pub mod constants;
pub mod db;
pub mod errors;
#[cfg(feature = "hashmap")]
pub mod hashmap;
#[cfg(feature = "leveldb")]
pub mod rusty;
#[cfg(feature = "sqlite")]
pub mod sqlite;
pub mod traits;
pub mod types;
pub use traits::KVStorage;

#[cfg(feature = "sqlite")]
pub type CurrentDbShim = sqlite::SqliteShim<'static>;

#[cfg(feature = "leveldb")]
pub type CurrentDbShim = rusty::RustyLevelDbShim;