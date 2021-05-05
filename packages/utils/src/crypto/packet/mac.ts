import { createHmac } from 'crypto'
import { HASH_ALGORITHM, HASH_LENGTH, SECRET_LENGTH } from './constants'
import { expand } from 'futoin-hkdf'

const HASH_KEY_HMAC = 'HASH_KEY_HMAC'

/**
 * Computes the authentication tag to make the integrity of
 * the packet header verifiable
 * @param secret shared secret with the creator of the packet
 * @param header the packet header
 * @returns the authentication tag
 */
export function createMAC(secret: Uint8Array, header: Uint8Array): Uint8Array {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const key = expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), SECRET_LENGTH, HASH_KEY_HMAC)

  return createHmac(HASH_ALGORITHM, key).update(header).digest()
}
