import { getPaddingLength, styleValue } from '../utils'
import { AbstractCommand, GlobalState } from '../abstractCommand'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

function booleanSetter(name: string) {
  return function setter(query: string, state: GlobalState): string {
    if (!query.match(/true|false/i)) {
      return styleValue(`Invalid option.`, 'failure')
    }
    state[name] = !!query.match(/true/i)
    return `You have set your “${styleValue(name, 'highlight')}” settings to “${styleValue(state[name])}”.`
  }
}

export default class Settings extends AbstractCommand {
  private paddingLength: number
  private settings

  constructor(private node: Hopr<HoprCoreConnector>) {
    super()
    this.settings = {
      includeRecipient: ['Prepends your address to all messages (true|false)', booleanSetter('includeRecipient')],
      strategy: ['set an automatic strategy for the node. (PASSIVE|PROMISCUOUS)', this.setStrategy.bind(this), this.getStrategy.bind(this)]
    }
    this.paddingLength = getPaddingLength(Object.keys(this.settings))
  }

  private async setStrategy(query: string): Promise<string> {
    try {
      this.node.setChannelStrategy(query as any)
      return 'Strategy was set'
    } catch {
      return 'Could not set strategy. Try PASSIVE or PROMISCUOUS'
    }
  }

  private getStrategy(): string {
    return this.node.getChannelStrategy()
  }

  public name() {
    return 'settings'
  }

  public help() {
    return 'list your current settings'
  }

  private get settingsKeys(): string[] {
    return Object.keys(this.settings)
  }

  private listSettings(state: GlobalState): string {
    const entries = this.settingsKeys.map((setting) => {
      return [setting, this.getState(setting, state)]
    })

    const results: string[] = []
    for (const [key, value] of entries) {
      results.push(key.padEnd(this.paddingLength) + styleValue(value))
    }

    return results.join('\n')
  }

  private getState(setting: string, state: GlobalState) {
    if (this.settings[setting] && this.settings[setting][2]){
      // Use getter
      return this.settings[setting][2]()
    }
    return state[setting]
  }

  public async execute(query: string, state: GlobalState): Promise<string | void> {
    if (!query) {
      return this.listSettings(state)
    }

    const [setting, ...optionArray] = query.split(' ')
    const option = optionArray.join(' ')

    if (!option) {
      return this.getState(setting, state)
    }

    // found an exact match, run the setting's execute
    const matchesASetting = this.settingsKeys.find((s) => {
      return s === setting
    })
    if (typeof matchesASetting !== 'undefined') {
      return this.settings[matchesASetting][1](option, state)
    }

    return styleValue(`Setting “${styleValue(setting)}” does not exist.`, 'failure')
  }

  /*
  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    // nothing provided, just show all settings
    if (!query) {
      return [
        getOptions(
          Object.values(this.settings).map((setting) => {
            return {
              value: setting.name ? setting.name(),
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
  */
}
