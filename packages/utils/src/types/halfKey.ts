import { publicKeyCreate } from 'secp256k1'
import { u8aToHex, u8aEquals } from '../u8a'
import { HalfKeyChallenge } from '.'

export class HalfKey {
  constructor(private readonly arr: Uint8Array) {
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(arr)) {
      throw Error(`Expected a Uint8Array but got a Buffer`)
    }

    if (arr.length != HalfKey.SIZE) {
      throw new Error('Incorrect size Uint8Array for hash')
    }
  }

  toChallenge(): HalfKeyChallenge {
    return new HalfKeyChallenge(publicKeyCreate(this.arr))
  }

  serialize(): Uint8Array {
    return this.arr
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  eq(halfKey: HalfKey): boolean {
    return u8aEquals(this.arr, halfKey.arr)
  }

  static deserialize(arr: Uint8Array) {
    return new HalfKey(arr)
  }

  clone(): HalfKey {
    // create a new underlying buffer
    return new HalfKey(this.arr.slice())
  }

  static SIZE = 32
}
