import type Hopr from '@hoprnet/hopr-core'
import { AcknowledgedTicket, moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import { styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'

export default class RedeemTickets extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'redeemTickets'
  }

  public help() {
    return 'Redeems your tickets'
  }

  /**
   * @param query a ticket challange
   */
  public async execute(log): Promise<void> {
    try {
      const statistics = await this.node.getTicketStatistics()
      if (statistics.unredeemed === 0) {
        return 'No unredeemed tickets found.'
      }
      console.log(`Redeeming ${styleValue(statistics.unredeemed)} tickets..`)
      const result = await this.node.redeemAllTickets()
      log(`Redeemed ${result.redeemed} tickets with a sum of ${styleValue(result.total, 'number')} HOPR.`)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
