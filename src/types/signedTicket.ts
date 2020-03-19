import type { Types } from "@hoprnet/hopr-core-connector-interface"
import secp256k1 from 'secp256k1'
import { Signature, Ticket } from '.'
import { Uint8ArrayE } from '../types/extended'
import { u8aConcat } from '../core/u8a'
import { hashSync } from "../utils"

class SignedTicket extends Uint8ArrayE implements Types.SignedTicket<Ticket, Signature> {
  private _ticket?: Ticket
  private _signature?: Signature

  constructor(
    arr?: {
      bytes: Uint8Array
      offset: number
    },
    struct?: {
      signature: Signature
      ticket: Ticket
    }
  ) {
    if (arr != null && struct == null) {
      super(arr.bytes, arr.offset, SignedTicket.SIZE)
    } else if (arr == null && struct != null) {
      const ticket = struct.ticket.toU8a()

      if (ticket.length == Ticket.SIZE) {
        super(u8aConcat(struct.signature, ticket))
      } else if (ticket.length < Ticket.SIZE) {
        super(u8aConcat(struct.signature, ticket, new Uint8Array(Ticket.SIZE - ticket.length)))
      } else {
        throw Error(`Ticket is too big by ${ticket.length - Ticket.SIZE} elements.`)
      }
    } else {
      throw Error(`Invalid constructor arguments.`)
    }
  }

  get ticket(): Ticket {
    if (this._ticket == null) {
      const ticket = this.subarray(Signature.SIZE, Signature.SIZE + Ticket.SIZE)
      this._ticket = new Ticket({
        bytes: ticket.buffer,
        offset: ticket.byteOffset
      })
    }

    return this._ticket
  }

  get signature(): Signature {
    if (this._signature == null) {
      this._signature = new Signature({
        bytes: this.buffer,
        offset: this.byteOffset
      })
    }

    return this._signature
  }

  static get SIZE() {
    return Signature.SIZE + Ticket.SIZE
  }

  static create(
    arr?: {
      bytes: Uint8Array
      offset: number
    },
    struct?: {
      signature: Signature
      ticket: Ticket
    }
  ): SignedTicket {
    return new SignedTicket(arr, struct)
  }

  get signer(): Promise<Uint8Array> {
    let signer: Uint8Array

    try {
      signer = secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, hashSync(this.ticket))
      return Promise.resolve(signer)
    } catch (err) {
      return Promise.reject(err)
    }
  }
}

export default SignedTicket
