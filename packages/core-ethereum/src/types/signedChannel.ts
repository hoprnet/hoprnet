import type { Types } from '@hoprnet/hopr-core-connector-interface'
import Signature from './signature'
import Public from './public'
import { Channel } from './channel'
import { Uint8ArrayE } from '../types/extended'

class SignedChannel extends Uint8ArrayE implements Types.SignedChannel {
  private _channel?: Channel

  constructor(
    counterparty: Public,
    channel: Channel
  ) {
    super(SignedChannel.SIZE)
    this.set(channel.toU8a(), this.channelOffset - this.byteOffset)
    this.set(counterparty, this.signatureOffset - this.byteOffset)
  }

  slice(begin = 0, end = SignedChannel.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = SignedChannel.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
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

  get channelOffset(): number {
    return this.byteOffset + Signature.SIZE
  }

  get channel(): Channel {
    if (!this._channel) {
      this._channel = new Channel.deserialize({
        bytes: this.buffer,
        offset: this.channelOffset
      })
    }

    return this._channel
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
