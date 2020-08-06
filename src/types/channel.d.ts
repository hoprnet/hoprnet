import ChannelBalance from './channelBalance'
import Moment from './moment'
import Signature from './signature'

declare namespace Channel {
  function createFunded(channelBalance: ChannelBalance): Channel

  function createActive(channelBalance: ChannelBalance): Channel

  function createPending(pending: Moment, balance: ChannelBalance): Channel
}
declare interface Channel {
  toU8a(): Uint8Array

  sign(
    privKey: Uint8Array,
    pubKey: Uint8Array,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<Signature>
}

export default Channel
