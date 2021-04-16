import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import { AbstractCommand } from './abstractCommand'
import { countSignedTickets, toSignedTickets, styleValue } from './utils'
import { Logger } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hoprd.commands.tickets')

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
      log.error('Error while displaying information about tickets', err)
      return styleValue(err.message, 'failure')
    }
  }
}
