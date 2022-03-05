import { AbstractCommand } from './abstractCommand'
import type PeerId from 'peer-id'
import { checkPeerIdInput, styleValue } from './utils'

export default class Addresses extends AbstractCommand {
  constructor() {
    super()
    this.hidden = true
  }

  public name() {
    return 'addresses'
  }

  public help() {
    return 'Get the known addresses of other nodes'
  }

  public async execute(log, query: string): Promise<void> {
    if (!query) {
      return log(`Invalid arguments. Expected 'addresses <peerId>'. Received '${query}'`)
    }

    let peerId: PeerId
    try {
      peerId = checkPeerIdInput(query, getState())
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    return log(
      `Announced addresses for ${query}:\n- ${(await this.node.getAnnouncedAddresses(peerId))
        .map((ma) => ma.toString())
        .join('\n- ')}` +
        `\nObserved addresses for ${query}:\n- ${this.node
          .getObservedAddresses(peerId)
          .map((addr) => `${addr.toString()}`)
          .join(`\n- `)}`
    )
  }
}
