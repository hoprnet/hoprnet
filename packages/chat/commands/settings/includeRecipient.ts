import chalk from 'chalk'
import { AbstractCommand, AutoCompleteResult, GlobalState } from '../abstractCommand'
import { styleValue } from '../../utils'

export class IncludeRecipient extends AbstractCommand {
  private readonly options: GlobalState['includeRecipient'][] = [true, false]

  public name() {
    return 'includeRecipient'
  }

  public help() {
    return 'preprends your address to all messages'
  }

  public async execute(query: string, state: GlobalState): Promise<string | void> {
    if (!query) {
      return styleValue(state.includeRecipient)
    }

    if (!query.match(/true|false/i)) {
      return chalk.red('Invalid option.')
    }

    state.includeRecipient = !!query.match(/true/i)
    return `You have set your “${styleValue(this.name())}” settings to “${styleValue(state.includeRecipient)}”.`
  }

  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    // nothing provided, just show all options
    if (!query) {
      return [this.options.map(styleValue), line]
    }

    // matches a option partly, show matches options
    const matchesPartly = this.options.filter((option) => {
      return String(option).toLowerCase().startsWith(query.toLowerCase())
    })
    if (matchesPartly.length > 0) {
      return [matchesPartly.map((str) => `settings ${this.name()} ${String(str)}`), line]
    }

    return [[this.name()], line]
  }
}
