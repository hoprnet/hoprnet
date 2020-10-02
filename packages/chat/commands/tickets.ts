import chalk from 'chalk'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import { SendMessageBase } from './sendMessage'
import { countSignedTickets, getSignedTickets } from '../utils'

export default class Tickets extends SendMessageBase {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super(node)
  }

  name() {
    return 'tickets'
  }

  help() {
    return 'lists information about redeemed and unredeemed tickets'
  }

  /**
   * @param query channelId to query tickets for
   */
  public async execute(): Promise<string | void> {
    try {
      const { Balance } = this.node.paymentChannels.types

      const results = await this.node.getAcknowledgedTickets().then((tickets) => {
        return tickets.filter((ticket) => !ticket.ackTicket.redeemed)
      })

      if (results.length === 0) {
        return 'No tickets found.'
      }

      const ackTickets = results.map((o) => o.ackTicket)
      const unredeemedResults = countSignedTickets(await getSignedTickets(ackTickets))
      const unredeemedAmount = moveDecimalPoint(unredeemedResults.total.toString(), Balance.DECIMALS * -1)

      return `Found ${chalk.blue(unredeemedResults.tickets.length)} unredeemed tickets with a sum of ${chalk.blue(
        unredeemedAmount
      )} HOPR.`
    } catch (err) {
      return chalk.red(err.message)
    }
  }
}
