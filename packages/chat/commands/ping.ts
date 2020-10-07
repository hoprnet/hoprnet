import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import type { AutoCompleteResult } from './abstractCommand'
import chalk from 'chalk'
import { AbstractCommand, GlobalState } from './abstractCommand'
import { checkPeerIdInput, isBootstrapNode, getPeerIdsAndAliases } from '../utils'

export default class Ping extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'ping'
  }

  public help() {
    return 'pings another node to check its availability'
  }

  public async execute(query: string, state: GlobalState): Promise<string> {
    if (!query) {
      return `Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query, state)
    } catch (err) {
      return chalk.red(err.message)
    }

    let out = ''
    if (isBootstrapNode(this.node, peerId)) {
      out += chalk.gray(`Pinging the bootstrap node ...`) + '\n'
    }

    try {
      const latency = await this.node.ping(peerId)
      return `${out}Pong received in: ${chalk.magenta(String(latency))}ms`
    } catch (err) {
      return `${out}Could not ping node. Error was: ${chalk.red(err.message)}`
    }
  }

  public async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeerIdsAndAliases(this.node, state, {
      noBootstrapNodes: true,
      returnAlias: true,
      mustBeOnline: true,
    })

    return this._autocompleteByFiltering(query, allIds, line)
  }
}
