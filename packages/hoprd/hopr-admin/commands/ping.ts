import type PeerId from 'peer-id'
import { AbstractCommand } from './abstractCommand'
import { checkPeerIdInput, styleValue } from './utils'
import HoprFetcher from '../fetch'

export default class Ping extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
  }

  public name() {
    return 'ping'
  }

  public help() {
    return 'Pings another node to check its availability'
  }

  public async execute(log, query: string): Promise<void> {
    if (!query) {
      return log(`Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`)
    }

    let peerId: PeerId
    try {
      peerId = checkPeerIdInput(query)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    let out = ''

    let pingResult: any

    let error: any

    try {
      pingResult = await this.hoprFetcher.pingNodePeer(query).then((res) => res.json())
    } catch (err) {
      error = err
    }

    if (pingResult.latency >= 0) {
      return log(`${out}Pong received in: ${styleValue(pingResult.latency)} ms ${pingResult?.info ?? ''}`)
    }

    if (error && error.message) {
      return log(`${out}Could not ping node. Error was: ${styleValue(error.message, 'failure')}`)
    }
    return log(`${out}Could not ping node. Timeout.`)
  }
}
