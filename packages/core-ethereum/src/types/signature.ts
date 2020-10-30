import type { Types } from '@hoprnet/hopr-core-connector-interface'
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
    if (arr == null) {
      super(Signature.SIZE)
    } else {
      super(arr.bytes, arr.offset, Signature.SIZE)
    }

    if (struct != null) {
      this.set(struct.signature, this.signatureOffset - this.byteOffset)
      this.set([struct.recovery], this.recoveryOffset - this.byteOffset)
    }
  }

  slice(begin = 0, end = Signature.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = Signature.SIZE) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get signatureOffset(): number {
    return this.byteOffset
  }
  get signature() {
    return new Uint8Array(this.buffer, this.signatureOffset, SIGNATURE_LENGTH)
  }

  get recoveryOffset(): number {
    return this.byteOffset + SIGNATURE_LENGTH
  }

  get recovery(): number {
    return this[this.recoveryOffset - this.byteOffset]
  }

  get msgPrefix(): Uint8Array {
    return new Uint8Array()
  }

  get onChainSignature(): Uint8Array {
    return this.signature
  }

  static get SIZE(): number {
    return SIGNATURE_LENGTH + SIGNATURE_RECOVERY_LENGTH
  }

  static create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature: Uint8Array
      recovery: number
    }
  ) {
    return Promise.resolve(new Signature(arr, struct))
  }
}

export default Signature
