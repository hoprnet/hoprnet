import type Hopr from '..'
import { u8aEquals } from '@hoprnet/hopr-utils'
import { LevelUp } from 'levelup'
import { Ticket, Acknowledgement, SubmitTicketResponse, UnacknowledgedTicket } from '@hoprnet/hopr-core-ethereum'
import { UnAcknowledgedTickets, AcknowledgedTickets, AcknowledgedTicketsParse } from '../dbKeys'

/**
 * Get all unacknowledged tickets
 * @param filter optionally filter by signer
 * @returns an array of all unacknowledged tickets
 */
export async function getUnacknowledgedTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<UnacknowledgedTicket[]> {
  const tickets: UnacknowledgedTicket[] = []

  return new Promise((resolve, reject) => {
    db.createReadStream({
      gte: Buffer.from(UnAcknowledgedTickets(new Uint8Array(0x00)))
    })
      .on('error', (err: any) => reject(err))
      .on('data', async ({ value }: { value: Buffer }) => {
        if (value.buffer.byteLength !== UnacknowledgedTicket.SIZE()) return

        const unAckTicket = UnacknowledgedTicket.deserialize(value)

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !unAckTicket.verify(filter.signer)) {
          return
        }

        tickets.push(unAckTicket)
      })
      .on('end', () => resolve(tickets))
  })
}

/**
 * Delete unacknowledged tickets
 * @param filter optionally filter by signer
 */
export async function deleteUnacknowledgedTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  const tickets = await getUnacknowledgedTickets(db, filter)

  await db.batch(
    await Promise.all(
      tickets.map<any>(async (ticket) => {
        return {
          type: 'del',
          key: Buffer.from(UnAcknowledgedTickets(ticket.ticket.challenge.serialize()))
        }
      })
    )
  )
}

/**
 * Get all acknowledged tickets
 * @param filter optionally filter by signer
 * @returns an array of all acknowledged tickets
 */
export async function getAcknowledgements(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<
  {
    ackTicket: Acknowledgement
    index: Uint8Array
  }[]
> {
  const results: {
    ackTicket: Acknowledgement
    index: Uint8Array
  }[] = []

  return new Promise((resolve, reject) => {
    db.createReadStream({
      gte: Buffer.from(AcknowledgedTickets(new Uint8Array(0x00)))
    })
      .on('error', (err) => reject(err))
      .on('data', async ({ key, value }: { key: Buffer; value: Buffer }) => {
        if (value.buffer.byteLength !== Acknowledgement.SIZE) return

        const index = AcknowledgedTicketsParse(key)
        const ackTicket = Acknowledgement.deserialize(value)

        // if signer provided doesn't match our ticket's signer dont add it to the list
        if (filter?.signer && !ackTicket.ticket.verify(filter.signer)) {
          return
        }

        results.push({
          ackTicket,
          index
        })
      })
      .on('end', () => resolve(results))
  })
}

/**
 * Delete acknowledged tickets
 * @param filter optionally filter by signer
 */
export async function deleteAcknowledgements(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  const acks = await getAcknowledgements(db, filter)
  await db.batch(
    await Promise.all(
      acks.map<any>(async (ack) => {
        return {
          type: 'del',
          key: Buffer.from(AcknowledgedTickets((await ack.ackTicket.ticket).challenge.serialize()))
        }
      })
    )
  )
}

/**
 * Update acknowledged ticket in database
 * @param ackTicket Uint8Array
 * @param index Uint8Array
 */
export async function updateAcknowledgement(db: LevelUp, ackTicket: Acknowledgement, index: Uint8Array): Promise<void> {
  await db.put(Buffer.from(AcknowledgedTickets(index)), Buffer.from(ackTicket.serialize()))
}

/**
 * Delete acknowledged ticket in database
 * @param index Uint8Array
 */
export async function deleteAcknowledgement(node: Hopr, index: Uint8Array): Promise<void> {
  await node.db.del(Buffer.from(node._dbKeys.AcknowledgedTickets(index)))
}

/**
 * Submit acknowledged ticket and update database
 * @param ackTicket Uint8Array
 * @param index Uint8Array
 */
export async function submitAcknowledgedTicket(
  node: Hopr,
  ackTicket: Acknowledgement,
  index: Uint8Array
): Promise<SubmitTicketResponse> {
  try {
    const ethereum = node.paymentChannels
    const signedTicket = ackTicket.ticket
    const self = ethereum.getPublicKey()
    const counterparty = signedTicket.getSigner()
    const channel = ethereum.getChannel(self, counterparty)

    const result = await channel.submitTicket(ackTicket)
    // TODO look at result.status and actually do something
    await deleteAcknowledgement(node, index)
    return result
  } catch (err) {
    return {
      status: 'ERROR',
      error: err
    }
  }
}

/**
 * Get signed tickets, both unacknowledged and acknowledged
 * @param node
 * @param filter optionally filter by signer
 * @returns an array of signed tickets
 */
export async function getTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<Ticket[]> {
  return Promise.all([getUnacknowledgedTickets(db, filter), getAcknowledgements(db, filter)]).then(
    async ([unAcks, acks]) => {
      const unAckTickets = await Promise.all(unAcks.map((o) => o.ticket))
      const ackTickets = await Promise.all(acks.map((o) => o.ackTicket.ticket))

      return [...unAckTickets, ...ackTickets]
    }
  )
}

/**
 * Get signed tickets, both unacknowledged and acknowledged
 * @param node
 * @param filter optionally filter by signer
 * @returns an array of signed tickets
 */
export async function deleteTickets(
  db: LevelUp,
  filter?: {
    signer: Uint8Array
  }
): Promise<void> {
  await Promise.all([deleteUnacknowledgedTickets(db, filter), deleteAcknowledgements(db, filter)])
}
