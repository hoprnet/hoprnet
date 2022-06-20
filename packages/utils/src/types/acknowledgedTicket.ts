import { Hash, Ticket, PublicKey, Response } from './index.js'
import { serializeToU8a, u8aSplit, validatePoRResponse } from '../index.js'

export class AcknowledgedTicket {
  constructor(
    public readonly ticket: Ticket,
    public readonly response: Response,
    public readonly preImage: Hash,
    public readonly signer: PublicKey
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
      [this.signer.serializeCompressed(), PublicKey.SIZE_COMPRESSED]
    ])
  }

  public verify(ticketIssuer: PublicKey): boolean {
    const check1 = validatePoRResponse(this.ticket.challenge, this.response)
    const check2 = this.ticket.verify(ticketIssuer)
    return check1 && check2
  }

  static deserialize(arr: Uint8Array) {
    const components = u8aSplit(arr, [Ticket.SIZE, Hash.SIZE, Hash.SIZE, PublicKey.SIZE_COMPRESSED])
    return new AcknowledgedTicket(
      Ticket.deserialize(components[0]),
      Response.deserialize(components[1]),
      Hash.deserialize(components[2]),
      PublicKey.deserialize(components[3])
    )
  }

  static get SIZE(): number {
    return Ticket.SIZE + Response.SIZE + Hash.SIZE + PublicKey.SIZE_COMPRESSED
  }
}
