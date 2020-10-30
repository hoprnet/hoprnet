import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
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
    return 'Alias an address with a more memorable name'
  }

  async execute(query: string, state: GlobalState): Promise<string> {
    // view aliases
    if (!query) {
      const names = Array.from(state.aliases.keys()).map((name) => `${name} -> `)

      // no aliases found
      if (names.length === 0) {
        return `No aliases found.\nTo set an alias use, ${this.usage(this.parameters)}`
      }

      const peerIds = Array.from(state.aliases.values())
      const paddingLength = getPaddingLength(names, false)

      return names
        .map((name, index) => {
          return name.padEnd(paddingLength) + styleValue(peerIds[index].toB58String(), 'peerId')
        })
        .join('\n')
    }

    const [error, id, name] = this._assertUsage(query, this.parameters)
    if (error) return styleValue(error, 'failure')

    try {
      let peerId = await checkPeerIdInput(id)
      state.aliases.set(name, peerId)

      return `Set alias '${styleValue(name, 'highlight')}' to '${styleValue(peerId.toB58String(), 'peerId')}'.`
    } catch (error) {
      return styleValue(error.message, 'failure')
    }
  }

  async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeerIdsAndAliases(this.node, state)
    return this._autocompleteByFiltering(query, allIds, line)
  }
}
