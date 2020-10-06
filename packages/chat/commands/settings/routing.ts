import chalk from 'chalk'
import { AbstractCommand, AutoCompleteResult, GlobalState } from '../abstractCommand'
import { styleValue } from '../../utils'

export class Routing extends AbstractCommand {
  private readonly options: GlobalState['routing'][] = ['auto', 'manual', 'direct']

  public name() {
    return 'routing'
  }

  public help() {
    return 'pick a routing algorithm'
  }

  public async execute(query: string, state: GlobalState): Promise<string | void> {
    if (!query) {
      return styleValue(state.routing)
    }

    const option = this.options.find((o) => query === o)

    if (!option) {
      return chalk.red('Invalid option.')
    }

    state.routing = option
    return `You have set your “${styleValue(this.name())}” settings to “${styleValue(state.routing)}”.`
  }

  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    // nothing provided, just show all options
    if (!query) {
      return [this.options.map(styleValue), line]
    }

    // matches a option partly, show matches options
    const matchesPartly = this.options.filter((option) => {
      return option.startsWith(query)
    })
    if (matchesPartly.length > 0) {
      return [matchesPartly.map((str: string) => `settings ${this.name()} ${str}`), line]
    }

    return [[this.name()], line]
  }
}
