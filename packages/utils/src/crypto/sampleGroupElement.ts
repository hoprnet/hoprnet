import { publicKeyCreate, publicKeyVerify, privateKeyVerify, publicKeyConvert } from 'secp256k1'
import { randomFillSync } from 'crypto'
import { SECP256K1_CONSTANTS } from './constants'

/**
 * Generates a secp256k1 keypair used for an
 * offline Diffie-Hellman key exchange.
 * @returns a secp256k1 keypair
 */
export function sampleGroupElement(compressed: boolean = false): [exponent: Uint8Array, groupElement: Uint8Array] {
  let privKey = new Uint8Array(SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH)
  let pubKey: Uint8Array

  do {
    randomFillSync(privKey, 0, SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH)

    if (!privateKeyVerify(privKey)) {
      continue
    }

    pubKey = publicKeyCreate(privKey, false)
  } while (!publicKeyVerify(pubKey))

  if (compressed) {
    return [privKey, publicKeyConvert(pubKey)]
  } else {
    return [privKey, pubKey]
  }
}
