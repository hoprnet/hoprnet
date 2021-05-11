import { u8aSplit, serializeToU8a, validatePoRHalfKeys } from '..'
import { HalfKeyChallenge, HalfKey, PublicKey, Ticket, Response } from '.'

export class UnacknowledgedTicket {
  constructor(readonly ticket: Ticket, readonly ownKey: HalfKey) {}

  static deserialize(arr: Uint8Array): UnacknowledgedTicket {
    const components = u8aSplit(arr, [Ticket.SIZE, HalfKey.SIZE])

    return new UnacknowledgedTicket(Ticket.deserialize(components[0]), new HalfKey(components[1]))
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.ownKey.serialize(), HalfKey.SIZE]
    ])
  }

  public verifySignature(signer: PublicKey): boolean {
    return this.ticket.verify(signer)
  }

  public getResponse(acknowledgement: HalfKey): Response {
    return Response.fromHalfKeys(this.ownKey, acknowledgement)
  }

  public getChallenge(): HalfKeyChallenge {
    return this.ownKey.toChallenge()
  }

  public verify(signer: PublicKey, acknowledgement: HalfKey): boolean {
    return this.verifySignature(signer) && validatePoRHalfKeys(this.ticket.challenge, this.ownKey, acknowledgement)
  }

  static SIZE(): number {
    return Ticket.SIZE + Response.SIZE
  }
}
