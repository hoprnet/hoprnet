/**
 * specifies in milliseconds for how long various
 * ethereum requests like `eth_getBalance` should be cached for
 */
export const PROVIDER_CACHE_TTL = 30e3 // 30 seconds
export const CONFIRMATIONS = 8
export const TX_CONFIRMATION_WAIT = 60e3 // 30 seconds
export const INDEXER_BLOCK_RANGE = 2000
export const INDEXER_TIMEOUT = 900000 // 15 minutes
export const MAX_TRANSACTION_BACKOFF = 1800000 // 30 minutes
