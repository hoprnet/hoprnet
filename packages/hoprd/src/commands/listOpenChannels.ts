import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { PublicKey } from '@hoprnet/hopr-utils'

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

  private generateOutput(
    id: string,
    channel: any,
    selfAddress
  ): string {
    const selfIsPartyA = channel.partyA.eq(selfAddress)
    const totalBalance = channel.totalBalance()
    const myBalance = selfIsPartyA ? channel.partyABalance : channel.partyBBalance
    const peerId = selfIsPartyA ? channel.partyB.toPeerId(): channel.partyA.toPeerId()

    return `
Channel:       ${styleValue(id, 'hash')}
CounterParty:  ${styleValue(peerId.toB58String(), 'peerId')}
Status:        ${styleValue(channel.status, 'highlight')}
Total Balance: ${styleValue(totalBalance, 'number')}
My Balance:    ${styleValue(myBalance, 'number')}
`
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(log): Promise<void> {
    log('fetching channels...')
    try {
      const selfPubKey = new PublicKey(this.node.getId().pubKey.marshal())
      const selfAddress = selfPubKey.toAddress()
      const channels = (await this.node.getChannelsOf(selfAddress)).filter((channel) => channel.status !== 'CLOSED')

      if (channels.length === 0) {
        return log(`\nNo open channels found.`)
      }

      const result: string[] = []
      // find counterpartys' peerIds
      for (const channel of channels) {
        const id = channel.getId()
        result.push(this.generateOutput(id.toHex(), channel, selfAddress))
      }

      return log(result.join('\n\n'))
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
