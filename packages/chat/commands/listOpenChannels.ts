import chalk from 'chalk'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

import { AbstractCommand } from './abstractCommand'
import { getMyOpenChannelInstances } from '../utils/openChannels'

import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'
import { u8aToHex } from '@hoprnet/hopr-utils'

export default class ListOpenChannels extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'openChannels'
  }
  help() {
    return 'lists all currently open channels'
  }

  private generateOutput(channelId: string, peerId?: string, status?: string): string {
    return [
      `ChannelId: ${chalk.yellow(channelId)}`,
      `PeerId: ${peerId ? chalk.blue(peerId) : chalk.gray('pre-opened')}`,
      `Status: ${status ? chalk.blue(status) : chalk.gray('UNKNOWN')}`,
    ].join(' - ')
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(): Promise<string | void> {
    try {
      const channels = await getMyOpenChannelInstances(this.node)
      const result: string[] = []

      if (channels.length === 0) {
        return chalk.yellow(`\nNo open channels found.`)
      }

      for (const channel of channels) {
        const id = await channel.channelId
        const counterParty = channel.counterparty

        if (!counterParty) {
          result.push(this.generateOutput(u8aToHex(id)))
          continue
        }

        const peerId = await pubKeyToPeerId(await channel.offChainCounterparty)
        const state = await channel.state.then((state) => {
          if (state.isActive) {
            return 'OPEN'
          } else if (state.isPending) {
            return 'CLOSING'
          } else if (state.isFunded) {
            return 'FUNDED'
          } else {
            return 'CLOSED'
          }
        })

        result.push(this.generateOutput(u8aToHex(id), peerId.toB58String(), state))
      }

      return result.join('\n')
    } catch (err) {
      return chalk.red(err.message)
    }
  }
}
