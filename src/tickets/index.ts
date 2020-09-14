import type HoprEthereum from '..'
import { AcknowledgedTicket, Public, Hash } from '../types'
import { u8aToHex } from '@hoprnet/hopr-utils'

/**
 * Store and get tickets stored by the node.
 */
class Tickets {
  constructor(public coreConnector: HoprEthereum) {}

  public async store(counterPartyPubKey: Public, ticket: AcknowledgedTicket): Promise<void> {
    const key = Buffer.from(
      this.coreConnector.dbKeys.AcknowledgedTicket(counterPartyPubKey, (await ticket.signedTicket).ticket.challenge)
    )
    const value = Buffer.from(ticket)

    await this.coreConnector.db.put(key, value)
  }

  public async getAll(): Promise<Map<string, AcknowledgedTicket>> {
    const tickets = new Map<string, AcknowledgedTicket>()

    return new Promise(async (resolve, reject) => {
      this.coreConnector.db
        .createReadStream({
          gte: Buffer.from(
            this.coreConnector.dbKeys.AcknowledgedTicket(
              new Public(Public.SIZE).fill(0x00),
              new Hash(Hash.SIZE).fill(0x00)
            )
          ),
          lte: Buffer.from(
            this.coreConnector.dbKeys.AcknowledgedTicket(
              new Public(Public.SIZE).fill(0xff),
              new Hash(Hash.SIZE).fill(0xff)
            )
          ),
        })
        .on('error', (err) => reject(err))
        .on('data', async ({ key, value }: { key: Buffer; value: Buffer }) => {
          const ticket = new AcknowledgedTicket(this.coreConnector, {
            bytes: value.buffer,
            offset: value.byteOffset,
          })

          tickets.set(u8aToHex(key), ticket)
        })
        .on('end', () => resolve(tickets))
    })
  }

  public async get(counterPartyPubKey: Public): Promise<Map<string, AcknowledgedTicket>> {
    const tickets = new Map<string, AcknowledgedTicket>()

    return new Promise(async (resolve, reject) => {
      this.coreConnector.db
        .createReadStream({
          gte: Buffer.from(
            this.coreConnector.dbKeys.AcknowledgedTicket(counterPartyPubKey, new Uint8Array(Hash.SIZE).fill(0x00))
          ),
          lte: Buffer.from(
            this.coreConnector.dbKeys.AcknowledgedTicket(counterPartyPubKey, new Uint8Array(Hash.SIZE).fill(0xff))
          ),
        })
        .on('error', (err) => reject(err))
        .on('data', async ({ value }: { value: Buffer }) => {
          const ticket = new AcknowledgedTicket(this.coreConnector, {
            bytes: value.buffer,
            offset: value.byteOffset,
          })

          tickets.set(u8aToHex((await ticket.signedTicket).ticket.challenge), ticket)
        })
        .on('end', () => resolve(tickets))
    })
  }
}

export default Tickets
