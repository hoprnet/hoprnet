import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import { countSignedTickets, styleValue, toSignedTickets } from './utils'
import { AbstractCommand } from './abstractCommand'
import { Logger, Balance, Acknowledgement } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hoprd.commands.redeemTickets')

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
   * @param query a ticket challenge
   */
  public async execute(): Promise<string | void> {
    try {
      const results = await this.node.getAcknowledgedTickets()
      if (results.length === 0) {
        return 'No unredeemed tickets found.'
      }

      log.info(`Redeeming ${results.length} tickets..`)

      const redeemedTickets: Acknowledgement[] = []
      let count = 0

      for (const ackTicket of results) {
        ++count
        const result = await this.node.submitAcknowledgedTicket(ackTicket)

        if (result.status === 'SUCCESS') {
          log.info(`Redeemed ticket ${count}`)
          redeemedTickets.push(ackTicket)
        } else {
          log.info(`Failed to redeem ticket ${count}`)
        }
      }

      const signedTickets = await toSignedTickets(redeemedTickets)
      const result = countSignedTickets(signedTickets)
      const total = moveDecimalPoint(result.total, Balance.DECIMALS * -1)

      return `Redeemed ${styleValue(redeemedTickets.length)} out of ${styleValue(
        results.length
      )} tickets with a sum of ${styleValue(total, 'number')} HOPR.`
    } catch (err) {
      log.error('Error while acknowledging tickets', err)
      return styleValue(err.message, 'failure')
    }
  }
}
