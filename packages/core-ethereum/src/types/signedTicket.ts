import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aConcat } from '@hoprnet/hopr-utils'
import secp256k1 from 'secp256k1'
import { Signature, Ticket } from '../types'
import { Uint8ArrayE } from '../types/extended'
import { verify } from '../utils'

class SignedTicket extends Uint8ArrayE implements Types.SignedTicket {
  private _ticket?: Ticket
  private _signature?: Signature
  private _signer?: Uint8Array

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
    if (arr == null) {
      super(SignedTicket.SIZE)
    } else {
      super(arr.bytes, arr.offset, SignedTicket.SIZE)
    }

    if (struct != null) {
      if (struct.signature != null) {
        this.set(struct.signature, this.signatureOffset - this.byteOffset)
      }

      if (struct.ticket != null) {
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

  get ticketOffset(): number {
    return this.byteOffset + Signature.SIZE
  }

  get ticket(): Ticket {
    if (this._ticket == null) {
      this._ticket = new Ticket({
        bytes: this.buffer,
        offset: this.ticketOffset,
      })
    }

    return this._ticket
  }

  get signatureOffset(): number {
    return this.byteOffset
  }

  get signature(): Signature {
    if (this._signature == null) {
      this._signature = new Signature({
        bytes: this.buffer,
        offset: this.signatureOffset,
      })
    }

    return this._signature
  }

  get signer(): Promise<Uint8Array> {
    if (this._signer != null) {
      return Promise.resolve(this._signer)
    }

    return new Promise(async (resolve, reject) => {
      try {
        this._signer = secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, await this.ticket.hash)

        return resolve(this.signer)
      } catch (err) {
        return reject(err)
      }
    })
  }

  async verify(pubKey: Uint8Array): Promise<boolean> {
    return verify(await this.ticket.hash, this.signature, pubKey)
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
