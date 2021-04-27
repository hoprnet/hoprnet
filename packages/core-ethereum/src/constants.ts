export const NETWORK = 'testnet'
export const CHAIN_NAME = 'ethereum'
export const ERRORS = {
  OOF_HOPR: 'Insufficient HOPR tokens.',
  OOF_NATIVE: 'Insufficient NATIVE funds.'
}
/**
 * specifies in milliseconds for how long various
 * ethereum requests like `eth_getBalance` should be cached for
 */
export const PROVIDER_CACHE_TTL = 30e3 // 30 seconds
export const DEFAULT_URI = 'ws://127.0.0.1:8545/'
export const MAX_CONFIRMATIONS = 8
export const INDEXER_BLOCK_RANGE = 2000
