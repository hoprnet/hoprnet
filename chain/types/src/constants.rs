/// specifies in milliseconds for how long various
/// ethereum requests like `eth_getBalance` should be cached for
pub const PROVIDER_CACHE_TTL: u32 = 30_000; // 30 seconds

/// Time to wait for a confirmation before giving up
/// If the gas price is too low, the indexer would otherwise wait forever.
pub const TX_CONFIRMATION_WAIT: u32 = 60_000; // 60 seconds

/// Default initial block range used to query the RPC provider
/// e.g. starting to query logs of 2000 blocks, if that fails,
/// try with 1000 blocks etc.
pub const INDEXER_BLOCK_RANGE: u32 = 2000;

/// Time the indexer waits to confirm a transaction
pub const INDEXER_TIMEOUT: u32 = 900_000; // 15 minutes

/// Submitting a transaction get retried using an exponential backoff
/// The last try should not take longer than `MAX_TRANSACTION_BACKOFF`
pub const MAX_TRANSACTION_BACKOFF: u32 = 1_800_000; // 30 minutes
