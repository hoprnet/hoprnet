import type { Types } from '@hoprnet/hopr-core-connector-interface'
import Signature from './signature'
import Public from './public'
import { Channel } from './channel'

class SignedChannel implements Types.SignedChannel {
  constructor(
    readonly counterparty: Public,
    readonly channel: Channel
  ) {}

  static deserialize(arr: Uint8Array): SignedChannel {
    const channel = Channel.deserialize()
    return new SignedChannel(signature, channel)
  }

  serialize(): Uint8Array {
    serializeToU8a([
      [this.counterparty, Signature.SIZE]
      [this.channel.toU8a(), Channel.SIZE],
    ])
  }

  get signatureOffset(): number {
    return this.byteOffset
  }

  get signature(): Signature {
    return new Signature(undefined, {
      signature: new Uint8Array(),
      recovery: 0
    })
  }

  get signer(): Promise<Uint8Array> {
    return Promise.resolve(new Uint8Array(this.buffer, this.signatureOffset, Public.SIZE))
  }

  async verify(_publicKey: Uint8Array): Promise<boolean> {
    throw Error('SignedChannel does not implement verify')
  }

  static get SIZE(): number {
    return Signature.SIZE + Channel.SIZE
  }

  static create(
    signature: Signature,
    channel: Channel
  ): SignedChannel {
    return new SignedChannel(signature, channel)
  }
}

export default SignedChannel
