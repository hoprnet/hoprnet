import { u8aSplit, serializeToU8a } from '@hoprnet/hopr-utils'
import { Hash, PublicKey, Ticket } from '..'
import PeerId from 'peer-id'

export class UnacknowledgedTicket {
  constructor(readonly ticket: Ticket, readonly secretA: Hash) {}

  static deserialize(arr: Uint8Array): UnacknowledgedTicket {
    const components = u8aSplit(arr, [Ticket.SIZE, Hash.SIZE])

    return new UnacknowledgedTicket(Ticket.deserialize(components[0]), new Hash(components[1]))
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.secretA.serialize(), Hash.SIZE]
    ])
  }

  private verifyChallenge(_hashedKeyHalf: Uint8Array): boolean {
    // @TODO: fix this
    return true
  }

  private verifySignature(peerId: PeerId): boolean {
    return this.ticket.verify(new PublicKey(peerId.pubKey.marshal()))
  }

  verify(peerId: PeerId, hashedKeyHalf: Uint8Array): boolean {
    return this.verifyChallenge(hashedKeyHalf) && this.verifySignature(peerId)
  }

  static SIZE(): number {
    return Ticket.SIZE + Hash.SIZE
  }
}
