import { publicKeyCreate, publicKeyVerify } from 'secp256k1'
import { randomFillSync } from 'crypto'
import { PRIVATE_KEY_LENGTH } from './constants'

export function generateKeyPair(): [privKey: Uint8Array, pubKey: Uint8Array] {
  let privKey = new Uint8Array(32)
  let pubKey: Uint8Array

  do {
    randomFillSync(privKey, 0, PRIVATE_KEY_LENGTH)
    pubKey = publicKeyCreate(privKey, false)
  } while (!publicKeyVerify(pubKey))

  return [privKey, pubKey]
}
