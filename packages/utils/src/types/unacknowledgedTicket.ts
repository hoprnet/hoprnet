import { u8aSplit, serializeToU8a, validatePoRHalfKeys } from '..'
import { HalfKeyChallenge, HalfKey, PublicKey, Ticket, Response } from '.'

export class UnacknowledgedTicket {
  constructor(readonly ticket: Ticket, readonly ownKey: HalfKey, readonly signer: PublicKey) {
    if (signer.toAddress().eq(this.ticket.counterparty)) {
      throw Error(`Given signer public key must be different from counterparty`)
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
      [this.signer.serialize(), PublicKey.SIZE]
    ])
  }

  public verifyChallenge(acknowledgement: HalfKey): boolean {
    return validatePoRHalfKeys(this.ticket.challenge, this.ownKey, acknowledgement)
  }

  public verifySignature(): boolean {
    return this.ticket.verify(this.signer)
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
