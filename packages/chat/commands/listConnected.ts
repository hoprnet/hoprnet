import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { getPeersIdsAsString, styleValue } from '../utils'

export default class ListConnectedPeers extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'listConnectedPeers'
  }

  public help() {
    return 'Lists connected HOPR nodes'
  }

  public async execute(): Promise<string | void> {
    const peerIds = getPeersIdsAsString(this.node).map((peerId) => styleValue(peerId, 'peerId'))
    if (peerIds.length == 0) {
      return 'Not currently connected to any peers'
    }

    return `Connected to: \n - ${peerIds.join('\n - ')}\n`
  }
}
