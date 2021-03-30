import { Hash, SignedTicket } from '.'

// @TODO this is a duplicate of the same class in hopr-core
class AcknowledgedTicket extends Uint8Array {
  private _signedTicket: SignedTicket

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signedTicket?: SignedTicket
      response?: Hash
      preImage?: Hash
      redeemed?: boolean
    }
  ) {
    if (!arr) {
      super(AcknowledgedTicket.SIZE)
    } else {
      super(arr.bytes, arr.offset, AcknowledgedTicket.SIZE)
    }

    if (struct) {
      if (struct.signedTicket) {
        this.set(struct.signedTicket, this.signedTicketOffset - this.byteOffset)
        this._signedTicket = struct.signedTicket
      }

      if (struct.response) {
        this.set(struct.response.serialize(), this.responseOffset - this.byteOffset)
      }

      if (struct.preImage) {
        this.set(struct.preImage.serialize(), this.preImageOffset - this.byteOffset)
      }

      if (struct.redeemed) {
        this.set([struct.redeemed ? 1 : 0], this.redeemedOffset - this.byteOffset)
      }
    }
  }

  subarray(begin: number = 0, end: number = AcknowledgedTicket.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get signedTicketOffset(): number {
    return this.byteOffset
  }

  get signedTicket(): Promise<SignedTicket> {
    if (this._signedTicket) {
      return Promise.resolve(this._signedTicket)
    }

    return new Promise<SignedTicket>(async (resolve) => {
      this._signedTicket = await SignedTicket.create({
        bytes: this.buffer,
        offset: this.signedTicketOffset
      })

      resolve(this._signedTicket)
    })
  }

  get responseOffset(): number {
    return this.byteOffset + SignedTicket.SIZE
  }

  get response(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.responseOffset, Hash.SIZE))
  }

  get preImageOffset(): number {
    return this.byteOffset + SignedTicket.SIZE + Hash.SIZE
  }

  get preImage(): Hash {
    return new Hash(new Uint8Array(this.buffer, this.preImageOffset, Hash.SIZE))
  }

  set preImage(_preImage: Hash) {
    this.set(_preImage.serialize(), this.preImageOffset - this.byteOffset)
  }

  get redeemedOffset(): number {
    return this.byteOffset + SignedTicket.SIZE + Hash.SIZE + Hash.SIZE
  }

  get redeemed(): boolean {
    return this[this.redeemedOffset - this.byteOffset] == 0 ? false : true
  }

  set redeemed(_redeemed: boolean) {
    this.set([_redeemed ? 1 : 0], this.redeemedOffset - this.byteOffset)
  }

  static get SIZE(): number {
    return SignedTicket.SIZE + Hash.SIZE + Hash.SIZE + 1
  }

  static create(
    arr?: { bytes: ArrayBuffer; offset: number },
    struct?: { signedTicket?: SignedTicket; response?: Hash; preImage?: Hash; redeemed?: boolean }
  ) {
    return new AcknowledgedTicket(arr, struct)
  }
}

export default AcknowledgedTicket
