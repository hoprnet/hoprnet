import { publicKeyCreate, publicKeyVerify, privateKeyVerify, publicKeyConvert } from 'secp256k1'
import { randomFillSync } from 'crypto'
import { SECP256K1_CONSTANTS } from './constants'

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

    if (!privateKeyVerify(exponent)) {
      continue
    }

    groupElement = publicKeyCreate(exponent, false)
  } while (!publicKeyVerify(groupElement))

  if (compressed) {
    return [exponent, publicKeyConvert(groupElement)]
  } else {
    return [exponent, groupElement]
  }
}
