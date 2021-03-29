import { Moment, Hash, Signature } from '..'

import ChannelBalance from './balance'
import ChannelState from './state'

declare interface ChannelStatic {
  createActive(channelBalance: ChannelBalance): Channel

  createPending(pending: Moment, balance: ChannelBalance): Channel

  SIZE: number
}

declare interface Channel {
  toU8a(): Uint8Array

  sign(
    privKey: Uint8Array,
    pubKey: Uint8Array | undefined,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<Signature>

  balance: ChannelBalance

  pending?: Moment

  isActive: boolean
  isPending: boolean

  rawState: ChannelState

  // computed properties
  hash: Promise<Hash>
}

declare var Channel: ChannelStatic

export { Channel, ChannelBalance, ChannelState }
