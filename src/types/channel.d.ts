import ChannelBalance from './channelBalance'
import Moment from './moment'
import Signature from './signature'

declare interface ChannelStatic {
  createFunded(channelBalance: ChannelBalance): Channel

  createActive(channelBalance: ChannelBalance): Channel

  createPending(pending: Moment, balance: ChannelBalance): Channel
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
}

declare var Channel: ChannelStatic

export default Channel
