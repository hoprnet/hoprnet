import { u8aConcat } from '@hoprnet/hopr-utils'
import { Hash, PublicKey, Ticket } from '@hoprnet/hopr-core-ethereum'
import PeerId from 'peer-id'

class UnacknowledgedTicket extends Uint8Array {
  private _signedTicket: Ticket
  private _secretA: Hash

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      ticket: Ticket
      secretA: Hash
    }
  ) {
    if (arr == null) {
      super(UnacknowledgedTicket.SIZE())
    } else {
      super(arr.bytes, arr.offset, UnacknowledgedTicket.SIZE())
    }

    if (struct != null) {
      this.set(struct.ticket.serialize(), this.signedTicketOffset - this.byteOffset)
      this.set(struct.secretA.serialize(), this.secretAOffset - this.byteOffset)

      this._signedTicket = struct.ticket
      this._secretA = struct.secretA
    }
  }

  slice(begin: number = 0, end: number = UnacknowledgedTicket.SIZE()) {
    return this.subarray(begin, end)
  }

  subarray(begin: number = 0, end: number = UnacknowledgedTicket.SIZE()): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get signedTicketOffset(): number {
    return this.byteOffset
  }

  get signedTicket(): Promise<Ticket> {
    if (this._signedTicket != null) {
      return Promise.resolve(this._signedTicket)
    }

    return new Promise<Ticket>(async (resolve) => {
      this._signedTicket = await Ticket.deserialize(new Uint8Array(this.buffer, this.signedTicketOffset, Ticket.SIZE))

      resolve(this._signedTicket)
    })
  }

  get secretAOffset(): number {
    return this.byteOffset + Ticket.SIZE
  }

  get secretA(): Hash {
    if (this._secretA == null) {
      this._secretA = new Hash(new Uint8Array(this.buffer, this.secretAOffset, Hash.SIZE))
    }

    return this._secretA
  }

  async verifyChallenge(hashedKeyHalf: Uint8Array) {
    return Hash.create(u8aConcat(this.secretA.serialize(), hashedKeyHalf))
      .hash()
      .eq((await this.signedTicket).challenge as Hash)
  }

  async verifySignature(peerId: PeerId) {
    return (await this.signedTicket).verify(new PublicKey(peerId.pubKey.marshal()))
  }

  async verify(peerId: PeerId, hashedKeyHalf: Uint8Array): Promise<boolean> {
    return (await this.verifyChallenge(hashedKeyHalf)) && (await this.verifySignature(peerId))
  }

  static SIZE(): number {
    return Ticket.SIZE + Hash.SIZE
  }
}

export { UnacknowledgedTicket }
