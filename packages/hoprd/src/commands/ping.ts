import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import type { AutoCompleteResult } from './abstractCommand'
import { AbstractCommand, GlobalState } from './abstractCommand'
import { checkPeerIdInput, isBootstrapNode, getPeerIdsAndAliases, styleValue } from './utils'

export default class Ping extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'ping'
  }

  public help() {
    return 'Pings another node to check its availability'
  }

  public async execute(query: string, state: GlobalState): Promise<string> {
    if (!query) {
      return `Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query, state)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    let out = ''
    if (isBootstrapNode(this.node, peerId)) {
      out += styleValue(`Pinging the bootstrap node ...`, 'highlight') + '\n'
    }

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
      return `${out}Pong received in: ${styleValue(pingResult.latency)} ms ${pingResult.info}`
    }

    if (error && error.message) {
      return `${out}Could not ping node. Error was: ${styleValue(error.message, 'failure')}`
    }
    return `${out}Could not ping node. Timeout.`

  }

  public async autocomplete(query: string = '', line: string = '', state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeerIdsAndAliases(this.node, state, {
      noBootstrapNodes: true,
      returnAlias: true,
      mustBeOnline: true
    })

    return this._autocompleteByFiltering(query, allIds, line)
  }
}
