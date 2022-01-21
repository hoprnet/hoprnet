import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand, GlobalState } from './abstractCommand'
import { checkPeerIdInput, getPaddingLength, styleValue } from './utils'

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

  async execute(log, query: string, state: GlobalState): Promise<void> {
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
            return name.padEnd(paddingLength) + styleValue(peerIds[index].toB58String(), 'peerId')
          })
          .join('\n')
      )
    }

    const [error, id, name] = this._assertUsage(query, this.parameters)
    if (error) return log(styleValue(error, 'failure'))

    try {
      let peerId = checkPeerIdInput(id)
      state.aliases.set(name, peerId)

      return log(`Set alias '${styleValue(name, 'highlight')}' to '${styleValue(peerId.toB58String(), 'peerId')}'.`)
    } catch (error) {
      return log(styleValue(error.message, 'failure'))
    }
  }
}
