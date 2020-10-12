import { u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'

class Uint8ArrayE extends Uint8Array {
  toU8a() {
    return new Uint8Array(this)
  }

  toHex(prefixed?: boolean) {
    return u8aToHex(this, prefixed)
  }

  eq(b: Uint8Array) {
    return u8aEquals(this, b)
  }
}

export default Uint8ArrayE
