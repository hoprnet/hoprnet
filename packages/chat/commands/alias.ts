import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import chalk from 'chalk'
import { AbstractCommand, GlobalState, AutoCompleteResult } from './abstractCommand'
import { checkPeerIdInput, getPaddingLength, getPeerIdsAndAliases, styleValue } from '../utils'

export class Alias extends AbstractCommand {
  private parameters = ['PeerId', 'Name']

  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'alias'
  }

  public help() {
    return 'alias an address with a more memorable name'
  }

  async execute(query: string, state: GlobalState): Promise<string> {
    // view aliases
    if (!query) {
      const names = Array.from(state.aliases.keys())

      // no aliases found
      if (names.length === 0) {
        return `No aliases found.\nTo set an alias, ${this.usage(this.parameters)}`
      }

      const peerIds = Array.from(state.aliases.values())
      const paddingLength = getPaddingLength(names)

      return names
        .map((name, index) => {
          return name.padEnd(paddingLength) + chalk.green(peerIds[index].toB58String())
        })
        .join('\n')
    }

    const [error, id, name] = this._assertUsage(query, ['PeerId', 'Name'])
    if (error) return chalk.red(error)

    try {
      let peerId = await checkPeerIdInput(id)
      state.aliases.set(name, peerId)

      return `Set alias '${styleValue(name)}' to '${styleValue(peerId)}'.`
    } catch (error) {
      return chalk.red(error.message)
    }
  }

  async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeerIdsAndAliases(this.node, state)
    return this._autocompleteByFiltering(query, allIds, line)
  }
}
