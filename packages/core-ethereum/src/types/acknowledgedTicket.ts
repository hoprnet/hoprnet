import type HoprEthereum from '../'
import { Hash, SignedTicket } from '.'

import { HASHED_SECRET_WIDTH } from '../hashedSecret'

// @TODO this is a duplicate of the same class in hopr-core
class AcknowledgedTicket extends Uint8Array {
  private _signedTicket: SignedTicket
  private _response: Hash
  private _preImage: Hash

  private paymentChannels: HoprEthereum

  constructor(
    paymentChannels: HoprEthereum,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signedTicket?: SignedTicket
      response?: Hash
      preImage?: Hash
      redeemed?: boolean
    }
  ) {
    if (arr == null) {
      super(AcknowledgedTicket.SIZE(paymentChannels))
    } else {
      super(arr.bytes, arr.offset, AcknowledgedTicket.SIZE(paymentChannels))
    }

    this.paymentChannels = paymentChannels

    if (struct != null) {
      if (struct.signedTicket != null) {
        this.set(struct.signedTicket, this.signedTicketOffset - this.byteOffset)
        this._signedTicket = struct.signedTicket
      }

      if (struct.response != null) {
        this.set(struct.response, this.responseOffset - this.byteOffset)
        this._response = struct.response
      }

      if (struct.preImage != null) {
        this.set(struct.preImage, this.preImageOffset - this.byteOffset)
        this._preImage = struct.preImage
      }

      if (struct.redeemed != null) {
        this.set([struct.redeemed ? 1 : 0], this.redeemedOffset - this.byteOffset)
      }
    }
  }

  subarray(begin: number = 0, end: number = AcknowledgedTicket.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get signedTicketOffset(): number {
    return this.byteOffset
  }

  get signedTicket(): Promise<SignedTicket> {
    if (this._signedTicket != null) {
      return Promise.resolve(this._signedTicket)
    }

    return new Promise<SignedTicket>(async (resolve) => {
      this._signedTicket = await this.paymentChannels.types.SignedTicket.create({
        bytes: this.buffer,
        offset: this.signedTicketOffset,
      })

      resolve(this._signedTicket)
    })
  }

  get responseOffset(): number {
    return this.byteOffset + this.paymentChannels.types.SignedTicket.SIZE
  }

  get response(): Hash {
    if (this._response == null) {
      this._response = new this.paymentChannels.types.Hash(
        new Uint8Array(this.buffer, this.responseOffset, this.paymentChannels.types.Hash.SIZE)
      )
    }

    return this._response
  }

  get preImageOffset(): number {
    return this.byteOffset + this.paymentChannels.types.SignedTicket.SIZE + this.paymentChannels.types.Hash.SIZE
  }

  get preImage(): Hash {
    if (this._preImage == null) {
      this._preImage = new this.paymentChannels.types.Hash(
        new Uint8Array(this.buffer, this.preImageOffset, HASHED_SECRET_WIDTH)
      )
    }

    return this._preImage
  }

  set preImage(_preImage: Hash) {
    this.set(_preImage, this.preImageOffset)

    this._preImage = new this.paymentChannels.types.Hash(
      new Uint8Array(this.buffer, this.preImageOffset, HASHED_SECRET_WIDTH)
    )
  }

  get redeemedOffset(): number {
    return (
      this.byteOffset +
      this.paymentChannels.types.SignedTicket.SIZE +
      this.paymentChannels.types.Hash.SIZE +
      HASHED_SECRET_WIDTH
    )
  }

  get redeemed(): boolean {
    return this[this.redeemedOffset - this.byteOffset] == 0 ? false : true
  }

  set redeemed(_redeemed: boolean) {
    this.set([_redeemed ? 1 : 0], this.redeemedOffset - this.byteOffset)
  }

  static SIZE(hoprCoreConnector: HoprEthereum): number {
    return hoprCoreConnector.types.SignedTicket.SIZE + hoprCoreConnector.types.Hash.SIZE + HASHED_SECRET_WIDTH + 1
  }

  static create(
    coreConnector: HoprEthereum,
    arr?: { bytes: ArrayBuffer; offset: number },
    struct?: { signedTicket?: SignedTicket; response?: Hash; preImage?: Hash; redeemed?: boolean }
  ) {
    return new AcknowledgedTicket(coreConnector, arr, struct)
  }
}

export default AcknowledgedTicket
