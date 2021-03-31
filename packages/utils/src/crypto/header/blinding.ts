import { expand } from 'futoin-hkdf'
import { PRIVATE_KEY_LENGTH, SECRET_LENGTH } from './constants'

const HASH_KEY_BLINDING = 'HASH_KEY_BLINDING'

export function deriveBlinding(secret: Uint8Array): Uint8Array {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return expand('blake2s256', 32, Buffer.from(secret), PRIVATE_KEY_LENGTH, HASH_KEY_BLINDING)
}
