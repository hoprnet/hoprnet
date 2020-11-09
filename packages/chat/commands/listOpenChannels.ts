import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'
import { moveDecimalPoint, u8aToHex } from '@hoprnet/hopr-utils'
import chalk from 'chalk'
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

  private generateOutput(id: string, myBalance: string, totalBalance: string, peerId: string, status: string): string {
    const { NativeBalance, Balance } = this.node.paymentChannels.types
    return `
      Channel         ${styleValue(id, 'hash')}
      CounterParty    ${styleValue(peerId, 'peerId')}
      Status          ${styleValue(status, 'highlight')}
      Total Balance   ${styleValue(totalBalance, 'nativeBalance')}
      My Balance      ${styleValue(myBalance, 'nativeBalance')}
    `
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(): Promise<string | void> {
    try {
      const channels = await this.node.getAllOpenChannels()
      const { utils, types } = this.node.paymentChannels
      const result: string[] = []

      for (const channel of channels) {
        const id = u8aToHex(await channel.channelId)
        const status = await channel.status

        if (!channel.counterparty) {
          // Skip channels with no counterparty re #398
          continue
        }

        if (status === 'UNINITIALISED') {
          // Skip uninitialized channels re #398
          continue
        }

        const [offChainCounterparty, balance, balance_a] = await Promise.all([
          channel.offChainCounterparty,
          channel.balance,
          channel.balance_a
        ])

        const selfIsPartyA = utils.isPartyA(
          await this.node.paymentChannels.account.address,
          await utils.pubKeyToAccountId(channel.counterparty)
        )
        const totalBalance = moveDecimalPoint(balance.toString(), types.Balance.DECIMALS * -1)
        const myBalance = moveDecimalPoint(
          selfIsPartyA ? balance_a.toString() : balance.sub(balance_a).toString(),
          types.Balance.DECIMALS * -1
        )
        const peerId = (await pubKeyToPeerId(offChainCounterparty)).toB58String()

        result.push(this.generateOutput(id, myBalance, totalBalance, peerId, status))
      }
      if (result.length === 0) {
        return `\nNo open channels found.`
      }
      return result.join('\n\n')
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
