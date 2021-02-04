import type { Types } from '@hoprnet/hopr-core-connector-interface'
import Signature from './signature'
import Public from './public'
import { Channel } from './channel'
import { Uint8ArrayE } from '../types/extended'

class SignedChannel extends Uint8ArrayE implements Types.SignedChannel {
  private _channel?: Channel

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      counterparty?: Public
      channel?: Channel
    }
  ) {
    if (!arr) {
      super(SignedChannel.SIZE)
    } else {
      super(arr.bytes, arr.offset, SignedChannel.SIZE)
    }

    if (struct) {
      if (struct.channel) {
        this.set(struct.channel.toU8a(), this.channelOffset - this.byteOffset)
      }

      if (struct.counterparty) {
        this.set(struct.counterparty, this.signatureOffset - this.byteOffset)
      }
    }
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
      this._channel = new Channel({
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
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature?: Signature
      channel?: Channel
    }
  ): Promise<SignedChannel> {
    return Promise.resolve(new SignedChannel(arr, struct))
  }
}

export default SignedChannel
