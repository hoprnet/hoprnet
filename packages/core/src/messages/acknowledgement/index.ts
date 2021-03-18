import secp256k1 from 'secp256k1'

import { u8aConcat } from '@hoprnet/hopr-utils'
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
  private _responseSigningParty?: Uint8Array
  private _responseSignature?: Types.Signature
  private _hashedKey?: Uint8Array

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
    if (arr == null) {
      super(Acknowledgement.SIZE(paymentChannels))
    } else {
      super(arr.bytes, arr.offset, Acknowledgement.SIZE(paymentChannels))
    }

    if (struct != null) {
      if (struct.key != null) {
        this.set(struct.key, this.keyOffset - this.byteOffset)
      }

      if (struct.challenge != null) {
        this.set(struct.challenge, this.challengeOffset - this.byteOffset)
      }

      if (struct.signature != null) {
        this.set(struct.signature, this.responseSignatureOffset - this.byteOffset)
      }
    }

    this.paymentChannels = paymentChannels
  }

  slice(begin: number = 0, end: number = Acknowledgement.SIZE(this.paymentChannels)) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end: number = Acknowledgement.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get keyOffset(): number {
    return this.byteOffset
  }

  get key(): Uint8Array {
    return new Uint8Array(this.buffer, this.keyOffset, KEY_LENGTH)
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

  get challengeOffset(): number {
    return this.byteOffset + KEY_LENGTH
  }

  get challenge(): Challenge<Chain> {
    return new Challenge<Chain>(this.paymentChannels, {
      bytes: this.buffer,
      offset: this.challengeOffset
    })
  }

  get hash(): Promise<Uint8Array> {
    return this.paymentChannels.utils.hash(
      new Uint8Array(this.buffer, this.byteOffset, KEY_LENGTH + Challenge.SIZE(this.paymentChannels))
    )
  }

  get challengeSignatureHash(): Promise<Uint8Array> {
    return this.paymentChannels.utils.hash(this.challenge)
  }

  get challengeSigningParty() {
    return this.challenge.counterparty
  }

  get responseSignatureOffset(): number {
    return this.byteOffset + KEY_LENGTH + Challenge.SIZE(this.paymentChannels)
  }

  get responseSignature(): Promise<Types.Signature> {
    if (this._responseSignature != null) {
      return Promise.resolve(this._responseSignature)
    }

    return new Promise<Types.Signature>(async (resolve) => {
      this._responseSignature = await this.paymentChannels.types.Signature.create({
        bytes: this.buffer,
        offset: this.responseSignatureOffset
      })

      resolve(this._responseSignature)
    })
  }

  get responseSigningParty(): Promise<Uint8Array> {
    if (this._responseSigningParty != null) {
      return Promise.resolve(this._responseSigningParty)
    }

    return new Promise<Uint8Array>(async (resolve) => {
      const responseSignature = await this.responseSignature
      this._responseSigningParty = secp256k1.ecdsaRecover(
        responseSignature.signature,
        responseSignature.recovery,
        responseSignature.msgPrefix != null && responseSignature.msgPrefix.length > 0
          ? await this.paymentChannels.utils.hash(u8aConcat(responseSignature.msgPrefix, await this.hash))
          : await this.hash
      )

      resolve(this._responseSigningParty)
    })
  }

  async sign(peerId: PeerId): Promise<Acknowledgement<Chain>> {
    const signature = await this.paymentChannels.utils.sign(await this.hash, peerId.privKey.marshal())
    this.set(signature, this.responseSignatureOffset - this.byteOffset)
    return this
  }

  async verify(peerId: PeerId): Promise<boolean> {
    return this.paymentChannels.utils.verify(await this.hash, await this.responseSignature, peerId.pubKey.marshal())
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

    ack.set(deriveTicketKeyBlinding(derivedSecret), ack.keyOffset - ack.byteOffset)
    ack.set(challenge, ack.challengeOffset - ack.byteOffset)

    return ack.sign(signer)
  }

  static SIZE<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain): number {
    return KEY_LENGTH + Challenge.SIZE(hoprCoreConnector) + hoprCoreConnector.types.Signature.SIZE
  }
}

export { Acknowledgement }
