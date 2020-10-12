import { AbstractCommand, AutoCompleteResult, GlobalState } from '../abstractCommand'
import { styleValue, getOptions } from '../../utils'

export class Routing extends AbstractCommand {
  private readonly options: GlobalState['routing'][] = ['manual', 'direct']

  public name() {
    return 'routing'
  }

  public help() {
    return 'The routing algorithm that is used to send messages'
  }

  public async execute(query: string, state: GlobalState): Promise<string | void> {
    if (!query) {
      return styleValue(state.routing, 'highlight')
    }

    const option = this.options.find((o) => query === o)

    if (!option) {
      return styleValue('Invalid option.', 'failure')
    }

    state.routing = option
    return `You have set your “${styleValue(this.name(), 'highlight')}” settings to “${styleValue(
      state.routing,
      'highlight'
    )}”.`
  }

  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    // nothing provided, just show all options
    if (!query) {
      return [getOptions(this.options.map((o) => ({ value: styleValue(o, 'highlight') }))), line]
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
