import { u8aConcat, u8aEquals } from '@hoprnet/hopr-utils'
import type { UnacknowledgedTicket as IUnacknowledgedTicket } from '@hoprnet/hopr-core-connector-interface'
import { Hash, AcknowledgedTicket, SignedTicket } from '.'
import { hash } from '../utils'

class UnacknowledgedTicket implements IUnacknowledgedTicket {
  constructor(
    public signedTicket: SignedTicket,
    public secretA: Hash
  ) {}

  serialize(): Uint8Array {
    const serialized = new Uint8Array(UnacknowledgedTicket.SIZE())
    serialized.set(this.signedTicket.serialize(), 0)
    serialized.set(this.secretA, SignedTicket.SIZE)
    return serialized
  }

  static deserialize(arr: Uint8Array): UnacknowledgedTicket {
    const signedTicket = SignedTicket.deserialize(
      new Uint8Array(arr.buffer, 0, SignedTicket.SIZE)
    )
    const secretA = new Hash(
      new Uint8Array(arr.buffer, SignedTicket.SIZE, Hash.SIZE)
    )
    return new UnacknowledgedTicket(signedTicket, secretA)
  }

  async verifyChallenge(hashedKeyHalf: Uint8Array) {
    return u8aEquals(
      await hash(
        await hash(u8aConcat(this.secretA, hashedKeyHalf))
      ),
      (await this.signedTicket).ticket.challenge
    )
  }

  async verifySignature(pubKey: Uint8Array) {
    return await this.signedTicket.verifySignature(pubKey)
  }

  async verify(pubKey: Uint8Array, hashedKeyHalf: Uint8Array): Promise<AcknowledgedTicket> {
    const valid = (await this.verifyChallenge(hashedKeyHalf)) && (await this.verifySignature(pubKey))

    if (!valid) {
      throw new Error('Failure to validate')
    }
    return undefined
  }

  static SIZE(): number {
    return SignedTicket.SIZE + Hash.SIZE
  }
}

export default UnacknowledgedTicket
