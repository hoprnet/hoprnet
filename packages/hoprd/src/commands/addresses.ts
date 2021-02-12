import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import type PeerId from 'peer-id'
import { checkPeerIdInput, styleValue } from './utils'
import type { GlobalState } from './abstractCommand'

export default class Addresses extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
    this.hidden = true
  }

  public name() {
    return 'addresses'
  }

  public help() {
    return 'Get the announced addresses from other nodes'
  }

  public async execute(query: string, state: GlobalState): Promise<string | void> {
    if (!query) {
      return `Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query, state)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    return `DHT record for ${query}:\n- ${(await this.node._libp2p.peerRouting.findPeer(peerId)).multiaddrs
      .map((ma) => ma.toString())
      .join('\n- ')}`
  }
}
