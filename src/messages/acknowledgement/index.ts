import secp256k1 from 'secp256k1'

import { u8aConcat } from '../../utils'
import { deriveTicketKeyBlinding } from '../packet/header'
import { KEY_LENGTH } from '../packet/header/parameters'
import { Challenge } from '../packet/challenge'
import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'

/**
 * This class encapsulates the message that is sent back to the relayer
 * and allows that party to compute the key that is necessary to redeem
 * the previously received transaction.
 */
class Acknowledgement<Chain extends HoprCoreConnector> extends Uint8Array {
  private _responseSigningParty: Uint8Array
  private _hashedKey: Uint8Array

  private paymentChannels: Chain

  constructor(
    paymentChannels: Chain,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      key: Uint8Array
      challenge: Challenge<Chain>
      signature?: Types.Signature
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, Acknowledgement.SIZE(paymentChannels))
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.key, struct.challenge, struct.signature != null ? struct.signature : new Uint8Array(paymentChannels.types.Signature.SIZE)))
    } else {
      throw Error('Invalid constructor parameters.')
    }

    this.paymentChannels = paymentChannels
  }
  subarray(begin: number = 0, end: number = Acknowledgement.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get key(): Uint8Array {
    return this.subarray(0, KEY_LENGTH)
  }

  get hashedKey(): Promise<Uint8Array> {
    if (this._hashedKey != null) {
      return Promise.resolve(this._hashedKey)
    }

    return this.paymentChannels.utils.hash(this.key).then((hashedKey: Uint8Array) => {
      this._hashedKey = hashedKey

      return hashedKey
    })
  }

  get challenge(): Challenge<Chain> {
    return new Challenge<Chain>(this.paymentChannels, {
      bytes: this.buffer,
      offset: this.byteOffset + KEY_LENGTH
    })
  }

  get hash(): Promise<Uint8Array> {
    return this.paymentChannels.utils.hash(u8aConcat(this.challenge, this.key))
  }

  get challengeSignatureHash(): Promise<Uint8Array> {
    return this.paymentChannels.utils.hash(this.challenge)
  }

  get challengeSigningParty() {
    return this.challenge.counterparty
  }

  get responseSignature(): Types.Signature {
    return this.paymentChannels.types.Signature.create({
      bytes: this.buffer,
      offset: this.byteOffset + KEY_LENGTH + Challenge.SIZE(this.paymentChannels)
    })
  }

  get responseSigningParty(): Promise<Uint8Array> {
    if (this._responseSigningParty) {
      return Promise.resolve(this._responseSigningParty)
    }

    return new Promise<Uint8Array>(async resolve => {
      this._responseSigningParty = secp256k1.ecdsaRecover(this.responseSignature.signature, this.responseSignature.recovery, await this.hash)

      resolve(this._responseSigningParty)
    })
  }

  async sign(peerId: PeerId): Promise<Acknowledgement<Chain>> {
    this.responseSignature.set(await this.paymentChannels.utils.sign(await this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal()))

    return this
  }

  async verify(peerId: PeerId): Promise<boolean> {
    return this.paymentChannels.utils.verify(await this.hash, this.responseSignature, peerId.pubKey.marshal())
  }

  /**
   * Takes a challenge from a relayer and returns an acknowledgement that includes a
   * signature over the requested key half.
   *
   * @param challenge the signed challenge of the relayer
   * @param derivedSecret the secret that is used to create the second key half
   * @param signer contains private key
   */
  static async create<Chain extends HoprCoreConnector>(
    hoprCoreConnector: Chain,
    challenge: Challenge<Chain>,
    derivedSecret: Uint8Array,
    signer: PeerId
  ): Promise<Acknowledgement<Chain>> {
    const ack = new Acknowledgement(hoprCoreConnector, {
      bytes: new Uint8Array(Acknowledgement.SIZE(hoprCoreConnector)),
      offset: 0
    })

    ack.key.set(deriveTicketKeyBlinding(derivedSecret))

    ack.challenge.set(challenge)
    return ack.sign(signer)
  }

  static SIZE<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain): number {
    return KEY_LENGTH + Challenge.SIZE(hoprCoreConnector) + hoprCoreConnector.types.Signature.SIZE
  }
}

export { Acknowledgement }
