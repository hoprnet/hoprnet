import { Hash, Ticket } from '.'
import { serializeToU8a, u8aSplit, validateAcknowledgement } from '..'
import { PublicKey } from '..'

export class AcknowledgedTicket {
  constructor(readonly ticket: Ticket, readonly response: Hash, readonly preImage: Hash) {}

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.response.serialize(), Hash.SIZE],
      [this.preImage.serialize(), Hash.SIZE]
    ])
  }

  public verify(ticketIssuer: PublicKey): boolean {
    return (
      validateAcknowledgement(undefined, undefined, this.ticket.challenge, undefined, this.response.serialize())
        .valid && this.ticket.verify(ticketIssuer)
    )
  }

  static deserialize(arr: Uint8Array) {
    const components = u8aSplit(arr, [Ticket.SIZE, Hash.SIZE, Hash.SIZE])
    return new AcknowledgedTicket(Ticket.deserialize(components[0]), new Hash(components[1]), new Hash(components[2]))
  }

  static get SIZE(): number {
    return Ticket.SIZE + Hash.SIZE + Hash.SIZE
  }
}
