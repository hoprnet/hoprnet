import chalk from 'chalk'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

import { AbstractCommand } from './abstractCommand'

import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'
import { u8aToHex } from '@hoprnet/hopr-utils'

export default class ListOpenChannels extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() { return 'openChannels' }
  help() { return 'lists all currently open channels' }
  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(): Promise<string | void> {
    let str = `${chalk.yellow('ChannelId:'.padEnd(66, ' '))} - ${chalk.blue('PeerId:')}\n`

    try {
      await this.node.paymentChannels.channel.getAll(
        async (channel: ChannelInstance) => {
          const channelId = await channel.channelId
          if (!channel.counterparty) {
            str += `${chalk.yellow(u8aToHex(channelId))} - ${chalk.gray('pre-opened')}`
            return
          }

          const peerId = await pubKeyToPeerId(await channel.offChainCounterparty)

          str += `${chalk.yellow(u8aToHex(channelId))} - ${chalk.blue(peerId.toB58String())}\n`
          return
        },
        async (promises: Promise<void>[]) => {
          if (promises.length == 0) {
            str = chalk.yellow(`  There are currently no open channels.`)
            return
          }

          await Promise.all(promises)

          return
        }
      )
      return str
    } catch (err) {
      return chalk.red(err.message)
    }
  }
}
