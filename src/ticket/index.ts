import type HoprEthereum from ".."
import type { Types } from "@hoprnet/hopr-core-connector-interface"
import { SignedTicket } from '../types'

class Ticket {
  static async store(coreConnector: HoprEthereum, channelId: Types.Hash, signedTicket: Types.SignedTicket<any, any>): Promise<void> {
    const { dbKeys, db } = coreConnector

    const key = Buffer.from(dbKeys.Ticket(channelId, signedTicket.ticket.challenge))
    const value = Buffer.from(signedTicket)

    await db.put(key, value)
  }

  static async get(coreConnector: HoprEthereum, channelId: Types.Hash): Promise<Map<string, SignedTicket>> {
    const { dbKeys, db } = coreConnector
    const tickets = new Map<string, SignedTicket>()

    return new Promise(async (resolve, reject) => {
      db.createReadStream({
        gt: Buffer.from(dbKeys.Ticket(channelId, new Uint8Array(SignedTicket.SIZE).fill(0x00))),
        lt: Buffer.from(dbKeys.Ticket(channelId, new Uint8Array(SignedTicket.SIZE).fill(0xff)))
      })
        .on('error', err => reject(err))
        .on('data', ({ key, value }: { key: Buffer; value: Buffer }) => {
          const signedTicket = new SignedTicket({
            bytes: value.buffer,
            offset: value.byteOffset
          })

          tickets.set(signedTicket.ticket.challenge.toHex(), signedTicket)
        })
        .on('end', () => resolve(tickets))
    })
  }
}

export default Ticket
