import { getPaddingLength, styleValue, getOptions } from '../utils'
import { AbstractCommand, GlobalState, AutoCompleteResult } from '../abstractCommand'
import { IncludeRecipient } from './includeRecipient'
import { Routing } from './routing'

// to add a new setting, include it here and in class this.settings
type SettingsDirectory = {
  [key in keyof Pick<GlobalState, 'includeRecipient' | 'routing'>]: IncludeRecipient | Routing
}
type SettingsKeys = keyof SettingsDirectory
type SettingsValues = GlobalState[SettingsKeys]

export default class Settings extends AbstractCommand {
  private settings: SettingsDirectory
  private paddingLength: number

  constructor() {
    super()

    // to add a new setting, include it here
    this.settings = {
      includeRecipient: new IncludeRecipient(),
      routing: new Routing()
    }
    this.paddingLength = getPaddingLength(Object.keys(this.settings))
  }

  public name() {
    return 'settings'
  }

  public help() {
    return 'list your current settings'
  }

  private get settingsKeys(): SettingsKeys[] {
    return Object.keys(this.settings) as SettingsKeys[]
  }

  public async execute(query: string, state: GlobalState): Promise<string | void> {
    // nothing provided, just show current settings
    if (!query) {
      const entries = this.settingsKeys.map<[SettingsKeys, SettingsValues]>((setting) => {
        return [setting, state[setting] as SettingsValues]
      })

      const results: string[] = []
      for (const [key, value] of entries) {
        results.push(key.padEnd(this.paddingLength) + styleValue(value))
      }

      return results.join('\n')
    }

    const [setting, ...optionArray] = query.split(' ')
    const option = optionArray.join(' ')

    // found an exact match, run the setting's execute
    const matchesASetting = this.settingsKeys.find((s) => {
      return s === setting
    })
    if (typeof matchesASetting !== 'undefined') {
      return this.settings[matchesASetting].execute(option, state)
    }

    return styleValue(`Setting “${styleValue(setting)}” does not exist.`, 'failure')
  }

  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    // nothing provided, just show all settings
    if (!query) {
      return [
        getOptions(
          Object.values(this.settings).map((setting) => {
            return {
              value: setting.name(),
              description: setting.help()
            }
          }),
          'vertical'
        ),
        line
      ]
    }

    // found an exact match, run the setting's autocomplete
    const matchesASetting = this.settingsKeys.find((s) => {
      return s === query
    })
    if (typeof matchesASetting !== 'undefined') {
      const [, , ...subQueryArray] = line.split(' ')
      const subQuery = subQueryArray.join(' ')

      return this.settings[matchesASetting].autocomplete(subQuery, line)
    }

    // matches a setting partly, show matched settings
    const matchesPartlyASetting = this.settingsKeys.filter((s) => {
      return s.startsWith(query)
    })
    if (matchesPartlyASetting.length > 0) {
      return [matchesPartlyASetting.map((str: string) => `${this.name()} ${str}`), line]
    }

    // show nothing
    return [[this.name()], line]
  }
}
