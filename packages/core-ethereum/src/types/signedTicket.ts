import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aConcat } from '@hoprnet/hopr-utils'
import secp256k1 from 'secp256k1'
import { Signature, Ticket, PublicKey } from '../types'
import { Uint8ArrayE } from '../types/extended'
import { verify } from '../utils'

class SignedTicket extends Uint8ArrayE implements Types.SignedTicket {
  private _signature?: Signature
  private _signer?: PublicKey

  constructor(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature?: Signature
      ticket?: Ticket
    }
  ) {
    if (!arr) {
      super(SignedTicket.SIZE)
    } else {
      super(arr.bytes, arr.offset, SignedTicket.SIZE)
    }

    if (struct) {
      if (struct.signature) {
        this.set(struct.signature, this.signatureOffset - this.byteOffset)
      }

      if (struct.ticket) {
        const ticket = struct.ticket.toU8a()

        if (ticket.length == Ticket.SIZE) {
          this.set(ticket, this.ticketOffset - this.byteOffset)
        } else if (ticket.length < Ticket.SIZE) {
          this.set(u8aConcat(ticket, new Uint8Array(Ticket.SIZE - ticket.length)), this.ticketOffset - this.byteOffset)
        } else {
          throw Error(`Ticket is too big by ${ticket.length - Ticket.SIZE} elements.`)
        }
      }
    }
  }

  slice(begin = 0, end = SignedTicket.SIZE) {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = SignedTicket.SIZE) {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  get ticketOffset(): number {
    return this.byteOffset + Signature.SIZE
  }

  get ticket(): Ticket {
    return new Ticket({
      bytes: this.buffer,
      offset: this.ticketOffset
    })
  }

  get signatureOffset(): number {
    return this.byteOffset
  }

  get signature(): Signature {
    if (!this._signature) {
      this._signature = new Signature({
        bytes: this.buffer,
        offset: this.signatureOffset
      })
    }

    return this._signature
  }

  get signer(): Promise<PublicKey> {
    if (this._signer) {
      return Promise.resolve(this._signer)
    }

    return new Promise(async (resolve, reject) => {
      try {
        this._signer = new PublicKey(
          secp256k1.ecdsaRecover(
            this.signature.signature,
            this.signature.recovery,
            (await this.ticket.hash).serialize()
          )
        )
        return resolve(this._signer)
      } catch (err) {
        return reject(err)
      }
    })
  }

  async verify(pubKey: PublicKey): Promise<boolean> {
    return verify((await this.ticket.hash).serialize(), this.signature, pubKey.serialize())
  }

  static get SIZE() {
    return Signature.SIZE + Ticket.SIZE
  }

  static create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      signature?: Signature
      ticket?: Ticket
    }
  ): Promise<SignedTicket> {
    return Promise.resolve(new SignedTicket(arr, struct))
  }
}

export default SignedTicket
