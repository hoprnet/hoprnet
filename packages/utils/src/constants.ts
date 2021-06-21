import BN from 'bn.js'

export const PRIVATE_KEY_LENGTH = 32
export const PUBLIC_KEY_LENGTH = 33
export const UNCOMPRESSED_PUBLIC_KEY_LENGTH = 66
export const ADDRESS_LENGTH = 20
export const HASH_LENGTH = 32
export const SIGNATURE_LENGTH = 64
export const SIGNATURE_RECOVERY_LENGTH = 1
// a multi address can have an arbitary length
export const MULTI_ADDR_MAX_LENGTH = 200

export const PRICE_PER_PACKET = '10000000000000000' // 0.01 HOPR
// Must be a natural number, will be rounded to a natural number otherwise
export const INVERSE_TICKET_WIN_PROB = '1' // 100%

export const MINIMUM_REASONABLE_CHANNEL_STAKE = new BN(PRICE_PER_PACKET).muln(100)


export const MAX_AUTO_CHANNELS = 5
export const MIN_NATIVE_BALANCE = new BN('1000000000000000') // 0.001 ETH

export const SUGGESTED_NATIVE_BALANCE = MIN_NATIVE_BALANCE.muln(250) // 0.025 ETH

// enough to fund 10 channels
export const SUGGESTED_BALANCE = MINIMUM_REASONABLE_CHANNEL_STAKE.muln(MAX_AUTO_CHANNELS * 2)
