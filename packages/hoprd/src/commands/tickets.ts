import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import { AbstractCommand } from './abstractCommand'
import { countSignedTickets, toSignedTickets, styleValue } from './utils'

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

  public async execute(log): Promise<void> {
    try {
      const ackTickets = await this.node.getAcknowledgedTickets()

      if (ackTickets.length === 0) {
        log('No tickets found.')
        return
      }

      const unredeemedResults = countSignedTickets(await toSignedTickets(ackTickets))
      const unredeemedAmount = moveDecimalPoint(unredeemedResults.total.toString(), Balance.DECIMALS * -1)

      log(`Found ${styleValue(unredeemedResults.tickets.length)} unredeemed tickets with a sum of ${styleValue(
        unredeemedAmount,
        'number'
      )} HOPR.`)
    } catch (err) {
      log(styleValue(err.message, 'failure'))
    }
  }
}
