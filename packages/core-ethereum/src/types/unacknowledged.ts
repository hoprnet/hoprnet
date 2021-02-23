import type { UnacknowledgedTicket as IUnacknowledgedTicket } from '@hoprnet/hopr-core-connector-interface'
import { Hash, SignedTicket } from '.'

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

  static SIZE(): number {
    return SignedTicket.SIZE + Hash.SIZE
  }
}

export default UnacknowledgedTicket
