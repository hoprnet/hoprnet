import { u8aSplit, serializeToU8a, validatePoRHalfKeys } from '../index.js'
import { HalfKeyChallenge, HalfKey as TSHalfKey, PublicKey, Ticket, Response } from './index.js'
import { EthereumChallenge, HalfKey } from '../index.js'

export class UnacknowledgedTicket {
  constructor(public readonly ticket: Ticket, public readonly ownKey: TSHalfKey, public readonly signer: PublicKey) {
    if (signer.toAddress().eq(this.ticket.counterparty)) {
      throw Error(`Given signer public key must be different from counterparty`)
    }
  }

  static deserialize(arr: Uint8Array): UnacknowledgedTicket {
    const components = u8aSplit(arr, [Ticket.SIZE, TSHalfKey.SIZE, PublicKey.SIZE_UNCOMPRESSED])

    return new UnacknowledgedTicket(
      Ticket.deserialize(components[0]),
      TSHalfKey.deserialize(components[1]),
      PublicKey.deserialize(components[2])
    )
  }

  public serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.ownKey.serialize(), TSHalfKey.SIZE],
      [this.signer.serializeUncompressed(), PublicKey.SIZE_UNCOMPRESSED]
    ])
  }

  public verifyChallenge(acknowledgement: TSHalfKey): boolean {
    return validatePoRHalfKeys(
      EthereumChallenge.deserialize(this.ticket.challenge.serialize()),
      HalfKey.deserialize(this.ownKey.serialize()),
      HalfKey.deserialize(acknowledgement.serialize())
    )
  }

  public verifySignature(): boolean {
    return this.ticket.verify(this.signer)
  }

  public getResponse(acknowledgement: HalfKey): Response {
    return Response.fromHalfKeys(this.ownKey, TSHalfKey.deserialize(acknowledgement.serialize()))
  }

  public getChallenge(): HalfKeyChallenge {
    return this.ownKey.toChallenge()
  }

  static SIZE(): number {
    return Ticket.SIZE + Response.SIZE + PublicKey.SIZE_UNCOMPRESSED
  }
}
