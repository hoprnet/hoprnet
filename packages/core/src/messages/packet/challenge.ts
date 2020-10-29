import secp256k1 from 'secp256k1'
import PeerId from 'peer-id'

import HoprCoreConnector, {Types} from '@hoprnet/hopr-core-connector-interface'

import BN from 'bn.js'
import {u8aConcat} from '@hoprnet/hopr-utils'

const KEY_LENGTH = 32

/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an
 * inappropriate acknowledgement.
 */
class Challenge<Chain extends HoprCoreConnector> extends Uint8Array {
  // private : Uint8Array
  private paymentChannels: Chain
  private _hashedKey: Uint8Array
  private _fee: BN
  private _counterparty: Uint8Array
  private _challengeSignature: Types.Signature

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
    if (arr == null) {
      super(Challenge.SIZE(paymentChannels))
    } else {
      super(arr.bytes, arr.offset, Challenge.SIZE(paymentChannels))
    }

    if (struct != null) {
      super(struct.signature)
    }

    this.paymentChannels = paymentChannels
  }

  get challengeSignatureOffset(): number {
    return this.byteOffset
  }

  get challengeSignature(): Promise<Types.Signature> {
    if (this._challengeSignature != null) {
      return Promise.resolve(this._challengeSignature)
    }

    return this.paymentChannels.types.Signature.create({
      bytes: this.buffer,
      offset: this.challengeSignatureOffset
    })
  }

  get signatureHash(): Promise<Types.Hash> {
    return new Promise(async (resolve) => {
      resolve(await this.paymentChannels.utils.hash(await this.challengeSignature))
    })
  }

  static SIZE<Chain extends HoprCoreConnector>(paymentChannels: Chain): number {
    return paymentChannels.types.Signature.SIZE
  }

  get hash(): Types.Hash {
    if (this._hashedKey == null) {
      throw Error(`Challenge was not set yet.`)
    }

    return this._hashedKey
  }

  subarray(begin: number = 0, end: number = Challenge.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  getCopy(): Challenge<Chain> {
    const arrCopy = new Uint8Array(Challenge.SIZE(this.paymentChannels))

    arrCopy.set(this)

    const copiedChallenge = new Challenge<Chain>(this.paymentChannels, {
      bytes: arrCopy.buffer,
      offset: arrCopy.byteOffset
    })

    copiedChallenge._hashedKey = this._hashedKey
    copiedChallenge._fee = this._fee

    return copiedChallenge
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

    return new Promise<Uint8Array>(async (resolve) => {
      const challengeSignature = await this.challengeSignature
      resolve(
        secp256k1.ecdsaRecover(
          challengeSignature.signature,
          challengeSignature.recovery,
          challengeSignature.msgPrefix != null && challengeSignature.msgPrefix.length > 0
            ? await this.paymentChannels.utils.hash(u8aConcat(challengeSignature.msgPrefix, await this.hash))
            : this.hash
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
    await this.paymentChannels.utils.sign(this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal(), {
      bytes: this.buffer,
      offset: this.challengeSignatureOffset
    })

    return this
  }

  /**
   * Creates a challenge object.
   *
   * @param hashedKey that is used to generate the key half
   * @param fee
   */
  static create<Chain extends HoprCoreConnector>(
    hoprCoreConnector: Chain,
    hashedKey: Uint8Array,
    fee: BN,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Challenge<Chain> {
    if (hashedKey.length != KEY_LENGTH) {
      throw Error(
        `Invalid secret format. Expected a ${Uint8Array.name} of ${KEY_LENGTH} elements but got one with ${hashedKey.length}`
      )
    }

    if (arr == null) {
      const tmp = new Uint8Array(this.SIZE(hoprCoreConnector))

      arr = {
        bytes: tmp.buffer,
        offset: tmp.byteOffset
      }
    }

    const challenge = new Challenge(hoprCoreConnector, arr)

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

    return this.paymentChannels.utils.verify(this.hash, await this.challengeSignature, peerId.pubKey.marshal())
  }
}

export {Challenge}
