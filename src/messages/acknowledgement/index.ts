import secp256k1 from 'secp256k1'

import { u8aConcat } from '../../utils'
import { deriveTicketKeyBlinding } from '../packet/header'
import { KEY_LENGTH } from '../packet/header/parameters'
import { Challenge } from '../packet/challenge'
import { HoprCoreConnectorInstance, Types } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'

/**
 * This class encapsulates the message that is sent back to the relayer
 * and allows that party to compute the key that is necessary to redeem
 * the previously received transaction.
 */
class Acknowledgement<Chain extends HoprCoreConnectorInstance> extends Uint8Array {
  private _responseSigningParty: Uint8Array
  private paymentChannels: Chain

  constructor(
    paymentChannels: Chain,
    arr?: Uint8Array,
    struct?: {
      key: Uint8Array
      challenge: Challenge<Chain>
      signature?: Types.Signature
    }
  ) {
    if (arr != null && struct == null) {
      super(arr)
    } else if (arr == null && struct != null) {
      super(u8aConcat(struct.key, struct.challenge, struct.signature != null ? struct.signature : new Uint8Array(paymentChannels.types.Signature.SIZE)))
    } else {
      throw Error('Invalid constructor parameters.')
    }

    this.paymentChannels = paymentChannels
  }

  subarray(begin: number = 0, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end != null ? end - begin : undefined)
  }

  get key(): Uint8Array {
    return this.subarray(0, KEY_LENGTH)
  }

  set key(newKey: Uint8Array) {
    this.set(newKey, 0)
  }

  get hashedKey(): Uint8Array {
    return secp256k1.publicKeyCreate(Buffer.from(this.key))
  }

  get challenge(): Challenge<Chain> {
    return new Challenge<Chain>(this.paymentChannels, {
      bytes: this.buffer,
      offset: KEY_LENGTH
    })
  }

  set challenge(challenge: Challenge<Chain>) {
    this.set(challenge, KEY_LENGTH)
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
    return new this.paymentChannels.types.Signature({
      bytes: this.buffer,
      offset: this.byteOffset + KEY_LENGTH + Challenge.SIZE(this.paymentChannels)
    })
  }

  set responseSignature(newSignature: Types.Signature) {
    this.set(newSignature, KEY_LENGTH + Challenge.SIZE(this.paymentChannels))
  }

  get responseSigningParty(): Uint8Array {
    if (this._responseSigningParty) {
      return this._responseSigningParty
    }

    this._responseSigningParty = secp256k1.recover(
      // @ts-ignore
      Buffer.from(this.responseSignature.sr25519PublicKey),
      Buffer.from(this.responseSignature.signature),
      this.responseSignature.recovery
    )

    return this._responseSigningParty
  }

  async sign(peerId: PeerId): Promise<Acknowledgement<Chain>> {
    this.responseSignature = await this.paymentChannels.utils.sign(await this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal())

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
  static async create<Chain extends HoprCoreConnectorInstance>(
    hoprCoreConnector: Chain,
    challenge: Challenge<Chain>,
    derivedSecret: Uint8Array,
    signer: PeerId
  ): Promise<Acknowledgement<Chain>> {
    const ack = new Acknowledgement(hoprCoreConnector, new Uint8Array(Acknowledgement.SIZE(hoprCoreConnector)))

    ack.key = deriveTicketKeyBlinding(derivedSecret)

    ack.challenge = challenge
    return ack.sign(signer)
  }

  static SIZE<Chain extends HoprCoreConnectorInstance>(hoprCoreConnector: Chain): number {
    return KEY_LENGTH + Challenge.SIZE(hoprCoreConnector) + hoprCoreConnector.types.Signature.SIZE
  }
}

export { Acknowledgement }
