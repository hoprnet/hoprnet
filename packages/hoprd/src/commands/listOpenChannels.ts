import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { PublicKey, ChannelStatus } from '@hoprnet/hopr-utils'

const channelStatusToString = (status: ChannelStatus): string => {
  if (status === 0) return 'Closed'
  else if (status === 1) return 'WaitingForCommitment'
  else if (status === 2) return 'Open'
  else if (status === 3) return 'PendingToClose'
  return 'Unknown'
}

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
      const selfPubKey = new PublicKey(this.node.getId().pubKey.marshal())
      const selfAddress = selfPubKey.toAddress()

      const channelsFrom = (await this.node.getChannelsFrom(selfAddress)).filter(
        (channel) => channel.status !== ChannelStatus.Closed
      )
      if (channelsFrom.length == 0) {
        log(`\nNo open channels from node.`)
      }
      // find counterpartys' peerIds
      for (const channel of channelsFrom) {
        log(`
Outgoing Channel:       ${styleValue(channel.getId().toHex(), 'hash')}
To:                     ${styleValue(channel.destination.toPeerId().toB58String(), 'peerId')}
Status:                 ${styleValue(channelStatusToString(channel.status), 'highlight')}
Balance:                ${styleValue(channel.balance.toFormattedString(), 'number')}
`)
      }

      const channelsTo = (await this.node.getChannelsTo(selfAddress)).filter(
        (channel) => channel.status !== ChannelStatus.Closed
      )
      if (channelsTo.length == 0) {
        log(`\nNo open channels to node.`)
      }
      // find counterpartys' peerIds
      for (const channel of channelsTo) {
        log(`
Incoming Channel:       ${styleValue(channel.getId().toHex(), 'hash')}
From:                   ${styleValue(channel.source.toPeerId().toB58String(), 'peerId')}
Status:                 ${styleValue(channelStatusToString(channel.status), 'highlight')}
Balance:                ${styleValue(channel.balance.toFormattedString(), 'number')}
`)
      }
      return
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
