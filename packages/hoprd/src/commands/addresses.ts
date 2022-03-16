import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand, type GlobalState } from './abstractCommand'
import type PeerId from 'peer-id'
import { checkPeerIdInput, styleValue } from './utils'

export default class Addresses extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
    this.hidden = true
  }

  public name() {
    return 'addresses'
  }

  public help() {
    return 'Get the known addresses of a specific node'
  }

  public async execute(log: (toLog: string) => void, query: string, state: GlobalState): Promise<void> {
    let peerId: PeerId

    if (query != undefined && query.trim().length > 0) {
      try {
        peerId = checkPeerIdInput(query, state)
      } catch (err) {
        return log(styleValue(err.message, 'failure'))
      }
    }
    const announcedAddresses = await this.node.getAnnouncedAddresses(peerId)
    const announcedAddressesStr = announcedAddresses.map((a) => `\n- ${a.toString()}`)
    const observedAddresses = await this.node.getObservedAddresses(peerId)
    const observedAddressesStr = observedAddresses.map((a) => `\n- ${a.toString()}`)
    const msgAnnounced = `Announced addresses for ${query}:${announcedAddressesStr.join('')}`
    const msgObserved = `Observed addresses for ${query}:${observedAddressesStr.join('')}`

    return log(`${msgAnnounced}\n${msgObserved}`)
  }
}
