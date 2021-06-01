import { Hash, Ticket, PublicKey, Response } from '.'
import { serializeToU8a, u8aSplit, validatePoRResponse } from '..'

export class AcknowledgedTicket {
  constructor(
    readonly ticket: Ticket,
    readonly response: Response,
    readonly preImage: Hash,
    readonly signer: PublicKey
  ) {
    if (signer.toAddress().eq(this.ticket.counterparty)) {
      throw Error(`Given signer public key must be different from counterparty`)
    }
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.response.serialize(), Response.SIZE],
      [this.preImage.serialize(), Hash.SIZE],
      [this.signer.serialize(), PublicKey.SIZE]
    ])
  }

  public verify(ticketIssuer: PublicKey): boolean {
    return validatePoRResponse(this.ticket.challenge, this.response) && this.ticket.verify(ticketIssuer)
  }

  static deserialize(arr: Uint8Array) {
    const components = u8aSplit(arr, [Ticket.SIZE, Hash.SIZE, Hash.SIZE, PublicKey.SIZE])
    return new AcknowledgedTicket(
      Ticket.deserialize(components[0]),
      new Response(components[1]),
      new Hash(components[2]),
      new PublicKey(components[3])
    )
  }

  static get SIZE(): number {
    return Ticket.SIZE + Response.SIZE + Hash.SIZE
  }
}
