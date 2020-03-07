import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from 'src/tsc/utils'
import { u8aConcat, u8aToNumber } from 'src/core/u8a'
import { Uint8ArrayE } from 'src/types/extended'
import { SIGNATURE_LENGTH, SIGNATURE_RECOVERY_LENGTH } from 'src/constants'

@typedClass<TypeConstructors['Signature']>()
class Signature extends Uint8ArrayE {
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
    if (arr == null && struct != null) {
      super(u8aConcat(struct.signature, new Uint8Array([struct.recovery])))
    } else if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, SIGNATURE_LENGTH)
    } else {
      throw Error('Invalid constructor arguments.')
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
}

export default Signature
