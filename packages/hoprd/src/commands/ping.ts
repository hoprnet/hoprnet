import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { AbstractCommand, GlobalState } from './abstractCommand.js'
import { checkPeerIdInput, styleValue } from './utils/index.js'

export default class Ping extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'ping'
  }

  public help() {
    return 'Pings another node to check its availability'
  }

  public async execute(log, query: string, state: GlobalState): Promise<void> {
    if (!query) {
      return log(`Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`)
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query, state)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    let out = ''

    let pingResult: {
      info: string
      latency: number
    }

    let error: any

    try {
      pingResult = await this.node.ping(peerId)
    } catch (err) {
      error = err
    }

    if (pingResult.latency >= 0) {
      return log(`${out}Pong received in: ${styleValue(pingResult.latency)} ms ${pingResult.info}`)
    }

    if (error && error.message) {
      return log(`${out}Could not ping node. Error was: ${styleValue(error.message, 'failure')}`)
    }
    return log(`${out}Could not ping node. Timeout.`)
  }
}
