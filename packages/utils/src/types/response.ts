import { Challenge } from '.'
import { u8aToHex } from '../u8a'
import { SECP256K1_CONSTANTS } from '../crypto'
import type { HalfKey } from '.'

import { publicKeyCreate, privateKeyTweakAdd } from 'secp256k1'

export class Response {
  constructor(private readonly arr: Uint8Array) {
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(arr)) {
      throw Error(`Expected a Uint8Array but got a Buffer`)
    }

    if (arr.length != Response.SIZE) {
      throw new Error('Incorrect size Uint8Array for hash')
    }
  }

  static fromHalfKeys(firstHalfKey: HalfKey, secondHalfKey: HalfKey): Response {
    return new Response(privateKeyTweakAdd(firstHalfKey.clone().serialize(), secondHalfKey.serialize()))
  }

  static deserialize(arr: Uint8Array) {
    return new Response(arr)
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  serialize(): Uint8Array {
    return this.arr
  }

  toChallenge(): Challenge {
    return new Challenge(publicKeyCreate(this.arr))
  }

  static SIZE = SECP256K1_CONSTANTS.PRIVATE_KEY_LENGTH
}
