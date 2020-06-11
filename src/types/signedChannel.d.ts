import Channel from './channel'
import Signature from './signature'

declare namespace SignedChannel {
  const SIZE: number

  function create(
    arr?: {
      bytes: Uint8Array
      offset: number
    },
    struct?: {
      ticket: Channel
      signature: Signature
    }
  ): Promise<SignedChannel>
}

declare interface SignedChannel extends Uint8Array {
  channel: Channel
  signature: Signature
  signer: Promise<Uint8Array>

  verify(pubKey: Uint8Array): Promise<boolean>
}

export default SignedChannel
