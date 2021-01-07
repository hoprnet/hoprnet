import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint, u8aToHex, pubKeyToPeerId } from '@hoprnet/hopr-utils'
import chalk from 'chalk'
import { getMyOpenChannelInstances } from './utils/openChannels'
import { AbstractCommand } from './abstractCommand'
import { getPaddingLength, styleValue } from './utils'

export default class ListOpenChannels extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'channels'
  }

  public help() {
    return 'Lists your currently open channels'
  }

  private generateOutput({
    id,
    myBalance,
    totalBalance,
    peerId,
    status
  }: {
    id: string
    myBalance: string
    totalBalance: string
    peerId?: string
    status?: string
  }): string {
    const toDisplay: {
      name: string
      value: string
    }[] = [
      {
        name: 'Channel',
        value: styleValue(id, 'hash')
      },
      {
        name: 'CounterParty',
        value: peerId ? styleValue(peerId, 'peerId') : chalk.gray('pre-opened')
      },
      {
        name: 'Status',
        value: status ? styleValue(status, 'highlight') : chalk.gray('UNKNOWN')
      },
      {
        name: 'Total Balance',
        value: `${styleValue(totalBalance, 'number')} HOPR`
      },
      {
        name: 'My Balance',
        value: `${styleValue(myBalance, 'number')} HOPR`
      }
    ]

    const paddingLength = getPaddingLength(toDisplay.map((o) => o.name))

    return toDisplay.map((o) => `\n${o.name.padEnd(paddingLength)}:  ${o.value}`).join('')
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(): Promise<string | void> {
    try {
      const { utils, types } = this.node.paymentChannels
      const self = await this.node.paymentChannels.account.address
      const channels = await getMyOpenChannelInstances(this.node)
      const result: string[] = []

      if (channels.length === 0) {
        return `\nNo open channels found.`
      }

      for (const channel of channels) {
        const id = u8aToHex(await channel.channelId)

        if (!channel.counterparty) {
          result.push(
            this.generateOutput({
              id,
              totalBalance: '0',
              myBalance: '0'
            })
          )
        } else {
          const counterParty = await utils.pubKeyToAccountId(channel.counterparty)

          // @TODO: batch query
          const channelData = await Promise.all([
            channel.offChainCounterparty,
            channel.status,
            channel.balance,
            channel.balance_a
          ]).then(([offChainCounterparty, status, balance, balance_a]) => ({
            offChainCounterparty,
            status,
            balance,
            balance_a
          }))

          const status = channelData.status
          // do not print UNINITIALISED channels
          if (status === 'UNINITIALISED') continue

          const selfIsPartyA = utils.isPartyA(self, counterParty)
          const totalBalance = moveDecimalPoint(channelData.balance.toString(), types.Balance.DECIMALS * -1)
          const myBalance = moveDecimalPoint(
            selfIsPartyA ? channelData.balance_a.toString() : channelData.balance.sub(channelData.balance_a).toString(),
            types.Balance.DECIMALS * -1
          )
          const peerId = (await pubKeyToPeerId(channelData.offChainCounterparty)).toB58String()

          result.push(
            this.generateOutput({
              id,
              totalBalance,
              myBalance,
              peerId,
              status
            })
          )
        }
      }

      return result.join('\n\n')
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
