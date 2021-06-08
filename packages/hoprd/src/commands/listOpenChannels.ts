import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { PublicKey } from '@hoprnet/hopr-utils'
import type { ChannelEntry } from '@hoprnet/hopr-utils'

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

  private async generateOutput(id: string, channel: ChannelEntry): Promise<string> {
    const peerId = await this.node.getPublicKeyOf(channel.destination)
    return `
Outgoing Channel:       ${styleValue(id, 'hash')}
To:                     ${styleValue(peerId.toPeerId().toB58String(), 'peerId')}
Status:                 ${styleValue(channel.status, 'highlight')}
Balance:                ${styleValue(channel.balance.toFormattedString(), 'number')}
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
      const channels = (await this.node.getChannelsFrom(selfAddress)).filter((channel) => channel.status !== 'CLOSED')
      // TODO channels TO...

      if (channels.length == 0) {
        return log(`\nNo open channels found.`)
      }

      const result: string[] = []
      // find counterpartys' peerIds
      for (const channel of channels) {
        const id = channel.getId()
        result.push(await this.generateOutput(id.toHex(), channel))
      }

      return log(result.join('\n\n'))
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
