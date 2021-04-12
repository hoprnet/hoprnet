/**
 * Length of secp256k1 private keys resp. public keys
 */
export const PRIVATE_KEY_LENGTH = 32
export const COMPRESSED_PUBLIC_KEY_LENGTH = 33
export const UNCOMPRESSED_PUBLIC_KEY_LENGTH = 65

/**
 * Hash algorithm that is used to derive the shared secrets
 * and its output length
 */
export const HASH_ALGORITHM = 'blake2s256'
export const HASH_LENGTH = 32

/**
 * Length of the shared secret that is derived from
 * the mixnet packet
 */
export const SECRET_LENGTH = HASH_LENGTH

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
