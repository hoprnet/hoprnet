import { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'

import { u8aToNumber, toU8a } from '@hoprnet/hopr-utils'

class ChannelState extends Uint8Array implements Types.ChannelState {
  constructor(
      state: number
  ) {
    super(ChannelState.SIZE)
    this.set(toU8a(state, 1))
  }

  toBN(): BN {
    return new BN(this)
  }

  // @TODO remove this
  toNumber(): number {
    return u8aToNumber(this) as number
  }

  static get SIZE() {
    return 1
  }
}

export default ChannelState
