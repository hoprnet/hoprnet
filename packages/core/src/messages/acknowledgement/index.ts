import secp256k1 from 'secp256k1'
import { deriveTicketKeyBlinding } from '../packet/header'
import { KEY_LENGTH } from '../packet/header/parameters'
import { Challenge } from '../packet/challenge'
import { Hash, Signature, PublicKey } from '@hoprnet/hopr-core-ethereum'
import PeerId from 'peer-id'

/**
 * This class encapsulates the message that is sent back to the relayer
 * and allows that party to compute the key that is necessary to redeem
 * the previously received transaction.
 */
class Acknowledgement extends Uint8Array {
  private _responseSigningParty?: Uint8Array
  private _responseSignature?: Signature

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      key: Uint8Array
      challenge: Challenge
      signature?: Signature
    }
  ) {
    if (arr == null) {
      super(Acknowledgement.SIZE())
    } else {
      super(arr.bytes, arr.offset, Acknowledgement.SIZE())
    }

    if (struct != null) {
      if (struct.key != null) {
        this.set(struct.key, this.keyOffset - this.byteOffset)
      }

      if (struct.challenge != null) {
        this.set(struct.challenge, this.challengeOffset - this.byteOffset)
      }

      if (struct.signature != null) {
        this.set(struct.signature.serialize(), this.responseSignatureOffset - this.byteOffset)
      }
    }
  }

  slice(begin: number = 0, end: number = Acknowledgement.SIZE()) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end: number = Acknowledgement.SIZE()): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get keyOffset(): number {
    return this.byteOffset
  }

  get key(): Uint8Array {
    return new Uint8Array(this.buffer, this.keyOffset, KEY_LENGTH)
  }

  get hashedKey(): Promise<Hash> {
    return Promise.resolve(Hash.create(this.key))
  }

  get challengeOffset(): number {
    return this.byteOffset + KEY_LENGTH
  }

  get challenge(): Challenge {
    return new Challenge({
      bytes: this.buffer,
      offset: this.challengeOffset
    })
  }

  get hash(): Promise<Hash> {
    return Promise.resolve(Hash.create(new Uint8Array(this.buffer, this.byteOffset, KEY_LENGTH + Challenge.SIZE())))
  }

  get challengeSignatureHash(): Promise<Hash> {
    return Promise.resolve(Hash.create(this.challenge))
  }

  get challengeSigningParty() {
    return this.challenge.counterparty
  }

  get responseSignatureOffset(): number {
    return this.byteOffset + KEY_LENGTH + Challenge.SIZE()
  }

  get responseSignature(): Promise<Signature> {
    if (this._responseSignature != null) {
      return Promise.resolve(this._responseSignature)
    }

    return new Promise<Signature>(async (resolve) => {
      this._responseSignature = Signature.deserialize(
        new Uint8Array(this.buffer, this.responseSignatureOffset, Signature.SIZE)
      )
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
        (await this.hash).serialize()
      )

      resolve(this._responseSigningParty)
    })
  }

  async sign(peerId: PeerId): Promise<Acknowledgement> {
    const signature = Signature.create((await this.hash).serialize(), peerId.privKey.marshal())
    this.set(signature.serialize(), this.responseSignatureOffset - this.byteOffset)
    return this
  }

  async verify(peerId: PeerId): Promise<boolean> {
    return (await this.responseSignature).verify((await this.hash).serialize(), new PublicKey(peerId.pubKey.marshal()))
  }

  /**
   * Takes a challenge from a relayer and returns an acknowledgement that includes a
   * signature over the requested key half.
   *
   * @param challenge the signed challenge of the relayer
   * @param derivedSecret the secret that is used to create the second key half
   * @param signer contains private key
   */
  static async create(challenge: Challenge, derivedSecret: Uint8Array, signer: PeerId): Promise<Acknowledgement> {
    const ack = new Acknowledgement({
      bytes: new Uint8Array(Acknowledgement.SIZE()),
      offset: 0
    })

    ack.set(deriveTicketKeyBlinding(derivedSecret), ack.keyOffset - ack.byteOffset)
    ack.set(challenge, ack.challengeOffset - ack.byteOffset)

    return ack.sign(signer)
  }

  static SIZE(): number {
    return KEY_LENGTH + Challenge.SIZE() + Signature.SIZE
  }
}

export { Acknowledgement }
