import { u8aSplit, serializeToU8a, validateAcknowledgement } from '@hoprnet/hopr-utils'
import { Hash, PublicKey, Ticket } from '..'
import PeerId from 'peer-id'

export class UnacknowledgedTicket {
  constructor(readonly ticket: Ticket, readonly ownKey: Hash) {}

  static deserialize(arr: Uint8Array): UnacknowledgedTicket {
    const components = u8aSplit(arr, [Ticket.SIZE, Hash.SIZE])

    return new UnacknowledgedTicket(Ticket.deserialize(components[0]), new Hash(components[1]))
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.ownKey.serialize(), Hash.SIZE]
    ])
  }

  public verifyChallenge(acknowledgement: Hash) {
    return validateAcknowledgement(
      this.ownKey.serialize(),
      acknowledgement.serialize(),
      this.ticket.challenge.serialize()
    )
  }

  private verifySignature(peerId: PeerId): boolean {
    return this.ticket.verify(new PublicKey(peerId.pubKey.marshal()))
  }

  verify(peerId: PeerId): boolean {
    return this.verifySignature(peerId)
  }

  static SIZE(): number {
    return Ticket.SIZE + Hash.SIZE
  }
}
