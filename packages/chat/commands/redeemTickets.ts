import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import { countSignedTickets, styleValue, toSignedTickets } from '../utils'
import { AbstractCommand } from './abstractCommand'

export default class RedeemTickets extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
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
  public async execute(): Promise<string | void> {
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

      console.log(`Redeeming ${styleValue(results.length)} tickets..`)

      const redeemedTickets: Types.AcknowledgedTicket[] = []
      let count = 0

      for (const { ackTicket, index } of results) {
        ++count
        const result = await this.node.submitAcknowledgedTicket(ackTicket, index)

        if (result.status === 'SUCCESS') {
          console.log(`Redeemed ticket ${styleValue(count)}`)
          redeemedTickets.push(ackTicket)
        } else {
          console.log(`Failed to redeem ticket ${styleValue(count)}`)
        }
      }

      const signedTickets = await toSignedTickets(redeemedTickets)
      const result = countSignedTickets(signedTickets)
      const total = moveDecimalPoint(result.total, Balance.DECIMALS * -1)

      return `Redeemed ${styleValue(redeemedTickets.length)} out of ${styleValue(
        results.length
      )} tickets with a sum of ${styleValue(total, 'number')} HOPR.`
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
