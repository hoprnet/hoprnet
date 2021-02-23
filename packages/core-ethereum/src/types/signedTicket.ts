import type { SignedTicket as ISignedTicket } from '@hoprnet/hopr-core-connector-interface'
import secp256k1 from 'secp256k1'
import { Signature, Ticket } from '../types'
import { verify } from '../utils'

class SignedTicket implements ISignedTicket {
  constructor(readonly ticket: Ticket, readonly signature: Signature) {}

  public serialize(): Uint8Array {
    const serialized = new Uint8Array(SignedTicket.SIZE)
    serialized.set(this.signature, 0)
    serialized.set(this.ticket.serialize(), Signature.SIZE)
    return serialized
  }

  static deserialize(arr: Uint8Array): SignedTicket {
    const buffer = arr.buffer
    let i = arr.byteOffset
    const signature = new Signature({ bytes: buffer, offset: i })
    i += Signature.SIZE
    const ticket = Ticket.deserialize(new Uint8Array(buffer, i, Ticket.SIZE()))
    return new SignedTicket(ticket, signature)
  }

  async getSigner(): Promise<Uint8Array> {
    return secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, await this.ticket.hash)
  }

  async verifySignature(pubKey: Uint8Array): Promise<boolean> {
    return verify(await this.ticket.hash, this.signature, pubKey)
  }

  static get SIZE() {
    return Signature.SIZE + Ticket.SIZE()
  }
}

export default SignedTicket
