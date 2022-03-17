import type API from '../utils/api'
import { Command } from '../utils/command'
import PeerId from 'peer-id'

export default class Ping extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[['hoprAddressOrAlias', "node's hopr address or alias", false]], '']
      },
      api,
      extra
    )
  }

  public name() {
    return 'ping'
  }

  public description() {
    return 'Pings another node to check its availability'
  }

  public async execute(log, query: string): Promise<void> {
    const [error, , peerId] = this.assertUsage(query) as [string | undefined, string, PeerId]
    if (error) {
      return log(error)
    }

    let out = ''
    let pingResult: any
    let pingError: any

    try {
      pingResult = await this.api.ping(peerId.toB58String()).then((res) => res.json())
    } catch (error) {
      pingError = error.message
    }

    if (pingResult.latency >= 0) {
      return log(`${out}Pong received in: ${pingResult.latency} ms ${pingResult?.info ?? ''}`)
    } else if (pingError) {
      return log(`${out}Could not ping node. Error was: ${pingError}`)
    } else {
      return log(`${out}Could not ping node. Timeout.`)
    }
  }
}
