import { u8aSplit, serializeToU8a, validatePoRHalfKeys } from '..'
import { HalfKeyChallenge, HalfKey, PublicKey, Ticket, Response } from '.'

export class UnacknowledgedTicket {
  constructor(readonly ticket: Ticket, readonly ownKey: HalfKey, readonly counterparty: PublicKey) {
    if (!counterparty.toAddress().eq(this.ticket.counterparty)) {
      throw Error(`Given public key of counterparty does not fit to ticket data`)
    }
  }

  static deserialize(arr: Uint8Array): UnacknowledgedTicket {
    const components = u8aSplit(arr, [Ticket.SIZE, HalfKey.SIZE, PublicKey.SIZE])

    return new UnacknowledgedTicket(
      Ticket.deserialize(components[0]),
      new HalfKey(components[1]),
      new PublicKey(components[2])
    )
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.ownKey.serialize(), HalfKey.SIZE],
      [this.counterparty.serialize(), PublicKey.SIZE]
    ])
  }

  public verifyChallenge(acknowledgement: HalfKey): boolean {
    return validatePoRHalfKeys(this.ticket.challenge, this.ownKey, acknowledgement)
  }

  public verifySignature(): boolean {
    return this.ticket.verify(this.counterparty)
  }

  public getResponse(acknowledgement: HalfKey): Response {
    return Response.fromHalfKeys(this.ownKey, acknowledgement)
  }

  public getChallenge(): HalfKeyChallenge {
    return this.ownKey.toChallenge()
  }

  static SIZE(): number {
    return Ticket.SIZE + Response.SIZE
  }
}
