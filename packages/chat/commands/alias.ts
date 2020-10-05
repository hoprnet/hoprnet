import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import chalk from 'chalk'
import { AbstractCommand, GlobalState, AutoCompleteResult } from './abstractCommand'
import { checkPeerIdInput, getPeersIdsAsString, getPaddingLength } from '../utils'

export class Alias extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'alias'
  }

  public help() {
    return 'alias an address with a more memorable name'
  }

  async execute(query: string, state: GlobalState): Promise<string | void> {
    if (!query) {
      const names = Array.from(state.aliases.keys())
      const peerIds = Array.from(state.aliases.values())
      const paddingLength = getPaddingLength(names)

      return names
        .map((name, index) => {
          return name.padEnd(paddingLength) + chalk.green(peerIds[index].toB58String())
        })
        .join('\n')
    }

    const [err, id, name] = this._assertUsage(query, ['PeerId', 'Name'])
    if (err) return err

    try {
      let peerId = await checkPeerIdInput(id)
      state.aliases.set(name, peerId)
    } catch (e) {
      return e
    }
  }

  async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeersIdsAsString(this.node, {
      noBootstrapNodes: true,
    }).concat(Array.from(state.aliases.keys()))
    return this._autocompleteByFiltering(query, allIds, line)
  }
}
