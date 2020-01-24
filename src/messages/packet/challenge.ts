import secp256k1 from 'secp256k1'

import { toU8a } from '../../utils'
import PeerId from 'peer-id'

import { HoprCoreConnectorInstance, Types } from '@hoprnet/hopr-core-connector-interface'

import BN from 'bn.js'

const COMPRESSED_PUBLIC_KEY_LENGTH = 33

/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an
 * inappropriate acknowledgement.
 */
export class Challenge<Chain extends HoprCoreConnectorInstance> extends Uint8Array {
  // private : Uint8Array
  private paymentChannels: Chain
  private _hashedKey: Uint8Array
  private _fee: BN
  private _counterparty: Uint8Array

  constructor(
    paymentChannels: Chain,
    buf?: Uint8Array,
    struct?: {
      signature: Types.Signature
    }
  ) {
    if (buf != null && struct == null) {
      super(buf)
    } else if (buf == null && struct != null) {
      super(struct.signature)
    } else {
      throw Error(`Invalid constructor parameters`)
    }

    this.paymentChannels = paymentChannels
  }

  get challengeSignature(): Types.Signature {
    return new this.paymentChannels.types.Signature(this.subarray(0, this.paymentChannels.types.Signature.SIZE))
  }

  set challengeSignature(signature: Types.Signature) {
    this.set(signature, 0)
  }

  get signatureHash(): Promise<Types.Hash> {
    return this.paymentChannels.utils.hash(this.challengeSignature)
  }

  static SIZE<Chain extends HoprCoreConnectorInstance>(paymentChannels: Chain): number {
    return paymentChannels.types.Signature.SIZE
  }

  get hash(): Promise<Types.Hash> {
    if (this._hashedKey == null) {
      return Promise.reject(Error(`Challenge was not set yet.`))
    }
    return this.paymentChannels.utils.hash(this._hashedKey)
  }

  subarray(begin?: number, end?: number): Uint8Array {
    return new Uint8Array(this.buffer, begin, end != null ? end - begin : undefined)
  }

  /**
   * Uses the derived secret and the signature to recover the public
   * key of the signer.
   */
  get counterparty(): Promise<Uint8Array> {
    if (this._hashedKey == null) {
      return Promise.reject(Error(`Challenge was not set yet.`))
    }

    if (this._counterparty != null) {
      return Promise.resolve(this._counterparty)
    }

    return this.hash.then((hash: Uint8Array) => {
      // @ts-ignore
      return secp256k1.recover(Buffer.from(this.challengeSignature.sr25519PublicKey), Buffer.from(this.challengeSignature.signature), this.challengeSignature.recovery)
    })
  }

  /**
   * Signs the challenge and includes the transferred amount of money as
   * well as the ethereum address of the signer into the signature.
   *
   * @param peerId that contains private key and public key of the node
   */
  async sign(peerId: PeerId): Promise<Challenge<Chain>> {
    // const hashedChallenge = hash(Buffer.concat([this._hashedKey, this._fee.toBuffer('be', VALUE_LENGTH)], HASH_LENGTH + VALUE_LENGTH))
    const signature = await this.paymentChannels.utils.sign(await this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal())

    this.challengeSignature = signature

    return this
  }

  /**
   * Creates a challenge object.
   *
   * @param hashedKey that is used to generate the key half
   * @param fee
   */
  static create<Chain extends HoprCoreConnectorInstance>(hoprCoreConnector: Chain, hashedKey: Uint8Array, fee: BN): Challenge<Chain> {
    if (hashedKey.length != COMPRESSED_PUBLIC_KEY_LENGTH) {
      throw Error(`Invalid secret format. Expected a ${Uint8Array.name} of ${COMPRESSED_PUBLIC_KEY_LENGTH} elements but got one with ${hashedKey.length}`)
    }

    const challenge = new Challenge(hoprCoreConnector, new Uint8Array(Challenge.SIZE(hoprCoreConnector)))
    challenge._hashedKey = hashedKey
    challenge._fee = fee

    return challenge
  }

  /**
   * Verifies the challenge by checking whether the given public matches the
   * one restored from the signature.
   *
   * @param peerId PeerId instance that contains the public key of
   * the signer
   * @param secret the secret that was used to derive the key half
   */
  async verify(peerId: PeerId): Promise<boolean> {
    if (!peerId.pubKey) {
      throw Error('Unable to verify challenge without a public key.')
    }

    return this.paymentChannels.utils.verify(await this.hash, this.challengeSignature, peerId.pubKey.marshal())
  }
}
