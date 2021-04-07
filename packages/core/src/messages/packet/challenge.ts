import secp256k1 from 'secp256k1'
import PeerId from 'peer-id'
import { Signature, Hash, PublicKey } from '@hoprnet/hopr-core-ethereum'

import BN from 'bn.js'

/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an
 * inappropriate acknowledgement.
 */
class Challenge extends Uint8Array {
  // private : Uint8Array
  private _hashedKey: Hash
  private _fee: BN
  private _counterparty: Uint8Array

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature: Signature
    }
  ) {
    if (arr == null) {
      super(Challenge.SIZE())
    } else {
      super(arr.bytes, arr.offset, Challenge.SIZE())
    }

    if (struct != null) {
      super(struct.signature.serialize())
    }
  }

  serialize(): Uint8Array {
    return this // TODO
  }

  static deserialize(arr: Uint8Array) {
    return new Challenge({ bytes: arr.buffer, offset: arr.byteOffset }) // TODO
  }

  get challengeSignatureOffset(): number {
    return this.byteOffset
  }

  get challengeSignature(): Signature {
    return Signature.deserialize(new Uint8Array(this.buffer, this.challengeSignatureOffset, Signature.SIZE))
  }

  get signatureHash(): Promise<Hash> {
    return new Promise(async (resolve) => {
      resolve(Hash.create(this.challengeSignature.serialize()))
    })
  }

  static SIZE(): number {
    return Signature.SIZE
  }

  get hash(): Hash {
    if (this._hashedKey == null) {
      throw Error(`Challenge was not set yet.`)
    }

    return this._hashedKey
  }

  subarray(begin: number = 0, end: number = Challenge.SIZE()): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  getCopy(): Challenge {
    const arrCopy = new Uint8Array(Challenge.SIZE())

    arrCopy.set(this)

    const copiedChallenge = new Challenge({
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
      resolve(secp256k1.ecdsaRecover(challengeSignature.signature, challengeSignature.recovery, this.hash.serialize()))
    })
  }

  /**
   * Signs the challenge and includes the transferred amount of money as
   * well as the ethereum address of the signer into the signature.
   *
   * @param peerId that contains private key and public key of the node
   */
  async sign(peerId: PeerId): Promise<Challenge> {
    const signature = Signature.create(this.hash.serialize(), peerId.privKey.marshal())
    this.set(signature.serialize(), this.challengeSignatureOffset - this.byteOffset)
    return this
  }

  /**
   * Creates a challenge object.
   *
   * @param hashedKey that is used to generate the key half
   * @param fee
   */
  static create(
    hashedKey: Hash,
    fee: BN,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Challenge {
    if (arr == null) {
      const tmp = new Uint8Array(this.SIZE())

      arr = {
        bytes: tmp.buffer,
        offset: tmp.byteOffset
      }
    }

    const challenge = new Challenge(arr)

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
    return this.challengeSignature.verify(this.hash.serialize(), new PublicKey(peerId.pubKey.marshal()))
  }
}

export { Challenge }
