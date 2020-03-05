// @ts-ignore
import secp256k1 from 'secp256k1'
import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { Signature, Channel, ChannelBalance } from '.'
import { typedClass } from 'src/tsc/utils'
import { u8aConcat } from 'src/core/u8a'
import { Uint8Array } from 'src/types/extended'
import { sign, verify } from 'src/utils'
// TODO: check if this breaks, we should use `import type ..`
import type {HoprEthereumClass} from '..'

@typedClass<TypeConstructors['SignedChannel']>()
class SignedChannel extends Uint8Array {
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

  subarray(begin: number = 0, end: number = SignedChannel.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
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
    return secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery)
  }

  async verify(coreConnector: HoprEthereumClass) {
    return await verify(this.channel.toU8a(), this.signature, coreConnector.self.publicKey)
  }

  static get SIZE() {
    return Signature.SIZE + ChannelBalance.SIZE + 1
  }

  static async create(
    coreConnector: HoprEthereumClass,
    channel: Channel,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<SignedChannel> {
    const signature = await sign(channel.toU8a(), coreConnector.self.privateKey, coreConnector.self.publicKey)

    if (arr != null) {
      const signedChannel = new SignedChannel(arr)
      signedChannel.signature.set(signature, 0)
      signedChannel.set(channel.toU8a(), Signature.SIZE)

      return signedChannel
    }

    return new SignedChannel(undefined, {
      signature: signature as Signature,
      channel
    })
  }
}

export default SignedChannel
