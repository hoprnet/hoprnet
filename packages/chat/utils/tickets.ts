import type { Types } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { u8aToHex } from '@hoprnet/hopr-utils'

export async function getSignedTickets(ackTickets: Types.AcknowledgedTicket[]): Promise<Types.SignedTicket[]> {
  return Promise.all(ackTickets.map((ackTicket) => ackTicket.signedTicket))
}

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
        amount: signedTicket.ticket.amount.toString(10),
      })
      result.total = result.total.add(signedTicket.ticket.amount)

      return result
    },
    {
      tickets: [] as {
        challange: string
        amount: string
      }[],
      total: new BN(0),
    }
  )

  return {
    tickets,
    total: total.toString(10),
  }
}
