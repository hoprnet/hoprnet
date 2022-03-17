import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command } from '../utils/command'

export default class Settings extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        view: [[], 'show all settings'],
        update: [
          [
            ['string', "setting's key", false],
            ['string', "setting's value", false]
          ],
          'update a setting'
        ]
      },
      api,
      extra
    )
  }

  public name() {
    return 'settings'
  }

  public description() {
    return 'list your current settings or update them'
  }

  public async execute(log, query: string): Promise<void> {
    const [error, use, key, value] = this.assertUsage(query) as [string | undefined, string, string, string]
    if (error) {
      return log(error)
    }

    try {
      if (use === 'view') {
        const settings = await this.api.getSettings()
        return log(toPaddedString(Object.entries(settings).map<[string, string]>(([k, v]) => [k, String(v)])))
      } else {
        const response = await this.api.setSetting(key, value)
        if (response.status === 204) {
          return log('Settings updated.')
        } else {
          return log(`Unable to update settings. ${response.status}`)
        }
      }
    } catch (err) {
      return log(`Unexpected error: ${err.message}`)
    }
  }
}
