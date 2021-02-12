import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import type PeerId from 'peer-id'
import { checkPeerIdInput, styleValue } from './utils'
import type { GlobalState } from './abstractCommand'

export default class Addresses extends AbstractCommand {
  public hidden = true

  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'addresses'
  }

  public help() {
    return 'List addresses of other nodes'
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

    return (await this.node._libp2p.peerRouting.findPeer(peerId)).multiaddrs.map((ma) => ma.toString()).join(', ')
  }
}
