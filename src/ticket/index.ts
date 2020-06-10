import type HoprEthereum from '..'
import { SignedTicket, Hash } from '../types'

/**
 * Store and get tickets stored by the node.
 */
class Tickets {
  constructor(public coreConnector: HoprEthereum) {}

  async store(channelId: Hash, signedTicket: SignedTicket): Promise<void> {
    const key = Buffer.from(this.coreConnector.dbKeys.Ticket(channelId, signedTicket.ticket.challenge))
    const value = Buffer.from(signedTicket)

    await this.coreConnector.db.put(key, value)
  }

  async get(channelId: Hash): Promise<Map<string, SignedTicket>> {
    const tickets = new Map<string, SignedTicket>()

    return new Promise(async (resolve, reject) => {
      this.coreConnector.db
        .createReadStream({
          gte: Buffer.from(this.coreConnector.dbKeys.Ticket(channelId, new Uint8Array(SignedTicket.SIZE).fill(0x00))),
          lte: Buffer.from(this.coreConnector.dbKeys.Ticket(channelId, new Uint8Array(SignedTicket.SIZE).fill(0xff))),
        })
        .on('error', (err) => reject(err))
        .on('data', ({ value }: { value: Buffer }) => {
          const signedTicket = new SignedTicket({
            bytes: value.buffer,
            offset: value.byteOffset,
          })

          tickets.set(signedTicket.ticket.challenge.toHex(), signedTicket)
        })
        .on('end', () => resolve(tickets))
    })
  }
}

export default Tickets
