import { Hash, Ticket } from '.'
import { serializeToU8a, u8aSplit } from '..'

export class Acknowledgement {
  constructor(readonly ticket: Ticket, readonly response: Hash, readonly preImage: Hash) {}

  serialize(): Uint8Array {
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.response.serialize(), Hash.SIZE],
      [this.preImage.serialize(), Hash.SIZE]
    ])
  }

  static deserialize(arr: Uint8Array) {
    const components = u8aSplit(arr, [Ticket.SIZE, Hash.SIZE, Hash.SIZE])
    return new Acknowledgement(Ticket.deserialize(components[0]), new Hash(components[1]), new Hash(components[2]))
  }

  static get SIZE(): number {
    return Ticket.SIZE + Hash.SIZE + Hash.SIZE
  }
}

