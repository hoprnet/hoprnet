import { Hash, Ticket } from '.'
import { serializeToU8a, u8aSplit, validatePoRResponse } from '..'
import { PublicKey, Opening, Response } from '.'

export class AcknowledgedTicket {
  constructor(readonly ticket: Ticket, readonly response: Response, readonly preImage: Opening) {}

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.response.serialize(), Response.SIZE],
      [this.preImage.serialize(), Opening.SIZE]
    ])
  }

  public verify(ticketIssuer: PublicKey): boolean {
    return validatePoRResponse(this.ticket.challenge, this.response) && this.ticket.verify(ticketIssuer)
  }

  static deserialize(arr: Uint8Array) {
    const components = u8aSplit(arr, [Ticket.SIZE, Hash.SIZE, Hash.SIZE])
    return new AcknowledgedTicket(
      Ticket.deserialize(components[0]),
      new Response(components[1]),
      new Opening(components[2])
    )
  }

  static get SIZE(): number {
    return Ticket.SIZE + Response.SIZE + Opening.SIZE
  }
}
