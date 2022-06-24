import type { default as Hopr } from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand.js'
import { styleValue } from './utils/index.js'
import { PublicKey, ChannelStatus, channelStatusToString } from '@hoprnet/hopr-utils'

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

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(log: (str: string) => void): Promise<void> {
    log('fetching channels...')
    try {
      const selfPubKey = PublicKey.fromPeerId(this.node.getId())
      const selfAddress = selfPubKey.toAddress()

      const channelsFrom = (await this.node.getChannelsFrom(selfAddress)).filter(
        (channel) => channel.status !== ChannelStatus.Closed
      )
      if (channelsFrom.length == 0) {
        log(`\nNo open channels from node.`)
      }
      // find counterpartys' peerIds
      for (const channel of channelsFrom) {
        const out =
          `Outgoing Channel:       ${styleValue(channel.getId().toHex(), 'hash')}\n` +
          `To:                     ${styleValue(channel.destination.toPeerId().toB58String(), 'peerId')}\n` +
          `Status:                 ${styleValue(channelStatusToString(channel.status), 'highlight')}\n` +
          `Balance:                ${styleValue(channel.balance.toFormattedString(), 'number')}`
        log(out)
      }

      const channelsTo = (await this.node.getChannelsTo(selfAddress)).filter(
        (channel) => channel.status !== ChannelStatus.Closed
      )
      if (channelsTo.length == 0) {
        log(`\nNo open channels to node.`)
      }
      // find counterpartys' peerIds
      for (const channel of channelsTo) {
        const out =
          `Incoming Channel:       ${styleValue(channel.getId().toHex(), 'hash')}\n` +
          `From:                   ${styleValue(channel.source.toPeerId().toB58String(), 'peerId')}\n` +
          `Status:                 ${styleValue(channelStatusToString(channel.status), 'highlight')}\n` +
          `Balance:                ${styleValue(channel.balance.toFormattedString(), 'number')}\n`
        log(out)
      }
      return
    } catch (err) {
      return log(styleValue(err instanceof Error ? err.message : 'Unknown error', 'failure'))
    }
  }
}
