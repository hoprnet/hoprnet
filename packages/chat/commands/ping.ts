import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import type {AutoCompleteResult} from './abstractCommand'
import {AbstractCommand, GlobalState} from './abstractCommand'
import {checkPeerIdInput, isBootstrapNode, getPeerIdsAndAliases, styleValue} from '../utils'

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

    try {
      const {info, latency} = await this.node.ping(peerId)
      return `${out}Pong received in: ${styleValue(latency)} ms ${info}`
    } catch (err) {
      if (err && err.message) {
        return `${out}Could not ping node. Error was: ${styleValue(err.message, 'failure')}`
      }
      return `${out}Could not ping node. Unknown error.`
    }
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
