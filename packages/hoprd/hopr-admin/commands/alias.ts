import { AbstractCommand } from './abstractCommand'
import { checkPeerIdInput, getPaddingLength, styleValue } from './utils'
import { getAliases, setAliases } from '../fetch'

export class Alias extends AbstractCommand {
  private parameters = ['PeerId', 'Name']

  constructor() {
    super()
  }

  public name() {
    return 'alias'
  }

  public help() {
    return 'Alias an address with a more memorable name'
  }

  async execute(log, query: string): Promise<void> {
    // view aliases
    if (!query) {
      const aliases = await getAliases()
      const names = Object.entries(aliases).map(([name]) => `${name} -> `)

      // no aliases found
      if (names.length === 0) {
        return log(`No aliases found.\nTo set an alias use, ${this.usage(this.parameters)}`)
      }

      const peerIds = Object.entries(aliases).map(([name, alias]) => `${alias}`)
      const paddingLength = getPaddingLength(names, false)

      return log(
        names
          .map((name, index) => {
            return name.padEnd(paddingLength) + styleValue(peerIds[index], 'peerId')
          })
          .join('\n')
      )
    }

    // sets aliases
    const [error, id, name] = this._assertUsage(query, this.parameters)
    if (error) return log(styleValue(error, 'failure'))

    try {
      let peerId = checkPeerIdInput(id)
      // TODO: Handle responses
      setAliases(name, peerId)

      return log(`Set alias '${styleValue(name, 'highlight')}' to '${styleValue(peerId, 'peerId')}'.`)
    } catch (error) {
      return log(styleValue(error.message, 'failure'))
    }
  }
}
