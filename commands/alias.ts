import { AbstractCommand, GlobalState } from './abstractCommand'
import { checkPeerIdInput } from '../utils'

const COMMAND_FORMAT = /([A-Za-z0-9]{53})\s([A-Za-z0-9]*)/

export class Alias extends AbstractCommand {
  name() { return 'alias' }
  help() { return 'alias an address with a more memorable name' }

  async execute(query: string, settings: GlobalState): Promise<string | void> {
    const match = COMMAND_FORMAT.exec(query)
    if (!match){
      return "usage: alias <PeerId> <Name>"
    }
    const id = match[1]
    const name = match[2]
    let peerId
    try {
      peerId = await checkPeerIdInput(id)
    } catch (e) {
      console.log(e)
      return
    }
    settings.aliases.set(name, peerId)
  }
}

