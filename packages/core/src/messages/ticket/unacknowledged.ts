import { u8aConcat, u8aSplit, serializeToU8a } from '@hoprnet/hopr-utils'
import { Hash, PublicKey, Ticket } from '@hoprnet/hopr-core-ethereum'
import PeerId from 'peer-id'

class UnacknowledgedTicket {
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

  private verifyChallenge(hashedKeyHalf: Uint8Array): boolean {
    return Hash.create(u8aConcat(this.secretA.serialize(), hashedKeyHalf)).hash().eq(this.ticket.challenge)
  }

  private async verifySignature(peerId: PeerId): Promise<boolean> {
    return this.ticket.verify(new PublicKey(peerId.pubKey.marshal()))
  }

  async verify(peerId: PeerId, hashedKeyHalf: Uint8Array): Promise<boolean> {
    return this.verifyChallenge(hashedKeyHalf) && (await this.verifySignature(peerId))
  }

  static SIZE(): number {
    return Ticket.SIZE + Hash.SIZE
  }
}

export { UnacknowledgedTicket }
