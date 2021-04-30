import { u8aSplit, serializeToU8a, validateAcknowledgement } from '..'
import { Hash, PublicKey, Ticket } from '..'

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

  public verifySignature(signer: PublicKey): boolean {
    return this.ticket.verify(signer)
  }

  verify(signer: PublicKey, acknowledgement: Hash): ReturnType<typeof validateAcknowledgement> {
    if (!this.verifySignature(signer)) {
      return { valid: false }
    }

    return validateAcknowledgement(
      this.ownKey.serialize(),
      acknowledgement.serialize(),
      this.ticket.challenge.serialize()
    )
  }

  static SIZE(): number {
    return Ticket.SIZE + Hash.SIZE
  }
}
