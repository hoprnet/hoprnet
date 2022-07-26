import type Hopr from '@hoprnet/hopr-core'
import type { StateOps } from '../types.js'
import { AbstractCommand } from './abstractCommand.js'
import { checkPeerIdInput, getPaddingLength, styleValue } from './utils/index.js'

export class Alias extends AbstractCommand {
  private parameters = ['PeerId', 'Name']

  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'alias'
  }

  public help() {
    return 'Alias an address with a more memorable name'
  }

  async execute(log, query: string, { getState, setState }: StateOps): Promise<void> {
    const state = getState()

    // view aliases
    if (!query) {
      const names = Array.from(state.aliases.keys()).map((name) => `${name} -> `)

      // no aliases found
      if (names.length === 0) {
        return log(`No aliases found.\nTo set an alias use, ${this.usage(this.parameters)}`)
      }

      const peerIds = Array.from(state.aliases.values())
      const paddingLength = getPaddingLength(names, false)

      return log(
        names
          .map((name, index) => {
            return name.padEnd(paddingLength) + styleValue(peerIds[index].toString(), 'peerId')
          })
          .join('\n')
      )
    }

    const [error, id, name] = this._assertUsage(query, this.parameters)
    if (error) return log(styleValue(error, 'failure'))

    try {
      let peerId = checkPeerIdInput(id)
      state.aliases.set(name, peerId)
      setState(state)

      return log(`Set alias '${styleValue(name, 'highlight')}' to '${styleValue(peerId.toString(), 'peerId')}'.`)
    } catch (err) {
      return log(styleValue(err instanceof Error ? err.message : 'Unknown error', 'failure'))
    }
  }
}
