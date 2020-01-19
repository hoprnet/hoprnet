import secp256k1 from 'secp256k1'

import { toU8a } from '../../utils'
import PeerId from 'peer-id'

import { HoprCoreConnectorClass } from '@hoprnet/hopr-core-connector-interface'

import BN from 'bn.js'

const COMPRESSED_PUBLIC_KEY_LENGTH = 33

/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an
 * inappropriate acknowledgement.
 */
export class Challenge<Chain extends HoprCoreConnectorClass> extends Uint8Array {
  // private : Uint8Array
  private _hashedKey: Uint8Array
  private _fee: BN

  private constructor(private paymentChannels: Chain, buf: Uint8Array) {
    super(buf)
  }

  get challengeSignature(): Uint8Array {
    return this.subarray(0, this.paymentChannels.constants.SIGNATURE_LENGTH)
  }

  set challengeSignature(signature: Uint8Array) {
    this.set(signature, 0)
  }

  get challengeSignatureRecovery(): Uint8Array {
    return this.subarray(this.paymentChannels.constants.SIGNATURE_LENGTH, this.paymentChannels.constants.SIGNATURE_LENGTH + 1)
  }

  set challengeSignatureRecovery(recovery: Uint8Array) {
    this.set(recovery, this.paymentChannels.constants.SIGNATURE_LENGTH)
  }

  get signatureHash(): Promise<Uint8Array> {
    return this.paymentChannels.utils.hash(Buffer.from(this.subarray(0, this.paymentChannels.constants.SIGNATURE_LENGTH + 1).buffer))
  }

  get SIZE(): number {
    return this.paymentChannels.constants.SIGNATURE_LENGTH + 1
  }

  get hash(): Promise<Uint8Array> {
    return this.paymentChannels.utils.hash(this._hashedKey)
  }

  subarray(begin?: number, end?: number): Uint8Array {
    return new Uint8Array(this).subarray(begin, end)
  }

  /**
   * Uses the derived secret and the signature to recover the public
   * key of the signer.
   */
  // get counterparty(): Promise<Uint8Array> {
  //   return new Promise<Uint8Array>(async (resolve, reject) => {
  //     if (this._counterparty) return this._counterparty

  //     if (!this._hashedKey) {
  //       return reject(Error(`Can't recover public key without challenge.`))
  //     }

  //     this._counterparty = secp256k1.recover(
  //       Buffer.from(await this.paymentChannels.utils.hash(this._hashedKey)),
  //       Buffer.from(this.challengeSignature),
  //       parseInt(this.challengeSignatureRecovery.toString(), 16)
  //     )
  //     return resolve(this._counterparty)
  //   })
  // }

  /**
   * Signs the challenge and includes the transferred amount of money as
   * well as the ethereum address of the signer into the signature.
   *
   * @param peerId that contains private key and public key of the node
   */
  async sign(peerId: PeerId): Promise<Challenge<Chain>> {
    // const hashedChallenge = hash(Buffer.concat([this._hashedKey, this._fee.toBuffer('be', VALUE_LENGTH)], HASH_LENGTH + VALUE_LENGTH))
    const signature = await this.paymentChannels.utils.sign(await this.hash, peerId.privKey.marshal(), peerId.pubKey.marshal())

    this.challengeSignature = signature.signature
    
    this.challengeSignatureRecovery = toU8a(signature.recovery, 1)

    return this
  }

  /**
   * Creates a challenge object.
   *
   * @param hashedKey that is used to generate the key half
   * @param fee
   */
  static create<Chain extends HoprCoreConnectorClass>(hoprCoreConnector: Chain, hashedKey: Uint8Array, fee: BN): Challenge<Chain> {
    if (hashedKey.length != COMPRESSED_PUBLIC_KEY_LENGTH) {
      throw Error(`Invalid secret format. Expected a ${Uint8Array.name} of ${COMPRESSED_PUBLIC_KEY_LENGTH} elements but got one with ${hashedKey.length}`)
    }

    const challenge = new Challenge(hoprCoreConnector, new Uint8Array(SIZE(hoprCoreConnector.constants.SIGNATURE_LENGTH)))
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

    return this.paymentChannels.utils.verify(await this.hash, {
      signature: this.challengeSignature,
      recovery: this.challengeSignatureRecovery[0]
    }, peerId.pubKey.marshal())
  }
}

export const SIZE = (signatureSize: number) => signatureSize + 1
