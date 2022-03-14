import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import HoprFetcher from '../fetch'

export default class ListOpenChannels extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
  }

  public name() {
    return 'channels'
  }

  public help() {
    return 'Lists your currently open channels'
  }

  private static consoleOutput(channel) {
    return `
Outgoing Channel:       ${styleValue(channel.channelId, 'hash')}
To:                     ${styleValue(channel.peerId, 'peerId')}
Status:                 ${styleValue(channel.status, 'highlight')}
Balance:                ${styleValue(channel.balance, 'number')}
\``
  }

  /**
   * Lists all channels that we have with other nodes. Triggered from the CLI.
   */
  async execute(log: (str: string) => void): Promise<void> {
    log('fetching channels...')
    try {
      const channels = await this.hoprFetcher.getChannels()
      const channelsFrom = channels.incoming.filter((channel) => channel.status !== "Closed")

      if (channelsFrom.length == 0) {
        log(`\nNo open channels from node.`)
      }
      // find counterpartys' peerIds
      for (const channel of channelsFrom) {
        log(ListOpenChannels.consoleOutput(channel))
      }

      const channelsTo = channels.outgoing.filter((channel) => channel.status !== "Closed")
      if (channelsTo.length == 0) {
        log(`\nNo open channels to node.`)
      }
      // find counterpartys' peerIds
      for (const channel of channelsTo) {
        log(ListOpenChannels.consoleOutput(channel))
      }
      return
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
