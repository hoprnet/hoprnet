import {Channel} from './channel'
import Signature from './signature'

declare interface SignedChannelStatic {
  readonly SIZE: number

  create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      channel?: Channel
      signature?: Signature
    }
  ): Promise<SignedChannel>
}

declare interface SignedChannel extends Uint8Array {
  channel: Channel
  signature: Signature
  signer: Promise<Uint8Array>

  signatureOffset: number
  channelOffset: number

  verify(pubKey: Uint8Array): Promise<boolean>
}

declare var SignedChannel: SignedChannelStatic

export default SignedChannel
