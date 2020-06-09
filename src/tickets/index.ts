import type HoprEthereum from '..'
import { SignedTicket, Hash } from '../types'

/**
 * Store and get tickets stored by the node.
 */
class Tickets {
  static async store(coreConnector: HoprEthereum, channelId: Hash, signedTicket: SignedTicket): Promise<void> {
    const { dbKeys, db } = coreConnector

    const key = Buffer.from(dbKeys.Ticket(channelId, signedTicket.ticket.challenge))
    const value = Buffer.from(signedTicket)

    await db.put(key, value)
  }

  static async get(coreConnector: HoprEthereum, channelId: Hash): Promise<Map<string, SignedTicket>> {
    const { dbKeys, db } = coreConnector
    const tickets = new Map<string, SignedTicket>()

    return new Promise(async (resolve, reject) => {
      db.createReadStream({
        gte: Buffer.from(dbKeys.Ticket(channelId, new Uint8Array(SignedTicket.SIZE).fill(0x00))),
        lte: Buffer.from(dbKeys.Ticket(channelId, new Uint8Array(SignedTicket.SIZE).fill(0xff))),
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
