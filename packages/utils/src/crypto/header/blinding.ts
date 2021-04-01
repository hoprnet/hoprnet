import { expand } from 'futoin-hkdf'
import { PRIVATE_KEY_LENGTH, SECRET_LENGTH } from './constants'
import { PRG_IV_LENGTH, PRG_KEY_LENGTH } from '../prg'
import type { PRGParameters } from '../prg'

const HASH_KEY_BLINDING = 'HASH_KEY_BLINDING'
const HASH_KEY_PRG = 'HASH_KEY_PRG'

export function deriveBlinding(secret: Uint8Array): Uint8Array {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return expand('blake2s256', 32, Buffer.from(secret), PRIVATE_KEY_LENGTH, HASH_KEY_BLINDING)
}

export function derivePRGParameters(secret: Uint8Array): PRGParameters {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const rand = expand('blake2s256', 32, Buffer.from(secret), PRG_KEY_LENGTH + PRG_IV_LENGTH, HASH_KEY_PRG)

  return {
    iv: rand.subarray(0, PRG_IV_LENGTH),
    key: rand.subarray(PRG_IV_LENGTH, PRG_IV_LENGTH + PRG_KEY_LENGTH)
  }
}
