import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types, Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import type AbstractCommand from './abstractCommand'
import type Hopr from '@hoprnet/hopr-core'
import chalk from 'chalk'
import { u8aToHex, stringToU8a } from '@hoprnet/hopr-utils'

export default class Tickets implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * @param query channelId string to send message to
   */
  async execute(query?: string): Promise<void> {
    if (!query) {
      console.log(chalk.red(`\nChannel ID not provided.`))
      return
    }

    const signedTickets: Map<string, Types.SignedTicket<Types.Ticket, Types.Signature>> =
      // @ts-ignore
      // TODO: remove ignore once interface is updated
      await this.node.paymentChannels.ticket.get(this.node.paymentChannels, stringToU8a(query))

    if (signedTickets.size === 0) {
      console.log(chalk.yellow(`\nNo tickets found.`))
      return
    }

    const table = Array.from(signedTickets.values()).map(signedTicket => {
      const ticket = signedTicket.ticket

      return {
        id: u8aToHex(ticket.channelId),
        amount: ticket.amount.toString(),
      }
    })

    console.table(table)
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string) {
    this.node.paymentChannels.channel.getAll(
      this.node.paymentChannels,
      async (channel: ChannelInstance<HoprCoreConnector>) => u8aToHex(await channel.channelId),
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
