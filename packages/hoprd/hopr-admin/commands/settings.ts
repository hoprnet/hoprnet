import { getPaddingLength, styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'
import HoprFetcher from '../fetch'

export default class Settings extends AbstractCommand {
  private settings

  constructor(fetcher: HoprFetcher) {
    super(fetcher)
    this.settings = {
      includeRecipient: ['Prepends your address to all messages (true|false)', this.booleanSetter('includeRecipient')],
      strategy: [
        'Set an automatic strategy for the node. (passive|promiscuous)',
        this.setStrategy.bind(this),
        this.getStrategy.bind(this)
      ]
    }
  }

  private booleanSetter(name: string) {
    return async (query: string): Promise<string> => {
      if (!query.match(/true|false/i)) {
        return styleValue(`Invalid option.`, 'failure')
      }

      let nodeSettings = await this.hoprFetcher.getSettings()
      nodeSettings[name] = !!query.match(/true/i)
      return `You have set your “${styleValue(name, 'highlight')}” settings to “${styleValue(nodeSettings[name])}”.`
    }
  }

  private async getStrategy(): Promise<string> {
    return await this.hoprFetcher.getSettings().then((res) => res.strategy)
  }

  private async setStrategy(query: string): Promise<string> {
    if (query == 'passive') {
      const response = await this.hoprFetcher.setSettings('strategy', 'passive')
      if (response.status === 204) {
        return 'Strategy is now passive'
      }
    }

    if (query == 'promiscuous') {
      const response = await this.hoprFetcher.setSettings('strategy', 'promiscuous')
      if (response.status === 204) {
        return 'Strategy is now promiscuous'
      }
    }
    return 'Could not set strategy. Try PASSIVE or PROMISCUOUS'
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

  private async listSettings(): Promise<string> {
    // return key-value, for each key in settingsKeys array
    const entries = await Promise.all(
      this.settingsKeys.map(async (setting) => {
        return [setting, await this.getSingleState(setting)]
      })
    )

    const results: string[] = []
    const keyPadding = getPaddingLength(Object.keys(this.settings))
    const valuePadding = getPaddingLength(entries.map((x) => x[1] + ''))
    for (const [key, value] of entries) {
      results.push(key.padEnd(keyPadding) + styleValue(value + '').padEnd(valuePadding) + this.settings[key][0])
    }

    return results.join('\n')
  }

  // returns function to compute to get value of key in the array
  private async getSingleState(setting: string): Promise<any> {
    if (this.settings[setting] && this.settings[setting][2]) {
      // Use getter
      return this.settings[setting][2]()
    }

    let nodeSettings = await this.hoprFetcher.getSettings()
    return nodeSettings[setting]
  }

  public async execute(log, query: string): Promise<void> {
    // cmd settings
    if (!query) {
      log(await this.listSettings())
      return
    }

    const [setting, ...optionArray] = query.split(' ')
    const option = optionArray.join(' ')

    if (!option) {
      log(setting + ': ' + (await this.getSingleState(setting)))
      return
    }

    // found an exact match, run the setting's execute
    const matchesASetting = this.settingsKeys.find((s) => {
      return s === setting
    })
    if (typeof matchesASetting !== 'undefined') {
      this.settings[setting] = setting === 'includeRecipient' ? !!option : option
      return log(this.settings[matchesASetting][1](option))
    }

    return log(styleValue(`Setting “${styleValue(setting)}” does not exist.`, 'failure'))
  }
}
