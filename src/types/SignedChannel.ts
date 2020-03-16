import type { Types } from "@hoprnet/hopr-core-connector-interface"
import secp256k1 from 'secp256k1'
import { Signature, Channel } from '.'
import { u8aConcat, u8aEquals } from '../core/u8a'
import { Uint8ArrayE } from '../types/extended'
import { AccountId } from '../types'
import { sign, verify, hashSync } from '../utils'
import HoprEthereum from '..'

class SignedChannel extends Uint8ArrayE implements Types.SignedChannel<Channel, Signature> {
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
        offset: signature.byteOffset
      })
    }

    return this._signature
  }

  get channel() {
    if (this._channel == null) {
      const channel = this.subarray(Signature.SIZE, Signature.SIZE + Channel.SIZE)

      this._channel = new Channel({
        bytes: channel.buffer,
        offset: channel.byteOffset
      })
    }

    return this._channel
  }

  get signer() {
    return new AccountId(
      secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, hashSync(this.channel.toU8a()))
    )
  }

  async verify(coreConnector: HoprEthereum) {
    return await verify(this.channel.toU8a(), this.signature, coreConnector.self.publicKey)
  }

  static get SIZE() {
    return Signature.SIZE + Channel.SIZE
  }

  static async create(
    coreConnector: HoprEthereum,
    arr?: {
      bytes: ArrayBuffer,
      offset: number
    }, struct?: {
      channel: Channel,
      signature?: Signature
    }
  ): Promise<SignedChannel> {
    let signedChannel: SignedChannel
    if (arr != null && struct == null) {
      signedChannel = new SignedChannel(arr)

      if (u8aEquals(signedChannel.signature, new Uint8Array(Signature.SIZE).fill(0x00))) {
        signedChannel.set(await sign(signedChannel.channel.toU8a(), coreConnector.self.privateKey), 0)
      }
    } else if (arr == null && struct != null) {
      const array = new Uint8Array(SignedChannel.SIZE).fill(0x00)
      signedChannel = new SignedChannel({
        bytes: array.buffer,
        offset: array.byteOffset
      })

      signedChannel.set(struct.channel.toU8a(), Signature.SIZE)

      if (struct.signature == null || u8aEquals(struct.signature, new Uint8Array(Signature.SIZE).fill(0x00))) {
        signedChannel.signature.set(await sign(signedChannel.channel.toU8a(), coreConnector.self.privateKey), 0)
      }

      if (struct.signature != null) {
        signedChannel.set(struct.signature, 0)
      }
    } else if (arr != null && struct != null) {
      signedChannel = new SignedChannel(arr)

      if (struct.channel != null) {
        if (!u8aEquals(signedChannel.channel.toU8a(), new Uint8Array(signedChannel.channel.toU8a().length).fill(0x00)) && !signedChannel.channel.eq(struct.channel)) {
          throw Error(`Argument mismatch. Please make sure the encoded channel in the array is the same as the one given throug struct.`)
        }

        signedChannel.set(struct.channel.toU8a(), Signature.SIZE)
      }

      if (struct.signature != null) {
        signedChannel.set(struct.signature, 0)
      } else {
        signedChannel.signature.set(await sign(signedChannel.channel.toU8a(), coreConnector.self.privateKey), 0)
      }
    } else {
      throw Error(`Invalid input parameters.`)
    }

    return signedChannel
  }
}

export default SignedChannel
