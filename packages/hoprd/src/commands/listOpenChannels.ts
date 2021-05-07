import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint, u8aEquals } from '@hoprnet/hopr-utils'
import chalk from 'chalk'
import { AbstractCommand } from './abstractCommand'
import { getPaddingLength, styleValue } from './utils'
import { PublicKey, Balance } from '@hoprnet/hopr-utils'

export default class ListOpenChannels extends AbstractCommand {
  constructor(public node: Hopr) {
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
      const selfPubKey = new PublicKey(this.node.getId().pubKey.marshal())
      const selfAddress = selfPubKey.toAddress()
      const channels = (await this.node.getChannelsOf(selfAddress))
        // do not print CLOSED channels
        .filter((channel) => channel.status !== 'CLOSED')
      const result: string[] = []

      if (channels.length === 0) {
        return `\nNo open channels found.`
      }

      // find counterpartys' peerIds
      for (const channel of channels) {
        const id = channel.getId()
        const selfIsPartyA = u8aEquals(selfAddress.serialize(), channel.partyA.serialize())
        const counterpartyPubKey = await this.node.getPublicKeyOf(selfIsPartyA ? channel.partyB : channel.partyA)
        // counterparty has not initialized
        if (!counterpartyPubKey) continue

        const totalBalance = channel.partyABalance.toBN().add(channel.partyABalance.toBN())
        const myBalance = moveDecimalPoint(
          selfIsPartyA ? channel.partyABalance.toString() : totalBalance.sub(channel.partyABalance.toBN()).toString(),
          Balance.DECIMALS * -1
        )
        const peerId = counterpartyPubKey.toPeerId().toB58String()

        result.push(
          this.generateOutput({
            id: id.toHex(),
            totalBalance: moveDecimalPoint(totalBalance.toString(), Balance.DECIMALS * -1),
            myBalance,
            peerId,
            status: channel.status
          })
        )
      }

      return result.join('\n\n')
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
