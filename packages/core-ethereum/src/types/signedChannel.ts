import secp256k1 from 'secp256k1'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import Signature from './signature'
import { Channel } from './channel'
import { Uint8ArrayE } from '../types/extended'
import { verify } from '../utils'

class SignedChannel extends Uint8ArrayE implements Types.SignedChannel {
  private _signature?: Signature
  private _channel?: Channel

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature?: Signature
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

      if (struct.signature) {
        this.set(struct.signature, this.signatureOffset - this.byteOffset)
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
    if (!this._signature) {
      this._signature = new Signature({
        bytes: this.buffer,
        offset: this.signatureOffset
      })
    }

    return this._signature
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

  get signer(): Promise<Uint8Array> {
    return new Promise<Uint8Array>(async (resolve, reject) => {
      try {
        resolve(secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, await this.channel.hash))
      } catch (err) {
        reject(err)
      }
    })
  }

  async verify(publicKey: Uint8Array): Promise<boolean> {
    return await verify(await this.channel.hash, this.signature, publicKey)
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
