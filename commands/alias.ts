import { AbstractCommand, GlobalState, AutoCompleteResult} from './abstractCommand'
import { checkPeerIdInput, getPeersIdsAsString } from '../utils'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

export class Alias extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() { return 'alias' }
  help() { return 'alias an address with a more memorable name' }

  async execute(query: string, settings: GlobalState): Promise<string | void> {
    const [err, id, name] = this._assertUsage(query, ['PeerId', 'Name'])
    if (err) return err

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

