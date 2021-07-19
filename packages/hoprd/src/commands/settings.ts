import { getPaddingLength, styleValue } from './utils'
import { AbstractCommand, GlobalState } from './abstractCommand'
import type Hopr from '@hoprnet/hopr-core'
import { PassiveStrategy, PromiscuousStrategy } from '@hoprnet/hopr-core'

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
  private settings

  constructor(private node: Hopr) {
    super()
    this.settings = {
      includeRecipient: ['Prepends your address to all messages (true|false)', booleanSetter('includeRecipient')],
      strategy: [
        'Set an automatic strategy for the node. (passive|promiscuous)',
        this.setStrategy.bind(this),
        this.getStrategy.bind(this)
      ]
    }
  }

  private setStrategy(query: string): string {
    if (query == 'passive') {
      this.node.setChannelStrategy(new PassiveStrategy())
      return 'Strategy is now passive'
    }
    if (query == 'promiscuous') {
      this.node.setChannelStrategy(new PromiscuousStrategy())
      return 'Strategy is now promiscuous'
    }
    return 'Could not set strategy. Try PASSIVE or PROMISCUOUS'
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
    const keyPadding = getPaddingLength(Object.keys(this.settings))
    const valuePadding = getPaddingLength(entries.map((x) => x[1] + ''))
    for (const [key, value] of entries) {
      results.push(key.padEnd(keyPadding) + styleValue(value + '').padEnd(valuePadding) + this.settings[key][0])
    }
    return results.join('\n')
  }

  private getState(setting: string, state: GlobalState): string {
    if (this.settings[setting] && this.settings[setting][2]) {
      // Use getter
      return this.settings[setting][2]()
    }
    return state[setting]
  }

  public async execute(log, query: string, state: GlobalState): Promise<void> {
    if (!query) {
      log(this.listSettings(state))
      return
    }

    const [setting, ...optionArray] = query.split(' ')
    const option = optionArray.join(' ')

    if (!option) {
      log(setting + ': ' + this.getState(setting, state))
      return
    }

    // found an exact match, run the setting's execute
    const matchesASetting = this.settingsKeys.find((s) => {
      return s === setting
    })
    if (typeof matchesASetting !== 'undefined') {
      return log(this.settings[matchesASetting][1](option, state))
    }

    return log(styleValue(`Setting “${styleValue(setting)}” does not exist.`, 'failure'))
  }
}
