import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint, u8aToHex, pubKeyToPeerId, u8aEquals } from '@hoprnet/hopr-utils'
import chalk from 'chalk'
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
      const self = new types.Public(this.node.getId().pubKey.marshal())
      const channels = await this.node.paymentChannels.indexer.getChannelEntries(self)
      const result: string[] = []

      if (channels.length === 0) {
        return `\nNo open channels found.`
      }

      for (const { partyA, partyB, channelEntry } of channels) {
        const id = u8aToHex(await utils.getId(partyA, partyB))
        const selfIsPartyA = u8aEquals(self, partyA)
        const counterparty = selfIsPartyA ? partyB : partyA

        // do not print UNINITIALISED channels
        if (channelEntry.status === 'UNINITIALISED') continue

        const totalBalance = moveDecimalPoint(channelEntry.deposit.toString(), types.Balance.DECIMALS * -1)
        const myBalance = moveDecimalPoint(
          selfIsPartyA
            ? channelEntry.partyABalance.toString()
            : channelEntry.deposit.sub(channelEntry.partyABalance).toString(),
          types.Balance.DECIMALS * -1
        )
        const peerId = (await pubKeyToPeerId(counterparty)).toB58String()

        result.push(
          this.generateOutput({
            id,
            totalBalance,
            myBalance,
            peerId,
            status: channelEntry.status
          })
        )
      }

      return result.join('\n\n')
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
