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
class Challenge {

  constructor(
    private createSignature: (arr: Uint8Array) => Types.Signature,
    private signatureSize: number, // Signature.SIZE
    private _hash: () => Uint8Array, // utils.hash
    private _sign: () => Uint8Array, //utils.sign

    private hash: Uint8Array,
    private fee: BN,
    private counterparty: Uint8Array,
    private challengeSignature: Types.Signature,
  ) {
  }

  public serialize(): Uint8Array {
    let arr = new Uint8Array(this.signatureSize)
    arr.set()
    return arr
  }

  public static deserialize(arr: Uint8Array, createSignature: (arr: Uint8Array) => Types.Signature): Challenge {
    const challengeSignature = createSignature(arr) 
    const challenge = secp256k1.ecdsaRecover(
          challengeSignature.signature,
          challengeSignature.recovery,
          challengeSignature.msgPrefix != null && challengeSignature.msgPrefix.length > 0
            ? await this._hash(u8aConcat(challengeSignature.msgPrefix, await this.hash))
            : this.hash
        )

    return new Challenge(challenge)
  }

  async signatureHash(): Promise<Types.Hash> {
    return await this._hash(this.challengeSignature)
  }

  static SIZE(signatureSize: number): number {
    return signatureSize
  }

  getCopy(): Challenge{
    return Challenge.deserialize(this.serialize())
  }

  /**
   * Uses the derived secret and the signature to recover the public
   * key of the signer.
   */
  /*
  get counterparty(): Promise<Uint8Array> {
    if (this._counterparty != null) {
      return Promise.resolve(this._counterparty)
    }

    return new Promise<Uint8Array>(async (resolve) => {
    })
  }
  */

  /**
   * Signs the challenge and includes the transferred amount of money as
   * well as the ethereum address of the signer into the signature.
   *
   * @param peerId that contains private key and public key of the node
   */
  async sign(peerId: PeerId): Promise<Challenge> {
    await this._sign(this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal(), {
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
  static create(
    hoprCoreConnector: HoprCoreConnector,
    hashedKey: Uint8Array,
    fee: BN,
  ): Challenge {
    if (hashedKey.length != KEY_LENGTH) {
      throw Error(
        `Invalid secret format. Expected a ${Uint8Array.name} of ${KEY_LENGTH} elements but got one with ${hashedKey.length}`
      )
    }

    const createSignature = (arr: Uint8Array) => hoprCoreConnector.types.Signature.create({arr.buffer, arr.byteOffset })
    const signatureSize = hoprCoreConnector.types.Signature.SIZE
    return new Challenge(createSignature, signatureSize, hashedKey, fee)
  }

  /**
   * Verifies the challenge by checking whether the given public matches the
   * one restored from the signature.
   *
   * @param peerId PeerId instance that contains the public key of
   * the signer
   * @param secret the secret that was used to derive the key half
   */
  async verify(peerId: PeerId, paymentChannels: HoprCoreConnector): Promise<boolean> {
    if (!peerId.pubKey) {
      throw Error('Unable to verify challenge without a public key.')
    }

    return paymentChannels.utils.verify(this.hash, this.challengeSignature, peerId.pubKey.marshal())
  }
}

export { Challenge }
