import type API from '../utils/api'
import type { PeerId } from '@libp2p/interface-peer-id'
import { toPaddedString } from '../utils'
import { Command, CacheFunctions } from '../utils/command'

export default class Alias extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[], 'show aliases'],
        setAlias: [[['hoprAddress'], ['string', 'name']], 'set alias'],
        removeAlias: [
          [
            ['constant', 'remove'],
            ['string', 'name']
          ],
          'remove alias'
        ]
      },
      api,
      cache
    )
  }

  public name() {
    return 'alias'
  }

  public description() {
    return 'View, set, or remove aliases'
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, use, peerId, name] = this.assertUsage(query) as [string | undefined, string, PeerId, string]
    if (error) return log(error)

    // get latest known aliases
    const aliases = this.cache.getCachedAliases()

    if (use === 'default') {
      const entries = Object.entries(aliases)

      // no aliases found
      if (entries.length === 0) {
        return log(`No aliases found.\n${this.usage()}`)
      }

      return log(toPaddedString(entries.map<[string, string]>(([name, peerId]) => [name, `-> ${peerId}`])))
    } else if (use === 'setAlias') {
      const response = await this.api.setAlias(peerId.toString(), name)

      if (response.status == 201) {
        this.cache.updateAliasCache((prevAliases) => ({
          ...prevAliases,
          [name]: peerId.toString()
        }))
        return log(`Alias '${name}' was set to '${peerId.toString()}'.`)
      } else {
        return log(
          await this.failedApiCall(response, `set alias '${name}'`, {
            400: `invalid peer ID ${peerId.toString()}`,
            422: (v) => v.error
          })
        )
      }
    } else {
      const response = await this.api.removeAlias(name)

      if (response.status == 204) {
        this.cache.updateAliasCache((prevAliases) => {
          delete prevAliases[name]
          return prevAliases
        })
        return log(`Alias '${name}' was removed.`)
      } else {
        return log(
          await this.failedApiCall(response, `remove alias '${name}'`, {
            422: ''
          })
        )
      }
    }
  }
}
