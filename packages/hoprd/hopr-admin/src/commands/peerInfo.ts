import type PeerId from 'peer-id'
import type API from '../utils/api'
import { Command } from '../utils/command'

export default class PeerInfo extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[['hoprAddressOrAlias', 'peer', false]], 'gets information about peer']
      },
      api,
      extra,
      true
    )
  }

  public name() {
    return 'addresses'
  }

  public description() {
    return 'Get information of a peer'
  }

  public async execute(log, query: string): Promise<void> {
    try {
      const [error, , peerId] = this.assertUsage(query) as [string | undefined, string, PeerId]
      if (error) return log(error)

      const peerIdStr = peerId.toB58String()
      const { announced, observed } = await this.api.getPeerInfo(peerIdStr)

      return log(
        [
          `Announced addresses for ${peerIdStr}:`,
          ...announced,
          `Observed addresses for ${peerIdStr}:`,
          ...observed
        ].join('\n')
      )
    } catch {
      log(this.invalidUsage(query))
    }
  }
}
