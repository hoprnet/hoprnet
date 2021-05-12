import { u8aEquals, u8aToHex } from '../u8a'

export class EthereumChallenge {
  constructor(private readonly arr: Uint8Array) {
    if (typeof Buffer !== 'undefined' && Buffer.isBuffer(arr)) {
      throw Error(`Expected a Uint8Array but got a Buffer`)
    }

    if (arr.length != EthereumChallenge.SIZE) {
      throw new Error('Incorrect size Uint8Array for hash')
    }
  }

  static deserialize(arr: Uint8Array) {
    return new EthereumChallenge(arr)
  }

  serialize(): Uint8Array {
    return this.arr
  }

  toHex(): string {
    return u8aToHex(this.arr)
  }

  eq(ethCallenge: EthereumChallenge): boolean {
    return u8aEquals(this.arr, ethCallenge.arr)
  }

  static SIZE = 20
}
