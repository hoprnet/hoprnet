import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions } from '../utils/command'

export default class Settings extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        view: [[], 'show all settings'],
        update: [
          [
            ['string', 'key'],
            ['string', 'value']
          ],
          'update a setting'
        ]
      },
      api,
      cache
    )
  }

  public name() {
    return 'settings'
  }

  public description() {
    return 'list your current settings or update them'
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, use, key, value] = this.assertUsage(query) as [string | undefined, string, string, string]
    if (error) {
      return log(error)
    }

    if (use === 'view') {
      const response = await this.api.getSettings()
      if (!response.ok) return log(this.failedCommand('get settings'))
      return log(
        toPaddedString(Object.entries(await response.json()).map<[string, string]>(([k, v]) => [k, String(v)]))
      )
    } else {
      const response = await this.api.setSetting(key, key === 'includeRecipient' ? value === 'true' : value)
      if (!response.ok) return log(this.failedCommand(`set setting "${key}" to "${value}"`))
      return log('Settings updated.')
    }
  }
}
