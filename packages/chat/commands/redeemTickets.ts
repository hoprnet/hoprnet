import chalk from 'chalk'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import { AbstractCommand } from './abstractCommand'
import { countSignedTickets, getSignedTickets } from '../utils'

export default class RedeemTickets extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'redeemTickets'
  }

  help() {
    return 'redeem tickets'
  }

  /**
   * @param query a ticket challange
   */
  async execute(): Promise<string | void> {
    const { paymentChannels } = this.node
    const { Balance } = paymentChannels.types

    try {
      // get only unredeemed tickets
      const results = await this.node.getAcknowledgedTickets().then((tickets) => {
        return tickets.filter((ticket) => !ticket.ackTicket.redeemed)
      })

      if (results.length === 0) {
        return 'No unredeemed tickets found.'
      }

      const redeemedTickets: Types.AcknowledgedTicket[] = []
      for (const { ackTicket, index } of results) {
        const result = await this.node.submitAcknowledgedTicket(ackTicket, index)

        if (result.status === 'SUCCESS') {
          redeemedTickets.push(ackTicket)
        }
      }

      const signedTickets = await getSignedTickets(redeemedTickets)
      const result = countSignedTickets(signedTickets)
      const total = moveDecimalPoint(result.total, Balance.DECIMALS * -1)

      return `Redeemed ${chalk.blue(redeemedTickets.length)} out of ${chalk.blue(
        results.length
      )} tickets with a sum of ${chalk.blue(total)} HOPR.`
    } catch (err) {
      return chalk.red(err.message)
    }
  }
}
