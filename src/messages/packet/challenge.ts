import secp256k1 from 'secp256k1'
import PeerId from 'peer-id'

import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface'

import BN from 'bn.js'
import { u8aConcat } from '@hoprnet/hopr-utils'

const KEY_LENGTH = 32

/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an
 * inappropriate acknowledgement.
 */
export class Challenge<Chain extends HoprCoreConnector> extends Uint8Array {
  // private : Uint8Array
  private paymentChannels: Chain
  private _hashedKey: Uint8Array
  private _fee: BN
  private _counterparty: Uint8Array

  constructor(
    paymentChannels: Chain,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature: Types.Signature
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, Challenge.SIZE(paymentChannels))
    } else if (arr == null && struct != null) {
      super(struct.signature)
    } else {
      throw Error(`Invalid constructor parameters`)
    }

    this.paymentChannels = paymentChannels
  }

  get challengeSignature(): Types.Signature {
    return this.paymentChannels.types.Signature.create({
      bytes: this.buffer,
      offset: this.byteOffset,
    })
  }

  set challengeSignature(signature: Types.Signature) {
    this.set(signature, 0)
  }

  get signatureHash(): Promise<Types.Hash> {
    return this.paymentChannels.utils.hash(this.challengeSignature)
  }

  static SIZE<Chain extends HoprCoreConnector>(paymentChannels: Chain): number {
    return paymentChannels.types.Signature.SIZE
  }

  get hash(): Promise<Types.Hash> {
    if (this._hashedKey == null) {
      return Promise.reject(Error(`Challenge was not set yet.`))
    }
    return this.paymentChannels.utils.hash(this._hashedKey)
  }

  subarray(begin: number = 0, end: number = Challenge.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  getCopy(): Challenge<Chain> {
    const arrCopy = new Uint8Array(Challenge.SIZE(this.paymentChannels))

    arrCopy.set(this.subarray())

    return new Challenge<Chain>(this.paymentChannels, {
      bytes: arrCopy.buffer,
      offset: arrCopy.byteOffset,
    })
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

    return new Promise<Uint8Array>(async resolve => {
      resolve(
        secp256k1.ecdsaRecover(
          this.challengeSignature.signature,
          this.challengeSignature.recovery,
          await this.paymentChannels.utils.hash(u8aConcat(this.challengeSignature.msgPrefix, await this.hash))
        )
      )
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
  static create<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain, hashedKey: Uint8Array, fee: BN): Challenge<Chain> {
    if (hashedKey.length != KEY_LENGTH) {
      throw Error(`Invalid secret format. Expected a ${Uint8Array.name} of ${KEY_LENGTH} elements but got one with ${hashedKey.length}`)
    }

    const challenge = new Challenge(hoprCoreConnector, {
      bytes: new Uint8Array(Challenge.SIZE(hoprCoreConnector)).buffer,
      offset: 0,
    })

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
