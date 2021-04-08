import { expand } from 'futoin-hkdf'
import { PRIVATE_KEY_LENGTH, SECRET_LENGTH, HASH_ALGORITHM, HASH_LENGTH } from './constants'
import { PRG_IV_LENGTH, PRG_KEY_LENGTH } from '../prg'
import { PRP_IV_LENGTH, PRP_KEY_LENGTH } from '../prp'
import type { PRGParameters } from '../prg'
import type { PRPParameters } from '../prp'

const HASH_KEY_BLINDING = 'HASH_KEY_BLINDING'
const HASH_KEY_PRG = 'HASH_KEY_PRG'
const HASH_KEY_PRP = 'HASH_KEY_PRP'

export function deriveBlinding(secret: Uint8Array): Uint8Array {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  return expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), PRIVATE_KEY_LENGTH, HASH_KEY_BLINDING)
}

export function derivePRGParameters(secret: Uint8Array): PRGParameters {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const rand = expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), PRG_KEY_LENGTH + PRG_IV_LENGTH, HASH_KEY_PRG)

  return {
    iv: rand.subarray(0, PRG_IV_LENGTH),
    key: rand.subarray(PRG_IV_LENGTH, PRG_IV_LENGTH + PRG_KEY_LENGTH)
  }
}

export function derivePRPParameters(secret: Uint8Array): PRPParameters {
  if (secret.length != SECRET_LENGTH) {
    throw Error(`Invalid arguments`)
  }

  const rand = expand(HASH_ALGORITHM, HASH_LENGTH, Buffer.from(secret), PRP_KEY_LENGTH + PRP_IV_LENGTH, HASH_KEY_PRP)

  const key = rand.subarray(0, PRP_KEY_LENGTH)
  const iv = rand.subarray(PRP_KEY_LENGTH)

  return { key, iv }
}
