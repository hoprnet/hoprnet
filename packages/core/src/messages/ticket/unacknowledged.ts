import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'

import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'

class UnacknowledgedTicket<Chain extends HoprCoreConnector> extends Uint8Array {
  private _signedTicket: Types.SignedTicket
  private _secretA: Types.Hash

  private paymentChannels: Chain

  constructor(
    paymentChannels: Chain,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signedTicket: Types.SignedTicket
      secretA: Types.Hash
    }
  ) {
    if (arr == null) {
      super(UnacknowledgedTicket.SIZE(paymentChannels))
    } else {
      super(arr.bytes, arr.offset, UnacknowledgedTicket.SIZE(paymentChannels))
    }

    this.paymentChannels = paymentChannels

    if (struct != null) {
      this.set(struct.signedTicket.serialize(), this.signedTicketOffset - this.byteOffset)
      this.set(struct.secretA, this.secretAOffset - this.byteOffset)

      this._signedTicket = struct.signedTicket
      this._secretA = struct.secretA
    }
  }

  slice(begin: number = 0, end: number = UnacknowledgedTicket.SIZE(this.paymentChannels)) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end: number = UnacknowledgedTicket.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get signedTicketOffset(): number {
    return this.byteOffset
  }

  get signedTicket(): Promise<Types.SignedTicket> {
    if (this._signedTicket != null) {
      return Promise.resolve(this._signedTicket)
    }

    return new Promise<Types.SignedTicket>(async (resolve) => {
      this._signedTicket = new this.paymentChannels.types.SignedTicket({
        bytes: this.buffer,
        offset: this.signedTicketOffset
      })

      resolve(this._signedTicket)
    })
  }

  get secretAOffset(): number {
    return this.byteOffset + this.paymentChannels.types.SignedTicket.SIZE
  }

  get secretA(): Types.Hash {
    if (this._secretA == null) {
      this._secretA = new this.paymentChannels.types.Hash(
        new Uint8Array(this.buffer, this.secretAOffset, this.paymentChannels.types.Hash.SIZE)
      )
    }

    return this._secretA
  }

  async verifyChallenge(hashedKeyHalf: Uint8Array) {
    return u8aEquals(
      await this.paymentChannels.utils.hash(
        await this.paymentChannels.utils.hash(u8aConcat(this.secretA, hashedKeyHalf))
      ),
      (await this.signedTicket).ticket.challenge
    )
  }

  async verifySignature(peerId: PeerId) {
    return (await this.signedTicket).verify(peerId.pubKey.marshal())
  }

  async verify(peerId: PeerId, hashedKeyHalf: Uint8Array): Promise<boolean> {
    return (await this.verifyChallenge(hashedKeyHalf)) && (await this.verifySignature(peerId))
  }

  static SIZE<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain): number {
    return hoprCoreConnector.types.SignedTicket.SIZE + hoprCoreConnector.types.Hash.SIZE
  }
}

export { UnacknowledgedTicket }
