import { expand } from 'futoin-hkdf'
import { HASH_ALGORITHM, HASH_LENGTH, TAG_LENGTH, PRESECRET_LENGTH } from './constants'
import { SECP256K1_CONSTANTS } from '../constants'
import { PRG_IV_LENGTH, PRG_KEY_LENGTH } from '../prg'
import { PRP_IV_LENGTH, PRP_KEY_LENGTH } from '../prp'
import type { PRGParameters } from '../prg'
import type { PRPParameters } from '../prp'

const HASH_KEY_BLINDING = 'HASH_KEY_BLINDING'
const HASH_KEY_PRG = 'HASH_KEY_PRG'
const HASH_KEY_PRP = 'HASH_KEY_PRP'
const HASH_KEY_PACKET_TAG = 'HASH_KEY_PACKET_TAG'

/**
 * Derive the blinding that is applied to the group element
 * before forwarding the packet
 * @param secret shared secret with the creator of the packet
 * @returns the blinding
 */
export function deriveBlinding(secret: Uint8Array): Uint8Array {
  if (secret.length != PRESECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return expand(
    HASH_ALGORITHM,
    HASH_LENGTH,
    Buffer.from(secret),
    SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH,
    HASH_KEY_BLINDING
  )
}

/**
 * Derive the seed for the pseudo-randomness generator
 * by using the secret shared derived from the mixnet packet
 * @param secret shared secret with the creator of the packet
 * @returns the PRG seed
 */
export function derivePRGParameters(secret: Uint8Array): PRGParameters {
  if (secret.length != PRESECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const rand = expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), PRG_KEY_LENGTH + PRG_IV_LENGTH, HASH_KEY_PRG)

  return {
    iv: rand.subarray(0, PRG_IV_LENGTH),
    key: rand.subarray(PRG_IV_LENGTH, PRG_IV_LENGTH + PRG_KEY_LENGTH)
  }
}

/**
 * Derive the seed for the pseudo-random permutation
 * by using the secret shared with the creator of the packet
 * @param secret shared secret with the creator of the packet
 * @returns
 */
export function derivePRPParameters(secret: Uint8Array): PRPParameters {
  if (secret.length != PRESECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const rand = expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), PRP_KEY_LENGTH + PRP_IV_LENGTH, HASH_KEY_PRP)

  const key = rand.subarray(0, PRP_KEY_LENGTH)
  const iv = rand.subarray(PRP_KEY_LENGTH)

  return { key, iv }
}

export function derivePacketTag(secret: Uint8Array): Uint8Array {
  if (secret.length != PRESECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), TAG_LENGTH, HASH_KEY_PACKET_TAG)
}
