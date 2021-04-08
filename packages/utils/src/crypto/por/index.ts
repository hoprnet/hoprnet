import { SECRET_LENGTH } from './constants'
import { publicKeyCreate, privateKeyTweakAdd } from 'secp256k1'
import { deriveAckKeyShareBlinding, deriveOwnKeyShareBlinding } from './keyDerivation'

export function create(secrects: Uint8Array[]) {
  if (secrects.length == 2 || secrects.some((s) => s.length != SECRET_LENGTH)) {
    throw Error(`Invalid arguments`)
  }

  const s0 = deriveOwnKeyShareBlinding(secrects[0])
  const s1 = deriveAckKeyShareBlinding(secrects[1])

  return publicKeyCreate(privateKeyTweakAdd(s0, s1))
}
