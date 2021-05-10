import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import BN from 'bn.js'
import type { AcknowledgedTicket, Ticket } from '@hoprnet/hopr-utils'


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
function countSignedTickets(
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

export default class Tickets extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'tickets'
  }

  public help() {
    return 'Displays information about your redeemed and unredeemed tickets'
  }

  public async execute(): Promise<string | void> {
    try {
      const ackTickets = await this.node.getAcknowledgedTickets()

      if (ackTickets.length === 0) {
        return 'No tickets found.'
      }

      const unredeemedResults = countSignedTickets(await toSignedTickets(ackTickets))
      const unredeemedAmount = moveDecimalPoint(unredeemedResults.total.toString(), Balance.DECIMALS * -1)

      return `Found ${styleValue(unredeemedResults.tickets.length)} unredeemed tickets with a sum of ${styleValue(
        unredeemedAmount,
        'number'
      )} HOPR.`
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
