import { expand } from 'futoin-hkdf'
import { privateKeyVerify } from 'secp256k1'
import { SECRET_LENGTH, HASH_ALGORITHM, HASH_LENGTH } from './constants'

const HASH_KEY_OWN_KEY = 'HASH_KEY_OWN_KEY'
const HASH_KEY_ACK_KEY = 'HASH_KEY_ACK_KEY'

const MAX_ITERATIONS = 1000

export function deriveOwnKeyShare(secret: Uint8Array) {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return sampleFieldElement(secret, HASH_KEY_OWN_KEY)
}

export function deriveAckKeyShare(secret: Uint8Array) {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return sampleFieldElement(secret, HASH_KEY_ACK_KEY)
}

export function sampleFieldElement(
  secret: Uint8Array,
  _hashKey: string,
  __fakeExpand?: (hashKey: string) => Uint8Array
): Uint8Array {
  let result: Uint8Array
  let done = false
  let hashKey = _hashKey
  let i = 0

  do {
    result = __fakeExpand?.(hashKey) ?? expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), SECRET_LENGTH, hashKey)

    if (!privateKeyVerify(result)) {
      if (i == MAX_ITERATIONS) {
        throw Error(`Cannot derive a group element.`)
      }

      hashKey += '_'
      i++
    } else {
      done = true
    }
  } while (!done)

  return result
}
