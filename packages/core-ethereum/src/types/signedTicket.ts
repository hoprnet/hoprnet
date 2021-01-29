import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { u8aConcat } from '@hoprnet/hopr-utils'
import secp256k1 from 'secp256k1'
import { Signature, Ticket } from '../types'
import { Uint8ArrayE } from '../types/extended'
import { verify } from '../utils'

export const SIGNED_TICKET_SIZE = Signature.SIZE + Ticket.SIZE

class SignedTicket implements Types.SignedTicket {
  private _ticket?: Ticket
  private _signature?: Signature
  private _signer?: Uint8Array
  private _serialized: Uint8ArrayE

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
      this._serialized = new Uint8ArrayE(SIGNED_TICKET_SIZE)
    } else {
      this._serialized = new Uint8ArrayE(arr.bytes, arr.offset, SIGNED_TICKET_SIZE)
    }

    if (struct) {
      if (struct.signature) {
        this._serialized.set(struct.signature, this.signatureOffset - this._serialized.byteOffset)
      }

      if (struct.ticket) {
        const ticket = struct.ticket.toU8a()

        if (ticket.length == Ticket.SIZE) {
          this._serialized.set(ticket, this.ticketOffset - this._serialized.byteOffset)
        } else if (ticket.length < Ticket.SIZE) {
          this._serialized.set(u8aConcat(ticket, new Uint8Array(Ticket.SIZE - ticket.length)), this.ticketOffset - this._serialized.byteOffset)
        } else {
          throw Error(`Ticket is too big by ${ticket.length - Ticket.SIZE} elements.`)
        }
      }
    }
  }

  slice(begin = 0, end = SIGNED_TICKET_SIZE) {
    return this._serialized.subarray(begin, end)
  }

  subarray(begin = 0, end = SIGNED_TICKET_SIZE) {
    return new Uint8Array(this._serialized.buffer, begin + this._serialized.byteOffset, end - begin)
  }

  get ticketOffset(): number {
    return this._serialized.byteOffset + Signature.SIZE
  }

  get ticket(): Ticket {
    if (!this._ticket) {
      this._ticket = new Ticket({
        bytes: this._serialized.buffer,
        offset: this.ticketOffset
      })
    }

    return this._ticket
  }

  get signatureOffset(): number {
    return this._serialized.byteOffset
  }

  get signature(): Signature {
    if (!this._signature) {
      this._signature = new Signature({
        bytes: this._serialized.buffer,
        offset: this.signatureOffset
      })
    }

    return this._signature
  }

  get signer(): Promise<Uint8Array> {
    if (this._signer) {
      return Promise.resolve(this._signer)
    }

    return new Promise(async (resolve, reject) => {
      try {
        this._signer = secp256k1.ecdsaRecover(this.signature.signature, this.signature.recovery, await this.ticket.hash)
        return resolve(this._signer)
      } catch (err) {
        return reject(err)
      }
    })
  }

  async verify(pubKey: Uint8Array): Promise<boolean> {
    return verify(await this.ticket.hash, this.signature, pubKey)
  }

  serialize() {
    return this._serialized

  }

}

export default SignedTicket
