import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { AbstractCommand } from './abstractCommand.mjs'
import type { StateOps } from '../types.mjs'
import { checkPeerIdInput, styleValue } from './utils/index.mjs'

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

  public async execute(log, query: string, { getState }: StateOps): Promise<void> {
    if (!query) {
      return log(`Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`)
    }

    let peerId: PeerId
    try {
      peerId = checkPeerIdInput(query, getState())
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    let out = ''

    let pingResult: Awaited<ReturnType<Hopr['ping']>>

    let error: any

    try {
      pingResult = await this.node.ping(peerId)
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
