import secp256k1 from 'secp256k1'
import type { PeerId } from '@libp2p/interface-peer-id'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'

import { u8aToHex, u8aEquals, stringToU8a } from '../u8a/index.js'
import { SECP256K1_CONSTANTS } from '../crypto/index.js'
import { pubKeyToPeerId } from '../libp2p/index.js'
import { Address, Hash } from './index.js'

export class CurvePoint {
  // @TODO use uncompressed point internally
  constructor(private arr: Uint8Array) {
    if (arr.length !== CurvePoint.SIZE) {
      throw new Error('Incorrect size Uint8Array for compressed curve point')
    }
  }

  static fromExponent(exponent: Uint8Array): CurvePoint {
    if (exponent.length !== SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH) {
      throw new Error('Incorrect size Uint8Array for private key')
    }

    return new CurvePoint(secp256k1.publicKeyCreate(exponent, true))
  }

  static fromUncompressedUncompressedCurvePoint(arr: Uint8Array) {
    if (arr.length !== 65) {
      throw new Error('Incorrect size Uint8Array for uncompressed public key')
    }

    return new CurvePoint(secp256k1.publicKeyConvert(arr, true))
  }

  static fromPeerId(peerId: PeerId) {
    return new CurvePoint(unmarshalPublicKey(peerId.publicKey).marshal())
  }

  toAddress(): Address {
    return new Address(Hash.create(secp256k1.publicKeyConvert(this.arr, false).slice(1)).serialize().slice(12))
  }

  toUncompressedCurvePoint(): string {
    // Needed in only a few cases for interacting with secp256k1
    return u8aToHex(secp256k1.publicKeyConvert(this.arr, false).slice(1))
  }

  toPeerId(): PeerId {
    return pubKeyToPeerId(this.serialize())
  }

  static fromString(str: string): CurvePoint {
    return new CurvePoint(stringToU8a(str))
  }

  static get SIZE(): number {
    return SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  eq(b: CurvePoint) {
    return u8aEquals(this.arr, b.serialize())
  }
}
