import type { PeerId } from '@libp2p/interface-peer-id'
import type API from '../utils/api'
import { Command, type CacheFunctions } from '../utils/command'

export default class PeerInfo extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[['hoprAddressOrAlias']], 'gets information about peer']
      },
      api,
      cache,
      true
    )
  }

  public name() {
    return 'peerinfo'
  }

  public description() {
    return '*For devs* Get information of a peer'
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , peerId] = this.assertUsage(query) as [string | undefined, string, PeerId]
    if (error) return log(error)

    const peerIdStr = peerId.toString()
    const peerInfoRes = await this.api.getPeerInfo(peerIdStr)
    if (!peerInfoRes.ok) return log(this.failedCommand("get peer's information"))
    const { announced, observed } = await peerInfoRes.json()

    return log(
      [`Announced addresses for ${peerIdStr}:`, ...announced, `Observed addresses for ${peerIdStr}:`, ...observed].join(
        '\n'
      )
    )
  }
}
