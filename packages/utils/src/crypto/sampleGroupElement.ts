import secp256k1 from 'secp256k1'
import { randomFillSync } from 'crypto'
import { SECP256K1_CONSTANTS } from './constants.js'

/**
 * Samples a valid exponent and returns the exponent
 * and the product of exponent and base-point.
 * @dev can be used to derive a secp256k1 keypair
 * @returns [ exponent, groupElement]
 */
export function sampleGroupElement(compressed: boolean = false): [exponent: Uint8Array, groupElement: Uint8Array] {
  let exponent = new Uint8Array(SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH)
  let groupElement: Uint8Array

  do {
    randomFillSync(exponent, 0, SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH)

    if (!secp256k1.privateKeyVerify(exponent)) {
      continue
    }

    groupElement = secp256k1.publicKeyCreate(exponent, false)
  } while (!secp256k1.publicKeyVerify(groupElement))

  if (compressed) {
    return [exponent, secp256k1.publicKeyConvert(groupElement)]
  } else {
    return [exponent, groupElement]
  }
}
