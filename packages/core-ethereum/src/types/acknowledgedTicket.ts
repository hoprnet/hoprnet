import { Hash, Ticket } from '.'
import { Acknowledgement as IAcknowledgement } from '@hoprnet/hopr-core-connector-interface'
import { serializeToU8a } from '@hoprnet/hopr-utils'

class AcknowledgedTicket implements IAcknowledgement {
  constructor(
    readonly ticket: Ticket,
    readonly response: Hash,
    readonly preImage: Hash
  ) {}

  serialize(): Uint8Array{
    return serializeToU8a([
      [this.ticket.serialize(), Ticket.SIZE],
      [this.response.serialize(), Hash.SIZE],
      [this.preImage.serialize(), Hash.SIZE]
    ])
  }

  static get SIZE(): number {
    return Ticket.SIZE + Hash.SIZE + Hash.SIZE
  }
}

export default AcknowledgedTicket
