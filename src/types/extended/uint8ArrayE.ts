import { u8aToHex, u8aEquals } from '../../core/u8a'

class Uint8ArrayE extends Uint8Array {
  // TODO: verify jf it's correct
  subarray(begin: number = 0, end?: number) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end != null ? end - begin : undefined)
  }

  toU8a() {
    return new Uint8Array(this)
  }

  toHex() {
    return u8aToHex(this)
  }

  eq(b: Uint8Array) {
    return u8aEquals(this, b)
  }
}

export default Uint8ArrayE
