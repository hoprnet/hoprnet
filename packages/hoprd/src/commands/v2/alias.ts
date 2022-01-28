import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand, GlobalState } from '../abstractCommand'
import { getPaddingLength, styleValue } from '../utils'
import { setAlias } from './logic/alias'

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
      // no aliases found
      if (state.aliases.size === 0) {
        return log(`No aliases found.\nTo set an alias use, ${this.usage(this.parameters)}`)
      }

      // NOTE: this one can be aslo extracted as getAllAliases function and put into v2 pureLogic folder
      const names = Array.from(state.aliases.keys()).map((name) => `${name} -> `)
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

    setAlias({ peerId: id, alias: name, state, log })
    return
  }
}
