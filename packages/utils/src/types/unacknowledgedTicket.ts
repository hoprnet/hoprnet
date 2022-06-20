import { u8aSplit, serializeToU8a, validatePoRHalfKeys } from '../index.js'
import { HalfKeyChallenge, HalfKey, PublicKey, Ticket, Response } from './index.js'

export class UnacknowledgedTicket {
  constructor(public readonly ticket: Ticket, public readonly ownKey: HalfKey, public readonly signer: PublicKey) {
    if (signer.toAddress().eq(this.ticket.counterparty)) {
      throw Error(`Given signer public key must be different from counterparty`)
    }
  }

  static deserialize(arr: Uint8Array): UnacknowledgedTicket {
    const components = u8aSplit(arr, [Ticket.SIZE, HalfKey.SIZE, PublicKey.SIZE_UNCOMPRESSED])

    return new UnacknowledgedTicket(
      Ticket.deserialize(components[0]),
      HalfKey.deserialize(components[1]),
      PublicKey.deserialize(components[2])
    )
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.ownKey.serialize(), HalfKey.SIZE],
      [this.signer.serializeUncompressed(), PublicKey.SIZE_UNCOMPRESSED]
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
    return Ticket.SIZE + Response.SIZE + PublicKey.SIZE_UNCOMPRESSED
  }
}
