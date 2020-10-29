import {Types} from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'

import {u8aToNumber, toU8a} from '@hoprnet/hopr-utils'

class ChannelState extends Uint8Array implements Types.ChannelState {
  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      state: number
    }
  ) {
    if (arr == null) {
      super(ChannelState.SIZE)
    } else {
      super(arr.bytes, arr.offset, ChannelState.SIZE)
    }

    if (struct != null) {
      this.set(toU8a(struct.state, 1))
    }
  }

  toBN(): BN {
    return new BN(this)
  }

  // @TODO remove this
  toNumber(): number {
    return u8aToNumber(this)
  }

  static get SIZE() {
    return 1
  }
}

export default ChannelState
