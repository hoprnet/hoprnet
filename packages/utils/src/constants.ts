import BN from 'bn.js'

export const PRIVATE_KEY_LENGTH = 32
export const PUBLIC_KEY_LENGTH = 33
export const UNCOMPRESSED_PUBLIC_KEY_LENGTH = 66
export const ADDRESS_LENGTH = 20
export const HASH_LENGTH = 32
export const SECRET_LENGTH = 32
export const SIGNATURE_LENGTH = 64
export const SIGNATURE_RECOVERY_LENGTH = 1
// a multi address can have an arbitary length
export const MULTI_ADDR_MAX_LENGTH = 200

export const PRICE_PER_PACKET = new BN('10000000000000000') // 0.01 HOPR
// Must be a natural number, will be rounded to a natural number otherwise
export const INVERSE_TICKET_WIN_PROB = new BN('1') // 100%

export const MINIMUM_REASONABLE_CHANNEL_STAKE = new BN(PRICE_PER_PACKET).muln(100)

export const MAX_AUTO_CHANNELS = 5

// native balance (eth, xdai)
export const MIN_NATIVE_BALANCE = new BN('1000000000000000') // 0.001
export const SUGGESTED_NATIVE_BALANCE = MIN_NATIVE_BALANCE.muln(10) // 0.01

// balance (HOPR)
export const SUGGESTED_BALANCE = MINIMUM_REASONABLE_CHANNEL_STAKE.muln(MAX_AUTO_CHANNELS * 2) // enough to fund 10 channels
