import BN from 'bn.js'
import chalk from 'chalk'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types, Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { u8aToHex, stringToU8a, moveDecimalPoint } from '@hoprnet/hopr-utils'
import type AbstractCommand from './abstractCommand'

export default class Tickets implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * @param query channelId string to send message to
   */
  async execute(query?: string): Promise<void> {
    const { Balance } = this.node.paymentChannels.types

    const signedTickets: Map<string, Types.SignedTicket> = await this.node.paymentChannels.tickets.get(
      stringToU8a(query)
    )

    if (signedTickets.size === 0) {
      console.log(chalk.yellow(`\nNo tickets found.`))
      return
    }

    const result = Array.from(signedTickets.values()).reduce<{
      tickets: {
        'amount (HOPR)': string
      }[]
      total: BN
    }>(
      (result, signedTicket) => {
        result.tickets.push({
          'amount (HOPR)': moveDecimalPoint(signedTicket.ticket.amount.toString(), Balance.DECIMALS * -1).toString(),
        })
        result.total = result.total.add(signedTicket.ticket.amount)

        return result
      },
      {
        tickets: [],
        total: new BN(0),
      }
    )

    console.table(result.tickets)
    console.log('Found', result.tickets.length, 'unredeemed tickets in channel ID', chalk.blue(query))
    console.log(
      'You will receive',
      chalk.yellow(moveDecimalPoint(result.total.toString(), Balance.DECIMALS * -1).toString()),
      'HOPR',
      'once you redeem them.'
    )
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string) {
    this.node.paymentChannels.channel.getAll(
      async (channel: ChannelInstance) => u8aToHex(await channel.channelId),
      async (channelIdsPromise: Promise<string>[]) => {
        let channelIds: string[] = []

        try {
          channelIds = await Promise.all(channelIdsPromise)
        } catch (err) {
          console.log(chalk.red(err.message))
          return cb(undefined, [[''], line])
        }

        if (channelIds.length < 1) {
          console.log(chalk.red(`\nNo open channels found.`))
          return cb(undefined, [[''], line])
        }

        const hits = query ? channelIds.filter((channelId: string) => channelId.startsWith(query)) : channelIds

        return cb(undefined, [hits.length ? hits.map((str: string) => `tickets ${str}`) : ['tickets'], line])
      }
    )
  }
}
