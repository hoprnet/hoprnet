import PeerId from 'peer-id'
import type HoprCoreConnector from '..'
import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'
import { Hash, SignedTicket } from '.'

// @TODO this is a duplicate of the same class in hopr-core
class AcknowledgedTicket<Chain extends HoprCoreConnector = HoprCoreConnector> extends Uint8Array {
  private _signedTicket: SignedTicket
  private _response: Hash
  private _preImage: Hash

  private paymentChannels: Chain

  constructor(
    paymentChannels: Chain,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signedTicket: SignedTicket
      response: Hash
      preImage: Hash
      redeemed: boolean
    }
  ) {
    if (arr == null) {
      super(AcknowledgedTicket.SIZE(paymentChannels))
    } else {
      super(arr.bytes, arr.offset, AcknowledgedTicket.SIZE(paymentChannels))
    }

    this.paymentChannels = paymentChannels

    if (struct != null) {
      this.set(struct.signedTicket, this.signedTicketOffset - this.byteOffset)
      this.set(struct.response, this.responseOffset - this.byteOffset)
      this.set(struct.preImage, this.preImageOffset - this.byteOffset)
      this.set([struct.redeemed ? 1 : 0], this.redeemedOffset - this.byteOffset)

      this._signedTicket = struct.signedTicket
      this._response = struct.response
      this._preImage = struct.preImage
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
        new Uint8Array(this.buffer, this.signedTicketOffset, this.paymentChannels.types.Hash.SIZE)
      )
    }

    return this._preImage
  }

  get redeemedOffset(): number {
    return (
      this.byteOffset +
      this.paymentChannels.types.SignedTicket.SIZE +
      this.paymentChannels.types.Hash.SIZE +
      this.paymentChannels.types.Hash.SIZE
    )
  }

  get redeemed(): boolean {
    return this[this.redeemedOffset - this.byteOffset] == 0 ? false : true
  }

  set redeemed(_redeemed: boolean) {
    this.set([_redeemed ? 1 : 0], this.redeemedOffset - this.byteOffset)
  }

  async verify(peerId: PeerId): Promise<boolean> {
    const signatureOk = (await this.signedTicket).verify(peerId.pubKey.marshal())

    return (await this.verifyChallenge()) && signatureOk
  }

  async isWinning(): Promise<boolean> {
    const luck = await this.paymentChannels.utils.hash(
      u8aConcat(await (await this.signedTicket).ticket.hash, this.preImage, this.response)
    )

    return luck < (await this.signedTicket).ticket.winProb
  }

  static SIZE<Chain extends HoprCoreConnector = HoprCoreConnector>(hoprCoreConnector: Chain): number {
    return (
      hoprCoreConnector.types.SignedTicket.SIZE +
      hoprCoreConnector.types.Hash.SIZE +
      hoprCoreConnector.types.Hash.SIZE +
      1
    )
  }

  private async verifyChallenge(): Promise<boolean> {
    return u8aEquals((await this.signedTicket).ticket.challenge, await this.paymentChannels.utils.hash(this.response))
  }
}

export default AcknowledgedTicket
