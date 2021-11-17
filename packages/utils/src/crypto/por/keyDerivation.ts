import { expand } from 'futoin-hkdf'
import { privateKeyVerify } from 'secp256k1'
import { SECRET_LENGTH, HASH_ALGORITHM, HASH_LENGTH } from './constants'
import { HalfKey } from '../../types'
import { PRESECRET_LENGTH } from '../packet/constants'

const HASH_KEY_OWN_KEY = 'HASH_KEY_OWN_KEY'
const HASH_KEY_ACK_KEY = 'HASH_KEY_ACK_KEY'

const MAX_ITERATIONS = 1000

/**
 * Computes the key share derivable by the relayer
 * @param secret shared secret with the creator of the packet
 * @returns the key share
 */
export function deriveOwnKeyShare(secret: Uint8Array): HalfKey {
  if (secret.length != PRESECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return sampleFieldElement(secret, HASH_KEY_OWN_KEY)
}

/**
 * Computes the key share that is embedded in the acknowledgement
 * for a packet and thereby unlocks the incentive for the previous
 * relayer for transforming and delivering the packet
 * @param secret shared secret with the creator of the packet
 * @returns
 */
export function deriveAckKeyShare(secret: Uint8Array): HalfKey {
  if (secret.length != PRESECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return sampleFieldElement(secret, HASH_KEY_ACK_KEY)
}

/**
 * Samples a field element from a given seed using HKDF
 * If the result of HKDF does not lead to a field element,
 * the key identifier is padded until the key derivation
 * leads to a valid field element
 * @param secret the seed
 * @param _hashKey identifier used to derive the field element
 * @param __fakeExpand used for testing
 * @returns a field element
 */
export function sampleFieldElement(
  secret: Uint8Array,
  _hashKey: string,
  __fakeExpand?: (hashKey: string) => Uint8Array
): HalfKey {
  let result: Uint8Array
  let done = false
  let hashKey = _hashKey
  let i = 0

  do {
    result =
      __fakeExpand?.(hashKey) ??
      Uint8Array.from(expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), SECRET_LENGTH, hashKey))

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

  return new HalfKey(result)
}
