import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aToHex } from '@hoprnet/hopr-utils'

/**
 * Retrieves all signed tickets from the given acknowledged tickets.
 *
 * @param ackTickets
 * @returns a promise that resolves into an array of signed tickets
 */
export async function toSignedTickets(ackTickets: Types.AcknowledgedTicket[]): Promise<Types.SignedTicket[]> {
  return Promise.all(ackTickets.map((ackTicket) => ackTicket.signedTicket))
}

/**
 * Derive various data from the given signed tickets.
 *
 * @param signedTickets
 * @returns the total amount of tokens in the tickets & more
 */
export function countSignedTickets(
  signedTickets: Types.SignedTicket[]
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
        challange: u8aToHex(signedTicket.ticket.challenge),
        amount: signedTicket.ticket.amount.toString(10)
      })
      result.total = result.total.add(signedTicket.ticket.amount)

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
