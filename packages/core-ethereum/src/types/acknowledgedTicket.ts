import { Hash, SignedTicket } from '.'
import { HASHED_SECRET_WIDTH } from '../hashedSecret'
import type { AcknowledgedTicket as IAcknowledgedTicket } from '@hoprnet/hopr-core-connector-interface'

class AcknowledgedTicket implements IAcknowledgedTicket{
  private _preImage: Hash

  constructor(
    private signedTicket: SignedTicket,
    private response: Hash,
    preImage?: Hash
  ) {
    if (preImage) {
      this._preImage = preImage
    }
  }

  /*
  subarray(begin: number = 0, end: number = AcknowledgedTicket.SIZE(this.paymentChannels)): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }
  */

  getSignedTicket(): SignedTicket {
    return this.signedTicket
  }

  getResponse(): Hash {
    return this.response
  }


  getPreImage(): Hash {
    return this._preImage
  }

  setPreImage(preImage: Hash) {
    this._preImage = preImage 
  }

  static SIZE(): number {
    return SignedTicket.SIZE + Hash.SIZE + HASHED_SECRET_WIDTH + 1
  }

  public serialize(): Uint8Array {
    const serialized = new Uint8Array(AcknowledgedTicket.SIZE())
    serialized.set(this.signedTicket, 0)
    serialized.set(this.response, SignedTicket.SIZE)
    if (this._preImage) {
      serialized.set(this._preImage, SignedTicket.SIZE + Hash.SIZE)
    }
    return serialized
  }

  static async deserialize(arr: Uint8Array): Promise<IAcknowledgedTicket> {
    if (arr.length != AcknowledgedTicket.SIZE()){
      throw new Error('Cannot deserialize, bad length')
    }

    const signedTicket = await SignedTicket.create({
        bytes: arr.buffer,
        offset: arr.byteOffset 
      })

    const response = new Hash(
      new Uint8Array(arr.buffer, arr.byteOffset + SignedTicket.SIZE, Hash.SIZE)
    )

    const preImage = new Hash(
      new Uint8Array(arr.buffer, arr.byteOffset + SignedTicket.SIZE + Hash.SIZE, HASHED_SECRET_WIDTH)
    )

    return new AcknowledgedTicket(signedTicket, response, preImage)
  }


}

export default AcknowledgedTicket
