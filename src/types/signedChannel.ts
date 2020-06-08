import secp256k1 from 'secp256k1'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aConcat } from '@hoprnet/hopr-utils'
import Signature from './signature'
import Channel from './channel'
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
      signature: Signature
      channel: Channel
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, SignedChannel.SIZE)
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.signature, struct.channel))
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get signature() {
    if (this._signature == null) {
      const signature = this.subarray(0, Signature.SIZE)

      this._signature = new Signature({
        bytes: signature.buffer,
        offset: signature.byteOffset,
      })
    }

    return this._signature
  }

  get channel() {
    if (this._channel == null) {
      const channel = this.subarray(Signature.SIZE, Signature.SIZE + Channel.SIZE)

      this._channel = new Channel({
        bytes: channel.buffer,
        offset: channel.byteOffset,
      })
    }

    return this._channel
  }

  get signer(): Promise<Uint8Array> {
    return this.channel.hash.then((channelHash) => {
      return secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, channelHash)
    })
  }

  async verify(publicKey: Uint8Array) {
    return await verify(this.channel.toU8a(), this.signature, publicKey)
  }

  static get SIZE() {
    return Signature.SIZE + Channel.SIZE
  }
}

export default SignedChannel
