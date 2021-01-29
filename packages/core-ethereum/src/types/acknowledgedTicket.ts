import type HoprEthereum from '../'
import { Hash } from '.'
import SignedTicket, { SIGNED_TICKET_SIZE } from './signedTicket' 

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
    }
  ) {
    if (!arr) {
      super(AcknowledgedTicket.SIZE(paymentChannels))
    } else {
      super(arr.bytes, arr.offset, AcknowledgedTicket.SIZE(paymentChannels))
    }

    this.paymentChannels = paymentChannels

    if (struct) {
      if (struct.signedTicket) {
        this.set(struct.signedTicket.serialize(), this.signedTicketOffset - this.byteOffset)
        this._signedTicket = struct.signedTicket
      }

      if (struct.response) {
        this.set(struct.response, this.responseOffset - this.byteOffset)
        this._response = struct.response
      }

      if (struct.preImage) {
        this.set(struct.preImage, this.preImageOffset - this.byteOffset)
        this._preImage = struct.preImage
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
    if (this._signedTicket) {
      return Promise.resolve(this._signedTicket)
    }

    return new Promise<SignedTicket>(async (resolve) => {
      this._signedTicket = new SignedTicket({
        bytes: this.buffer,
        offset: this.signedTicketOffset
      })

      resolve(this._signedTicket)
    })
  }

  get responseOffset(): number {
    return this.byteOffset + SIGNED_TICKET_SIZE 
  }

  get response(): Hash {
    if (!this._response) {
      this._response = new this.paymentChannels.types.Hash(
        new Uint8Array(this.buffer, this.responseOffset, this.paymentChannels.types.Hash.SIZE)
      )
    }

    return this._response
  }

  get preImageOffset(): number {
    return this.byteOffset + SIGNED_TICKET_SIZE + this.paymentChannels.types.Hash.SIZE
  }

  get preImage(): Hash {
    if (!this._preImage) {
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

  static SIZE(hoprCoreConnector: HoprEthereum): number {
    return SIGNED_TICKET_SIZE + hoprCoreConnector.types.Hash.SIZE + HASHED_SECRET_WIDTH + 1
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
