import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import chalk from 'chalk'
import { AbstractCommand } from './abstractCommand'
import { countSignedTickets, toSignedTickets } from '../utils'

export default class Tickets extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'tickets'
  }

  public help() {
    return 'lists information about redeemed and unredeemed tickets'
  }

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
      const unredeemedResults = countSignedTickets(await toSignedTickets(ackTickets))
      const unredeemedAmount = moveDecimalPoint(unredeemedResults.total.toString(), Balance.DECIMALS * -1)

      return `Found ${chalk.blue(unredeemedResults.tickets.length)} unredeemed tickets with a sum of ${chalk.blue(
        unredeemedAmount
      )} HOPR.`
    } catch (err) {
      return chalk.red(err.message)
    }
  }
}
