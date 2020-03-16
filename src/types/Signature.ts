import type { Types } from "@hoprnet/hopr-core-connector-interface"
import { u8aConcat, u8aToNumber } from '../core/u8a'
import { Uint8ArrayE } from '../types/extended'
import { SIGNATURE_LENGTH, SIGNATURE_RECOVERY_LENGTH } from '../constants'

class Signature extends Uint8ArrayE implements Types.Signature {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature: Uint8Array
      recovery: number
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, Signature.SIZE)
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.signature, new Uint8Array([struct.recovery])))
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get signature() {
    return this.subarray(0, SIGNATURE_LENGTH)
  }

  get recovery() {
    return u8aToNumber(this.subarray(SIGNATURE_LENGTH, SIGNATURE_LENGTH + SIGNATURE_RECOVERY_LENGTH))
  }

  get msgPrefix() {
    return new Uint8Array()
  }

  get onChainSignature() {
    return this.signature
  }

  static get SIZE() {
    return SIGNATURE_LENGTH + SIGNATURE_RECOVERY_LENGTH
  }

  static create(    arr?: {
    bytes: ArrayBuffer
    offset: number
  },
  struct?: {
    signature: Uint8Array
    recovery: number
  }) {
    return new Signature(arr, struct)
  }
}

export default Signature
