import { AbstractCommand, AutoCompleteResult, GlobalState } from '../abstractCommand'
import { styleValue, getOptions, checkPeerIdInput } from '../../utils'

export class Routing extends AbstractCommand {
  // there is actually another option which takes in peerIds
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

    // provided peerIds
    if (query.match(',')) {
      try {
        await Promise.all(query.split(',').map(async (peerId) => await checkPeerIdInput(peerId)))
        state.routing = query
      } catch (err) {
        return styleValue(`Invalid option ${query}.`, 'failure')
      }
    }
    // provided a mode
    else {
      const option = this.options.find((o) => query === o)
      if (!option) {
        return styleValue(`Invalid option ${query}.`, 'failure')
      }

      state.routing = option
    }

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
