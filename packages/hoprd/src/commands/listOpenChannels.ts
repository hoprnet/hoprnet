import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { PublicKey } from '@hoprnet/hopr-utils'
import type { Address, ChannelEntry } from '@hoprnet/hopr-utils'

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

  private async generateOutput(id: string, channel: ChannelEntry, selfAddress: Address): Promise<string> {
    const selfIsPartyA = channel.partyA.eq(selfAddress)
    const totalBalance = channel.totalBalance()
    const myBalance = selfIsPartyA ? channel.partyABalance : channel.partyBBalance

    const peerId = await this.node.getPublicKeyOf(selfIsPartyA ? channel.partyB : channel.partyA)

    return `
Channel:       ${styleValue(id, 'hash')}
CounterParty:  ${styleValue(peerId.toPeerId().toB58String(), 'peerId')}
Status:        ${styleValue(channel.status, 'highlight')}
Total Balance: ${styleValue(totalBalance.toFormattedString(), 'number')}
My Balance:    ${styleValue(myBalance.toFormattedString(), 'number')}
`
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(log: (str: string) => void): Promise<void> {
    log('fetching channels...')
    try {
      const selfPubKey = new PublicKey(this.node.getId().pubKey.marshal())
      const selfAddress = selfPubKey.toAddress()
      const channels = (await this.node.getChannelsOf(selfAddress)).filter((channel) => channel.status !== 'CLOSED')

      if (channels.length == 0) {
        return log(`\nNo open channels found.`)
      }

      const result: string[] = []
      // find counterpartys' peerIds
      for (const channel of channels) {
        const id = channel.getId()
        result.push(await this.generateOutput(id.toHex(), channel, selfAddress))
      }

      return log(result.join('\n\n'))
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
