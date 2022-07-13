import type { PeerId } from '@libp2p/interface-peer-id'
import { unmarshalPublicKey } from '@libp2p/crypto/keys'

import secp256k1 from 'secp256k1'
import { Address, Hash } from './index.js'

import { SECP256K1_CONSTANTS } from '../crypto/index.js'
import { u8aToHex, u8aEquals, stringToU8a } from '../u8a/index.js'
import { pubKeyToPeerId } from '../libp2p/index.js'

export class HalfKeyChallenge {
  // @TODO use uncompressed point internally
  constructor(private arr: Uint8Array) {
    if (arr.length !== HalfKeyChallenge.SIZE) {
      throw new Error('Incorrect size Uint8Array for compressed curve point')
    }
  }

  static fromExponent(exponent: Uint8Array): HalfKeyChallenge {
    if (exponent.length !== SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH) {
      throw new Error('Incorrect size Uint8Array for private key')
    }

    return new HalfKeyChallenge(secp256k1.publicKeyCreate(exponent, true))
  }

  static fromUncompressedUncompressedCurvePoint(arr: Uint8Array) {
    if (arr.length !== 65) {
      throw new Error('Incorrect size Uint8Array for uncompressed public key')
    }

    return new HalfKeyChallenge(secp256k1.publicKeyConvert(arr, true))
  }

  static fromPeerId(peerId: PeerId) {
    return new HalfKeyChallenge(unmarshalPublicKey(peerId.publicKey).marshal())
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

  static fromString(str: string): HalfKeyChallenge {
    return new HalfKeyChallenge(stringToU8a(str))
  }

  static get SIZE(): number {
    return SECP256K1_CONSTANTS.COMPRESSED_PUBLIC_KEY_LENGTH
  }

  static deserialize(arr: Uint8Array) {
    return new HalfKeyChallenge(arr)
  }

  serialize() {
    return this.arr
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  clone(): HalfKeyChallenge {
    return new HalfKeyChallenge(this.arr.slice())
  }

  eq(b: HalfKeyChallenge) {
    return u8aEquals(this.arr, b.serialize())
  }
}
