import { Channel } from './channel'
import Signature from './signature'

declare interface SignedChannelStatic {
  readonly SIZE: number
  deserialize(arr: Uint8Array): SignedChannel;

  create(
    channel: Channel,
    signature: Signature
  ): SignedChannel
}

declare interface SignedChannel {
  channel: Channel
  signature: Signature
  signer: Promise<Uint8Array>
  signatureOffset: number
  channelOffset: number
  serialize(): Uint8Array;
  verify(pubKey: Uint8Array): Promise<boolean>
}

declare var SignedChannel: SignedChannelStatic
export default SignedChannel
