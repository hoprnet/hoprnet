import { AbstractCommand, GlobalState, AutoCompleteResult} from './abstractCommand'
import { checkPeerIdInput, getPeersIdsAsString } from '../utils'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

const COMMAND_FORMAT = /([A-Za-z0-9]{53})\s([A-Za-z0-9]*)/

export class Alias extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() { return 'alias' }
  help() { return 'alias an address with a more memorable name' }

  async execute(query: string, settings: GlobalState): Promise<string | void> {
    const match = COMMAND_FORMAT.exec(query)
    if (!match){
      return "usage: alias <PeerId> <Name>"
    }
    const id = match[1]
    const name = match[2]

    try {
      let peerId = await checkPeerIdInput(id)
      settings.aliases.set(name, peerId)
    } catch (e) {
      return e
    }
  }

  async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeersIdsAsString(this.node, {
      noBootstrapNodes: true,
    }).concat(Array.from(state.aliases.keys()))
    return this._autocompleteByFiltering(query, allIds, line)
  }
}

