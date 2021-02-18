import { Hash, SignedTicket } from '.'
import { HASHED_SECRET_WIDTH } from '../hashedSecret'

class AcknowledgedTicket{
  private _signedTicket: SignedTicket
  private _response: Hash
  private _preImage: Hash
  private _serialized: Uint8Array

  constructor(
    signedTicket: SignedTicket,
    response: Hash,
    preImage?: Hash
  ) {

    this._serialized = new Uint8Array(AcknowledgedTicket.SIZE())

    this._serialized.set(signedTicket, this.signedTicketOffset - this._serialized.byteOffset)
    this._signedTicket = signedTicket

    this._serialized.set(response, this.responseOffset - this._serialized.byteOffset)
    this._response = response

    if (preImage) {
      this._serialized.set(preImage, this.preImageOffset - this._serialized.byteOffset)
      this._preImage = preImage
    }
  }

  /*
  subarray(begin: number = 0, end: number = AcknowledgedTicket.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }
  */

  get signedTicketOffset(): number {
    return this._serialized.byteOffset
  }

  get signedTicket(): Promise<SignedTicket> {
    if (this._signedTicket) {
      return Promise.resolve(this._signedTicket)
    }

    return new Promise<SignedTicket>(async (resolve) => {
      this._signedTicket = await SignedTicket.create({
        bytes: this._serialized.buffer,
        offset: this.signedTicketOffset
      })

      resolve(this._signedTicket)
    })
  }

  get responseOffset(): number {
    return this._serialized.byteOffset + SignedTicket.SIZE
  }

  get response(): Hash {
    if (!this._response) {
      this._response = new Hash(
        new Uint8Array(this._serialized.buffer, this.responseOffset, Hash.SIZE)
      )
    }

    return this._response
  }

  get preImageOffset(): number {
    return this._serialized.byteOffset + SignedTicket.SIZE + Hash.SIZE
  }

  get preImage(): Hash {
    if (!this._preImage) {
      this._preImage = new Hash(
        new Uint8Array(this._serialized.buffer, this.preImageOffset, HASHED_SECRET_WIDTH)
      )
    }

    return this._preImage
  }

  set preImage(preImage: Hash) {
    this._serialized.set(preImage, this.preImageOffset)

    this._preImage = new Hash(
      new Uint8Array(this._serialized.buffer, this.preImageOffset, HASHED_SECRET_WIDTH)
    )
  }

  static SIZE(): number {
    return SignedTicket.SIZE + Hash.SIZE + HASHED_SECRET_WIDTH + 1
  }

  public serialized(): Uint8Array {
    return this._serialized
  }
}

export default AcknowledgedTicket
