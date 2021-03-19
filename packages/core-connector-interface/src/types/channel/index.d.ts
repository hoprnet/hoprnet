import { Moment, Hash, Signature } from '..'

import ChannelBalance from './balance'
import ChannelState from './state'

declare interface ChannelStatic {
  createFunded(channelBalance: ChannelBalance): Channel
  createActive(channelBalance: ChannelBalance): Channel
  createPending(pending: Moment, balance: ChannelBalance): Channel
  deserialize(arr: Uint8Array): Channel;
  SIZE: number
}

declare interface Channel {
  sign(
    privKey: Uint8Array,
    pubKey: Uint8Array | undefined
  ): Promise<Signature>

  balance: ChannelBalance

  pending?: Moment

  isFunded: boolean
  isActive: boolean
  isPending: boolean
  state: ChannelState

  hash(): Promise<Hash>

  serialize(): Uint8Array;
}

declare var Channel: ChannelStatic

export { Channel, ChannelBalance, ChannelState }
