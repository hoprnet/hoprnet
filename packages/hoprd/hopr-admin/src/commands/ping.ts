import type API from '../utils/api'
import type { PeerId } from '@libp2p/interface-peer-id'
import { Command, type CacheFunctions } from '../utils/command'

export default class Ping extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[['hoprAddressOrAlias']], '']
      },
      api,
      cache
    )
  }

  public name() {
    return 'ping'
  }

  public description() {
    return 'Pings another node to check its availability'
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , peerId] = this.assertUsage(query) as [string | undefined, string, PeerId]
    if (error) {
      return log(error)
    }

    const peerIdStr = peerId.toString()
    const response = await this.api.ping(peerIdStr)

    if (!response.ok) {
      return log(
        await this.failedApiCall(response, 'ping node', {
          400: `Invalid peer ID ${peerIdStr}`,
          422: `Peer ${peerIdStr} not reachable`
        })
      )
    }

    const ping = await response.json()

    if (ping.latency >= 0) {
      return log(`Pong from peer ${peerIdStr} received in ${ping.latency} ms`)
    } else {
      return log(this.failedCommand('ping node', 'timeout'))
    }
  }
}
