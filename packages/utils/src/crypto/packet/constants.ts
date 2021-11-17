/**
 * Hash algorithm that is used to derive the shared secrets
 * and its output length
 */
import { SECP256K1_CONSTANTS } from '../constants'

export const HASH_ALGORITHM = 'blake2s256'
export const HASH_LENGTH = 32

/**
 * Length of the shared secret that is derived from
 * the mixnet packet
 */
export const SECRET_LENGTH = HASH_LENGTH

/**
 * Length of the shared pre-secret which is the elliptic curve group element.
 */
export const PRESECRET_LENGTH = SECP256K1_CONSTANTS.UNCOMPRESSED_PUBLIC_KEY_LENGTH

/**
 * Length of the MAC as used for integrity protection
 * of mixnet packets
 */
export const MAC_LENGTH = HASH_LENGTH

/**
 * Size of the payload per packet
 */
export const PAYLOAD_SIZE = 500

/**
 * Prefix that signals a relayer that it is the final
 * recipient
 */
export const END_PREFIX = 0xff
export const END_PREFIX_LENGTH = 1

/**
 * Length of the tag used to prevent from replay attacks
 */
export const TAG_LENGTH = 16
