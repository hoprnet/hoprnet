import { Hash, SignedTicket } from '.'
import { HASHED_SECRET_WIDTH } from '../hashedSecret'
import type { AcknowledgedTicket as IAcknowledgedTicket } from '@hoprnet/hopr-core-connector-interface'

class AcknowledgedTicket implements IAcknowledgedTicket {
  constructor(private signedTicket: SignedTicket, private response: Hash, private preImage: Hash) {}

  getSignedTicket(): SignedTicket {
    return this.signedTicket
  }

  getPreImage(): Hash {
    return this.preImage
  }

  getResponse(): Hash {
    return this.response
  }

  static SIZE(): number {
    return SignedTicket.SIZE + Hash.SIZE + HASHED_SECRET_WIDTH + 1
  }

  public serialize(): Uint8Array {
    const serialized = new Uint8Array(AcknowledgedTicket.SIZE())
    serialized.set(this.signedTicket.serialize(), 0)
    serialized.set(this.response, SignedTicket.SIZE)
    serialized.set(this.preImage, SignedTicket.SIZE + Hash.SIZE)
    return serialized
  }

  static deserialize(arr: Uint8Array): AcknowledgedTicket {
    const signedTicket = SignedTicket.deserialize(new Uint8Array(arr.buffer, arr.byteOffset, SignedTicket.SIZE))
    const response = new Hash(new Uint8Array(arr.buffer, arr.byteOffset + SignedTicket.SIZE, Hash.SIZE))
    const preImage = new Hash(
      new Uint8Array(arr.buffer, arr.byteOffset + SignedTicket.SIZE + Hash.SIZE, HASHED_SECRET_WIDTH)
    )
    return new AcknowledgedTicket(signedTicket, response, preImage)
  }
}

export default AcknowledgedTicket
