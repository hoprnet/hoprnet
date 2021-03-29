import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint, pubKeyToPeerId, u8aEquals } from '@hoprnet/hopr-utils'
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
      const { utils, types, indexer } = this.node.paymentChannels
      const selfPubKey = new types.Public(this.node.getId().pubKey.marshal())
      const selfAddress = await selfPubKey.toAddress()
      const channels = (await this.node.paymentChannels.indexer.getChannelsOf(selfAddress))
        // do not print CLOSED channels
        .filter((channel) => channel.getStatus() !== 'CLOSED')
      const result: string[] = []

      if (channels.length === 0) {
        return `\nNo open channels found.`
      }

<<<<<<< HEAD
      for (const { partyA, partyB, channelEntry } of channels) {
        const id = (await utils.getId(await partyA.toAddress(), await partyB.toAddress())).toHex()
        const selfIsPartyA = u8aEquals(self, partyA)
        const counterparty = selfIsPartyA ? partyB : partyA
=======
      for (const channel of channels) {
        const id = await utils.getId(channel.parties[0], channel.parties[1])
        const [partyA, partyB] = utils.isPartyA(channel.parties[0], channel.parties[1])
          ? [channel.parties[0], channel.parties[1]]
          : [channel.parties[1], channel.parties[0]]
        const selfIsPartyA = u8aEquals(selfAddress.serialize(), partyA.serialize())
        const counterpartyPubKey = await indexer.getPublicKeyOf(selfIsPartyA ? partyB : partyA)
>>>>>>> 6b0fce304a7530c541e600131ec79f96b2b75aab

        const totalBalance = moveDecimalPoint(channel.deposit.toString(), types.Balance.DECIMALS * -1)
        const myBalance = moveDecimalPoint(
          selfIsPartyA ? channel.partyABalance.toString() : channel.deposit.sub(channel.partyABalance).toString(),
          types.Balance.DECIMALS * -1
        )
        const peerId = (await pubKeyToPeerId(counterpartyPubKey)).toB58String()

        result.push(
          this.generateOutput({
            id: u8aToHex(id),
            totalBalance,
            myBalance,
            peerId,
            status: channel.getStatus()
          })
        )
      }

      return result.join('\n\n')
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
