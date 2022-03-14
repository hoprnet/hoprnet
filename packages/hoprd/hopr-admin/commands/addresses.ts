import { AbstractCommand } from './abstractCommand'
import type PeerId from 'peer-id'
import {  styleValue } from './utils'
import HoprFetcher from '../fetch'

// TODO: Missing getObservedAddresses API for now
export default class Addresses extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
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
      peerId = await this.checkPeerIdInput(query)
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
