import type { AcknowledgedTicket, Ticket } from '@hoprnet/hopr-utils'
import BN from 'bn.js'

/**
 * Retrieves all signed tickets from the given acknowledged tickets.
 *
 * @param ackTickets
 * @returns a promise that resolves into an array of signed tickets
 */
export async function toSignedTickets(ackTickets: AcknowledgedTicket[]): Promise<Ticket[]> {
  return Promise.all(ackTickets.map((ackTicket) => ackTicket.ticket))
}

/**
 * Derive various data from the given signed tickets.
 *
 * @param signedTickets
 * @returns the total amount of tokens in the tickets & more
 */
export function countSignedTickets(
  signedTickets: Ticket[]
): {
  tickets: {
    challange: string
    amount: string
  }[]
  total: string
} {
  const { tickets, total } = signedTickets.reduce(
    (result, signedTicket) => {
      result.tickets.push({
        challange: signedTicket.challenge.toHex(),
        amount: signedTicket.amount.toBN().toString(10)
      })
      result.total = result.total.add(signedTicket.amount.toBN())

      return result
    },
    {
      tickets: [] as {
        challange: string
        amount: string
      }[],
      total: new BN(0)
    }
  )

  return {
    tickets,
    total: total.toString(10)
  }
}
