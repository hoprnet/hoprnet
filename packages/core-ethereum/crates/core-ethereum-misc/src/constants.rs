/// Number of blockchain block to wait until an on-chain
/// state-change is considered to be final
///
/// Note that the probability that on-chain state changes will get pruned due to
/// block reorganizations increases exponentially in the number of confirmations, e.g.
/// after one block it is `0.5` whereas after two blocks it is `0.25 = 0.5^2`  etc.
pub const DEFAULT_CONFIRMATIONS: u32 = 8;

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

#[cfg(feature = "wasm")]
pub mod wasm {
    // Need load as wasm_bindgen to make field annotations work
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct CoreEthereumConstants {
        #[wasm_bindgen(readonly, js_name = "DEFAULT_CONFIRMATIONS")]
        pub default_confirmations: u32,
        #[wasm_bindgen(readonly, js_name = "PROVIDER_CACHE_TTL")]
        pub provider_cache_ttl: u32,
        #[wasm_bindgen(readonly, js_name = "TX_CONFIRMATION_WAIT")]
        pub tx_confirmation_wait: u32,
        #[wasm_bindgen(readonly, js_name = "INDEXER_BLOCK_RANGE")]
        pub indexer_block_range: u32,
        #[wasm_bindgen(readonly, js_name = "INDEXER_TIMEOUT")]
        pub indexer_timeout: u32,
        #[wasm_bindgen(readonly, js_name = "MAX_TRANSACTION_BACKOFF")]
        pub max_transaction_backoff: u32,
    }

    /// Returns a struct with readonly constants, needs to be a function
    /// because Rust does not support exporting constants to WASM
    #[wasm_bindgen(js_name = "CORE_ETHEREUM_CONSTANTS")]
    pub fn get_constants() -> CoreEthereumConstants {
        CoreEthereumConstants {
            default_confirmations: super::DEFAULT_CONFIRMATIONS,
            provider_cache_ttl: super::PROVIDER_CACHE_TTL,
            indexer_block_range: super::INDEXER_BLOCK_RANGE,
            indexer_timeout: super::INDEXER_TIMEOUT,
            max_transaction_backoff: super::MAX_TRANSACTION_BACKOFF,
            tx_confirmation_wait: super::TX_CONFIRMATION_WAIT,
        }
    }
}
