import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'
import { u8aToHex } from '@hoprnet/hopr-utils'
import chalk from 'chalk'
import { getMyOpenChannelInstances } from '../utils/openChannels'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from '../utils'

export default class ListOpenChannels extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'openChannels'
  }

  public help() {
    return 'Lists your currently open channels'
  }

  private generateOutput(channelId: string, peerId?: string, status?: string): string {
    return [
      `ChannelId: ${styleValue(channelId, 'hash')}`,
      `PeerId: ${peerId ? styleValue(peerId, 'peerId') : chalk.gray('pre-opened')}`,
      `Status: ${status ? styleValue(status, 'highlight') : chalk.gray('UNKNOWN')}`,
    ]
      .map((str) => `\n - ${str}`)
      .join('')
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(): Promise<string | void> {
    try {
      const channels = await getMyOpenChannelInstances(this.node)
      const result: string[] = []

      if (channels.length === 0) {
        return `\nNo open channels found.`
      }

      for (const channel of channels) {
        const id = await channel.channelId
        const counterParty = channel.counterparty

        if (!counterParty) {
          result.push(this.generateOutput(u8aToHex(id)))
          continue
        }

        const peerId = await pubKeyToPeerId(await channel.offChainCounterparty)
        const status = await channel.status

        result.push(this.generateOutput(u8aToHex(id), peerId.toB58String(), status))
      }

      return result.join('\n\n')
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
